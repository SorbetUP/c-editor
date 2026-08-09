import { inject } from 'vue'
import { createAddonHostRuntime } from './addonHostRuntime'
import { ElephantAddonManager } from './AddonManagerWithState'
import { builtinAddons } from './builtin'
import { installExternalAddonRuntime } from './externalAddonRuntime'
import { installSettingsContributionRuntime } from './settingsContributionRuntime'
import { installAddonContentFallbackRuntime } from './addonContentFallbackRuntime'
import { normalizeAddonManifest } from './manifest'
import { useAddonsStore } from '@/store/addons'
export { ADDON_EXTENSION_POINTS } from './extensionPoints'
export { createAddonHostRuntime } from './addonHostRuntime'
export {
  ADDON_ACCESS_LEVEL,
  ADDON_API_VERSION,
  ADDON_STATUS,
  getAddonAccessLevel,
  isTrustedAddonManifest,
  normalizeAddonManifest
} from './manifest'
export { ElephantAddonManager } from './AddonManagerWithState'
export {
  getAddonActions,
  getAddonContributions,
  getAddonSettingsSections,
  getAddonSidebarItems
} from './contributionSelectors'

export const ADDON_MANAGER_KEY = Symbol('ElephantAddonManager')

const BUILTIN_INSTALL_STORAGE_KEY = 'elephantnote:installed-built-in-addons:v1'
const BUILTIN_ENABLED_STORAGE_KEY = 'elephantnote:enabled-built-in-addons:v1'
const REQUIRED_BUILTIN_ADDON_IDS = Object.freeze(['elephant.addon-packs', 'elephant.excalidraw'])

const createDiagnosticsLogger = (logger) => ({
  info: logger?.info || ((...args) => console.info('[addons]', ...args)),
  warn: logger?.warn || ((...args) => console.warn('[addons]', ...args)),
  error: logger?.error || ((...args) => console.error('[addons]', ...args))
})

const getLocalStorage = () => {
  try {
    return globalThis?.window?.localStorage || globalThis?.localStorage || null
  } catch {
    return null
  }
}

const parseStoredIds = (storage, key, allowedIds) => {
  try {
    const raw = storage?.getItem(key)
    if (!raw) return null
    const parsed = JSON.parse(raw)
    if (!Array.isArray(parsed)) return null
    return parsed.filter((id) => allowedIds.has(id))
  } catch {
    return null
  }
}

const readInstalledBuiltinIds = (availableIds) => {
  const available = new Set(availableIds)
  const required = REQUIRED_BUILTIN_ADDON_IDS.filter((id) => available.has(id))
  const storage = getLocalStorage()
  if (!storage) return new Set(required)
  const stored = parseStoredIds(storage, BUILTIN_INSTALL_STORAGE_KEY, available)
  return new Set([...required, ...(stored || [])])
}

const readEnabledBuiltinIds = (manager, availableIds) => {
  const installed = manager.list()
    .filter((addon) => addon.manifest.source === 'builtin')
  const installedIds = new Set(installed.map((addon) => addon.manifest.id))
  const allowed = new Set(availableIds.filter((id) => installedIds.has(id)))
  const required = REQUIRED_BUILTIN_ADDON_IDS.filter((id) => allowed.has(id))
  const storage = getLocalStorage()
  const stored = storage ? parseStoredIds(storage, BUILTIN_ENABLED_STORAGE_KEY, allowed) : null
  if (stored) return new Set([...required, ...stored])
  const defaults = installed
    .filter((addon) => addon.manifest.defaultEnabled)
    .map((addon) => addon.manifest.id)
  return new Set([...required, ...defaults])
}

const persistInstalledBuiltinIds = (manager, availableIds) => {
  const storage = getLocalStorage()
  if (!storage) return
  const available = new Set(availableIds)
  const installed = manager.list()
    .filter((addon) => addon.manifest.source === 'builtin' && available.has(addon.manifest.id))
    .map((addon) => addon.manifest.id)
    .sort()
  storage.setItem(BUILTIN_INSTALL_STORAGE_KEY, JSON.stringify(installed))
}

const persistEnabledBuiltinIds = (manager, availableIds) => {
  const storage = getLocalStorage()
  if (!storage) return
  const available = new Set(availableIds)
  const enabled = manager.list()
    .filter((addon) => addon.manifest.source === 'builtin' && addon.enabled && available.has(addon.manifest.id))
    .map((addon) => addon.manifest.id)
    .sort()
  storage.setItem(BUILTIN_ENABLED_STORAGE_KEY, JSON.stringify(enabled))
}

const createAddonManagerFacade = (managerRef) => Object.freeze({
  list: (...args) => managerRef.current?.list(...args) || [],
  get: (...args) => managerRef.current?.get(...args) || null,
  getContributions: (...args) => managerRef.current?.getContributions(...args) || [],
  getContributionMap: () => managerRef.current?.getContributionMap() || {},
  listBuiltinCatalog: () => managerRef.current?.listBuiltinCatalog?.() || [],
  installBuiltin: (...args) => managerRef.current?.installBuiltin?.(...args),
  uninstallBuiltin: (...args) => managerRef.current?.uninstallBuiltin?.(...args),
  enable: (...args) => managerRef.current?.enable(...args),
  disable: (...args) => managerRef.current?.disable(...args),
  runAction: (...args) => managerRef.current?.runAction(...args),
  on: (...args) => managerRef.current?.on(...args) || (() => {}),
  get external() { return managerRef.current?.external || null }
})

export const createAddonManager = (options = {}) => {
  const logger = createDiagnosticsLogger(options.logger)
  const usesDefaultBuiltinCatalog = !Object.prototype.hasOwnProperty.call(options, 'addons')
  const addonDefinitions = options.addons || builtinAddons
  const definitionsById = new Map(addonDefinitions.map((addon) => [addon?.manifest?.id, addon]).filter(([id]) => id))
  const availableIds = [...definitionsById.keys()]
  const managerRef = { current: null }
  const addonManagerFacade = createAddonManagerFacade(managerRef)
  const addonHost = options.addonHost || createAddonHostRuntime({ ...options, logger })
  const manager = new ElephantAddonManager({
    ...options,
    addonHost,
    addons: addonManagerFacade,
    logger
  })
  managerRef.current = manager
  manager.host = addonHost
  addonHost.provide('addons', addonManagerFacade)
  addonHost.provide('addonManager', manager)

  const installedIds = usesDefaultBuiltinCatalog
    ? readInstalledBuiltinIds(availableIds)
    : new Set(availableIds)

  manager.listBuiltinCatalog = () => addonDefinitions.map((definition) => {
    const manifest = manager.get(definition.manifest.id)?.manifest || normalizeAddonManifest(definition.manifest)
    return { manifest, installed: Boolean(manager.get(manifest.id)) }
  })

  manager.installBuiltin = async (id) => {
    const definition = definitionsById.get(id)
    if (!definition) throw new Error(`Unknown built-in addon: ${id}`)
    const existing = manager.get(id)
    if (existing) return existing
    const registered = manager.register(definition)
    if (usesDefaultBuiltinCatalog) {
      persistInstalledBuiltinIds(manager, availableIds)
      persistEnabledBuiltinIds(manager, availableIds)
    }
    logger.info('[addons] builtin:installed', { id })
    return registered
  }

  const disableAddon = manager.disable.bind(manager)
  manager.disable = async (id) => {
    if (REQUIRED_BUILTIN_ADDON_IDS.includes(id)) {
      const current = manager.get(id)
      if (!current) throw new Error(`Required built-in addon is not installed: ${id}`)
      if (!current.enabled) return manager.enable(id)
      return current
    }
    return disableAddon(id)
  }

  manager.uninstallBuiltin = async (id) => {
    const current = manager.get(id)
    if (!current || current.manifest.source !== 'builtin') throw new Error(`Built-in addon is not installed: ${id}`)
    if (current.manifest.removable === false || REQUIRED_BUILTIN_ADDON_IDS.includes(id)) {
      throw new Error(`${current.manifest.name} is required by the addon manager and cannot be removed.`)
    }
    if (current.enabled || current.status === 'error') await manager.disable(id)
    const removed = manager.unregister(id)
    if (usesDefaultBuiltinCatalog) {
      persistInstalledBuiltinIds(manager, availableIds)
      persistEnabledBuiltinIds(manager, availableIds)
    }
    logger.info('[addons] builtin:removed', { id })
    return removed
  }

  manager.restoreBuiltinEnabledState = async () => {
    const enabledIds = usesDefaultBuiltinCatalog
      ? readEnabledBuiltinIds(manager, availableIds)
      : new Set(manager.list()
        .filter((addon) => addon.manifest.defaultEnabled)
        .map((addon) => addon.manifest.id))
    for (const addon of manager.list()) {
      if (addon.manifest.source !== 'builtin' || !enabledIds.has(addon.manifest.id)) continue
      await manager.enable(addon.manifest.id)
    }
    if (usesDefaultBuiltinCatalog) persistEnabledBuiltinIds(manager, availableIds)
  }

  logger.info('[addons] register:start', {
    count: installedIds.size,
    available: addonDefinitions.length,
    ids: [...installedIds]
  })
  for (const id of installedIds) {
    const addon = definitionsById.get(id)
    if (!addon) continue
    const registered = manager.register(addon)
    logger.info('[addons] register:done', {
      id: registered.manifest.id,
      defaultEnabled: Boolean(registered.manifest.defaultEnabled)
    })
  }

  if (usesDefaultBuiltinCatalog) {
    const persistEnabledState = (snapshot) => {
      if (snapshot?.manifest?.source === 'builtin') persistEnabledBuiltinIds(manager, availableIds)
    }
    manager.on('enabled', persistEnabledState)
    manager.on('disabled', persistEnabledState)
  }

  return manager
}

export const installAddonSystem = (app, options = {}) => {
  const bootstrapLogger = createDiagnosticsLogger(options.logger)
  bootstrapLogger.info('[addons] install:start', {
    runtime: options.runtime || 'unknown',
    hasPinia: Boolean(options.pinia),
    hasTauriInvoke: Boolean(globalThis?.__TAURI__?.core?.invoke)
  })

  const manager = createAddonManager({
    ...options,
    vueApp: app,
    logger: bootstrapLogger
  })
  app.provide(ADDON_MANAGER_KEY, manager)
  app.config.globalProperties.$addons = manager
  app.config.globalProperties.$addonHost = manager.host

  if (options.pinia) {
    useAddonsStore(options.pinia).install(manager)
    manager.logger.info('[addons] store:installed', { registered: manager.list().length })
  } else {
    manager.logger.warn('[addons] store:not-installed')
  }

  manager.settingsContributions = installSettingsContributionRuntime(manager)
  manager.contentFallbacks = installAddonContentFallbackRuntime(manager)

  if (typeof window !== 'undefined') {
    window.__ELEPHANT_ADDONS__ = manager
    window.__ELEPHANT_ADDON_HOST__ = manager.host
    window.__ELEPHANT_VUE_APP__ = app
    window.dispatchEvent(new CustomEvent('elephantnote:addons-ready', {
      detail: {
        ids: manager.list().map((addon) => addon.manifest.id),
        resources: manager.host.list()
      }
    }))
  }

  manager.restoreBuiltinEnabledState()
    .then(() => {
      manager.logger.info('[addons] builtin-state:restored', {
        enabled: manager.list().filter((addon) => addon.enabled).map((addon) => addon.manifest.id),
        actions: manager.getActions().map((entry) => entry.contribution?.id).filter(Boolean)
      })
      manager.contentFallbacks?.refresh?.()
    })
    .catch((error) => manager.logger.error('[addons] builtin-state:failed', error))

  if (globalThis?.__TAURI__?.core?.invoke) {
    manager.logger.info('[addons] external-runtime:install:start')
    installExternalAddonRuntime(manager, { logger: manager.logger })
    manager.logger.info('[addons] external-runtime:install:done')
  } else {
    manager.logger.warn('[addons] external-runtime:skipped', { reason: 'tauri invoke unavailable' })
  }

  manager.logger.info('[addons] install:done', {
    registered: manager.list().map((addon) => addon.manifest.id),
    availableBuiltins: manager.listBuiltinCatalog().map((entry) => entry.manifest.id),
    resources: manager.host.list()
  })
  return manager
}

export const useAddonManager = () => {
  const manager = inject(ADDON_MANAGER_KEY)
  if (!manager) throw new Error('Addon manager is not installed')
  return manager
}
