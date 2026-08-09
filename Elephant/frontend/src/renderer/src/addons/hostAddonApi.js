import { ADDON_EXTENSION_POINTS, isKnownExtensionPoint } from './extensionPoints'

const HOST_ADDON_API_VERSION = 1
const EXTENSION_POINTS = Object.freeze(Object.values(ADDON_EXTENSION_POINTS))
const lifecycleControllers = new WeakMap()

const requireString = (value, label) => {
  if (typeof value !== 'string' || !value.trim()) {
    throw new TypeError(`${label} must be a non-empty string`)
  }
  return value.trim()
}

const requireFunction = (value, label) => {
  if (typeof value !== 'function') throw new TypeError(`${label} must be a function`)
  return value
}

const once = (dispose) => {
  let active = true
  return () => {
    if (!active) return
    active = false
    return dispose()
  }
}

const toDisposable = (value) => {
  if (typeof value === 'function') return value
  if (value && typeof value.dispose === 'function') return () => value.dispose()
  if (value && typeof value.unsubscribe === 'function') return () => value.unsubscribe()
  if (value && typeof value.abort === 'function') return () => value.abort()
  throw new TypeError('disposable must be a function or expose dispose(), unsubscribe() or abort()')
}

const createFallbackAbortController = () => {
  const listeners = new Set()
  const signal = {
    aborted: false,
    addEventListener(name, listener) {
      if (name === 'abort' && typeof listener === 'function') listeners.add(listener)
    },
    removeEventListener(name, listener) {
      if (name === 'abort') listeners.delete(listener)
    }
  }
  return {
    signal,
    abort() {
      if (signal.aborted) return
      signal.aborted = true
      for (const listener of listeners) listener()
      listeners.clear()
    }
  }
}

const ensureLifecycle = (record) => {
  const current = lifecycleControllers.get(record)
  if (current && !current.signal.aborted) return current
  const controller = typeof AbortController === 'function'
    ? new AbortController()
    : createFallbackAbortController()
  lifecycleControllers.set(record, controller)
  return controller
}

export const abortHostAddonApi = (record) => {
  const controller = lifecycleControllers.get(record)
  if (!controller) return false
  controller.abort()
  lifecycleControllers.delete(record)
  return true
}

const normalizeBatch = (definitions) => {
  if (Array.isArray(definitions)) return definitions
  if (!definitions || typeof definitions !== 'object') {
    throw new TypeError('contribution definitions must be an array or object map')
  }
  return Object.entries(definitions).flatMap(([area, contributions]) => {
    const entries = Array.isArray(contributions) ? contributions : [contributions]
    return entries.map((contribution) => ({ area, contribution }))
  })
}

const contributionId = (entry) => {
  const id = entry?.contribution?.id
  return typeof id === 'string' ? id.trim() : ''
}

const createStorageApi = (storage) => Object.freeze({
  get: (key, fallback) => storage.get(key, fallback),
  set: (key, value) => storage.set(key, value),
  remove: (key) => storage.remove(key),
  clear: () => storage.clear(),
  entries: () => storage.entries(),
  async has(key) {
    const entries = await storage.entries()
    return Object.prototype.hasOwnProperty.call(entries, key)
  },
  async keys() {
    return Object.keys(await storage.entries()).sort()
  },
  async update(key, updater, fallback = undefined) {
    requireFunction(updater, 'storage updater')
    const previous = await storage.get(key, fallback)
    const next = await updater(previous)
    await storage.set(key, next)
    return next
  }
})

const createSchedulerApi = (addDisposable, target = globalThis) => Object.freeze({
  timeout(callback, delay = 0) {
    requireFunction(callback, 'timeout callback')
    const id = target.setTimeout(callback, Math.max(0, Number(delay) || 0))
    const dispose = once(() => target.clearTimeout(id))
    addDisposable(dispose)
    return Object.freeze({ id, dispose })
  },
  interval(callback, delay = 0) {
    requireFunction(callback, 'interval callback')
    const id = target.setInterval(callback, Math.max(1, Number(delay) || 1))
    const dispose = once(() => target.clearInterval(id))
    addDisposable(dispose)
    return Object.freeze({ id, dispose })
  }
})

export const createHostAddonContext = (manager, record, legacyContext) => {
  const addonId = record.manifest.id
  const controller = ensureLifecycle(record)
  const addDisposable = (value) => {
    const rawDispose = toDisposable(value)
    const dispose = once(() => {
      const result = rawDispose()
      if (result && typeof result.then === 'function') {
        void result.catch((error) => manager.logger.warn?.(`[addon:${addonId}] async cleanup failed`, error))
      }
      return result
    })
    legacyContext.addDisposable(dispose)
    return dispose
  }
  const qualifyId = (value) => {
    const id = requireString(value, 'id')
    return id.startsWith(`${addonId}.`) ? id : `${addonId}.${id}`
  }
  const ownsId = (value) => {
    const id = typeof value === 'string' ? value.trim() : ''
    return id === addonId || id.startsWith(`${addonId}.`)
  }
  const validateArea = (value) => {
    const area = requireString(value, 'area')
    if (!isKnownExtensionPoint(area)) throw new Error(`Unknown addon extension point: ${area}`)
    return area
  }
  const register = (area, contribution) => {
    const dispose = once(manager.registerContribution(addonId, validateArea(area), contribution))
    legacyContext.addDisposable(dispose)
    return dispose
  }
  const registerMany = (definitions) => {
    const normalized = normalizeBatch(definitions).map((definition) => ({
      area: validateArea(definition?.area),
      contribution: definition?.contribution
    }))
    const disposables = []
    try {
      for (const definition of normalized) {
        disposables.push(once(manager.registerContribution(addonId, definition.area, definition.contribution)))
      }
    } catch (error) {
      while (disposables.length) disposables.pop()()
      throw error
    }
    const dispose = once(() => {
      while (disposables.length) disposables.pop()()
    })
    legacyContext.addDisposable(dispose)
    return dispose
  }
  const ownContributions = (area = '') => {
    const areas = area ? [validateArea(area)] : EXTENSION_POINTS
    return areas.flatMap((currentArea) => manager.getContributions(currentArea)
      .filter((entry) => entry.addonId === addonId)
      .map((entry) => Object.freeze({ area: currentArea, entry })))
  }
  const getOwnContribution = (area, id) => {
    const contributionIdValue = requireString(id, 'contributionId')
    return manager.getContributions(validateArea(area))
      .find((entry) => entry.addonId === addonId && contributionId(entry) === contributionIdValue) || null
  }
  const getOwnAction = (id) => {
    const actionId = requireString(id, 'actionId')
    return manager.getActions()
      .find((entry) => entry.addonId === addonId && contributionId(entry) === actionId) || null
  }
  const localEvent = (name) => `addon:${addonId}:${requireString(name, 'eventName')}`
  const subscribe = (name, listener, single = false) => {
    requireFunction(listener, 'event listener')
    const eventName = localEvent(name)
    let dispose = () => {}
    const wrapped = single
      ? (payload) => {
          dispose()
          listener(payload)
        }
      : listener
    dispose = manager.on(eventName, wrapped)
    return addDisposable(dispose)
  }
  const eventTarget = legacyContext.eventTarget || globalThis
  const dispatchHostEvent = (name, detail) => {
    const CustomEventCtor = eventTarget?.CustomEvent || globalThis.CustomEvent
    if (typeof eventTarget?.dispatchEvent !== 'function' || typeof CustomEventCtor !== 'function') return false
    eventTarget.dispatchEvent(new CustomEventCtor(name, { detail }))
    return true
  }
  const log = Object.freeze({
    debug: (...args) => (manager.logger.debug || manager.logger.info)?.call(manager.logger, `[addon:${addonId}]`, ...args),
    info: (...args) => manager.logger.info?.(`[addon:${addonId}]`, ...args),
    warn: (...args) => manager.logger.warn?.(`[addon:${addonId}]`, ...args),
    error: (...args) => manager.logger.error?.(`[addon:${addonId}]`, ...args)
  })
  const host = legacyContext.addonHost
  const resource = Object.freeze({
    get: (name) => host?.get(name),
    has: (name) => host?.has(name) || false,
    list: () => host?.list() || [],
    provide(name, value) {
      if (!host) throw new Error('Addon host registry is unavailable')
      return addDisposable(host.provide(requireString(name, 'resource name'), value))
    },
    watch(name, listener, options) {
      if (!host) throw new Error('Addon host registry is unavailable')
      return addDisposable(host.watch(requireString(name, 'resource name'), listener, options))
    }
  })
  const hooks = Object.freeze({
    register(name, handler) {
      if (!host) throw new Error('Addon host registry is unavailable')
      return addDisposable(host.registerHook(requireString(name, 'hook name'), handler))
    },
    run: (name, payload) => {
      if (!host) throw new Error('Addon host registry is unavailable')
      return host.runHook(requireString(name, 'hook name'), payload)
    }
  })

  const api = Object.freeze({
    version: HOST_ADDON_API_VERSION,
    manifest: record.manifest,
    ids: Object.freeze({ qualify: qualifyId, owns: ownsId }),
    log,
    capabilities: Object.freeze({
      extensionPoints: EXTENSION_POINTS,
      supports: (area) => isKnownExtensionPoint(area),
      storage: Boolean(legacyContext.storage),
      resources: Boolean(host)
    }),
    app: Object.freeze({
      getAddon: (id = addonId) => manager.get(requireString(id, 'addonId')),
      listAddons: () => manager.list(),
      onChanged: (listener) => addDisposable(manager.on('changed', listener))
    }),
    contributions: Object.freeze({
      register,
      registerMany,
      list: ownContributions,
      get: getOwnContribution,
      has: (area, id) => Boolean(getOwnContribution(area, id))
    }),
    commands: Object.freeze({
      register: (definition) => register(ADDON_EXTENSION_POINTS.actions, definition),
      registerMany: (definitions) => registerMany({ [ADDON_EXTENSION_POINTS.actions]: definitions }),
      get: getOwnAction,
      async execute(id, payload) {
        const entry = getOwnAction(id)
        if (!entry) throw new Error(`Unknown action for ${addonId}: ${id}`)
        const run = entry.contribution?.run
        if (typeof run !== 'function') throw new TypeError(`Addon action is not executable: ${id}`)
        return await run(payload, { addonId, actionId: id, addons: manager })
      }
    }),
    workspace: Object.freeze({
      registerView: (definition) => register(ADDON_EXTENSION_POINTS.views, definition),
      registerPanel: (definition) => register(ADDON_EXTENSION_POINTS.workspacePanels, definition),
      registerSidebarItem: (definition) => register(ADDON_EXTENSION_POINTS.sidebarItems, definition),
      registerStatusBarItem: (definition) => register(ADDON_EXTENSION_POINTS.statusBarItems, definition),
      openView: (viewId, params = {}) => dispatchHostEvent('elephantnote:open-addon-view', {
        viewId: requireString(viewId, 'viewId'), params, addonId
      })
    }),
    settings: Object.freeze({
      registerSection: (definition) => register(ADDON_EXTENSION_POINTS.settingsSections, definition),
      registerPage: (definition) => register(ADDON_EXTENSION_POINTS.settingsPages, definition),
      open: (section = 'addons') => dispatchHostEvent('elephantnote:open-settings', {
        section: requireString(section, 'section'), addonId
      })
    }),
    editor: Object.freeze({
      registerExtension: (definition) => register(ADDON_EXTENSION_POINTS.editorExtensions, definition),
      registerBlockType: (definition) => register(ADDON_EXTENSION_POINTS.editorBlockTypes, definition),
      registerInlineType: (definition) => register(ADDON_EXTENSION_POINTS.editorInlineTypes, definition),
      registerInputRule: (definition) => register(ADDON_EXTENSION_POINTS.editorInputRules, definition),
      registerToolbarItem: (definition) => register(ADDON_EXTENSION_POINTS.editorToolbarItems, definition),
      registerFooterItem: (definition) => register(ADDON_EXTENSION_POINTS.editorFooterItems, definition),
      registerPasteHandler: (definition) => register(ADDON_EXTENSION_POINTS.editorPasteHandlers, definition)
    }),
    markdown: Object.freeze({
      registerPostProcessor: (definition) => register(ADDON_EXTENSION_POINTS.markdownPostProcessors, definition),
      registerCodeBlockProcessor: (definition) => register(ADDON_EXTENSION_POINTS.markdownCodeBlockProcessors, definition),
      registerEmbedRenderer: (definition) => register(ADDON_EXTENSION_POINTS.markdownEmbedRenderers, definition)
    }),
    layout: Object.freeze({
      registerItem: (definition) => register(ADDON_EXTENSION_POINTS.layoutItems, definition),
      registerZone: (definition) => register(ADDON_EXTENSION_POINTS.layoutZones, definition)
    }),
    ai: Object.freeze({ registerProvider: (definition) => register(ADDON_EXTENSION_POINTS.aiProviders, definition) }),
    imports: Object.freeze({ register: (definition) => register(ADDON_EXTENSION_POINTS.importers, definition) }),
    sites: Object.freeze({ registerGenerator: (definition) => register(ADDON_EXTENSION_POINTS.siteGenerators, definition) }),
    storage: createStorageApi(legacyContext.storage),
    resources: resource,
    hooks,
    scheduler: createSchedulerApi(addDisposable),
    events: Object.freeze({
      on: (name, listener) => subscribe(name, listener),
      once: (name, listener) => subscribe(name, listener, true),
      emit: (name, payload) => manager.emit(localEvent(name), payload)
    }),
    lifecycle: Object.freeze({
      signal: controller.signal,
      addDisposable,
      onAbort(listener) {
        requireFunction(listener, 'abort listener')
        controller.signal.addEventListener('abort', listener, { once: true })
        return addDisposable(() => controller.signal.removeEventListener('abort', listener))
      }
    })
  })

  const resourceName = `addon.api.${addonId}`
  if (host && !host.has(resourceName)) addDisposable(host.provide(resourceName, api))

  return Object.freeze({
    ...legacyContext,
    api,
    registerContributions: registerMany,
    addSettingsPage: api.settings.registerPage,
    addWorkspacePanel: api.workspace.registerPanel,
    addEditorBlockType: api.editor.registerBlockType,
    addEditorInlineType: api.editor.registerInlineType,
    addEditorInputRule: api.editor.registerInputRule,
    addEditorToolbarItem: api.editor.registerToolbarItem,
    addEditorFooterItem: api.editor.registerFooterItem,
    addEditorPasteHandler: api.editor.registerPasteHandler,
    addMarkdownPostProcessor: api.markdown.registerPostProcessor,
    addMarkdownCodeBlockProcessor: api.markdown.registerCodeBlockProcessor,
    addMarkdownEmbedRenderer: api.markdown.registerEmbedRenderer,
    addLayoutItem: api.layout.registerItem,
    addLayoutZone: api.layout.registerZone,
    addAiProvider: api.ai.registerProvider,
    addImporter: api.imports.register,
    addSiteGenerator: api.sites.registerGenerator
  })
}
