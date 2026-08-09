import { ADDON_ACCESS_LEVEL, getAddonAccessLevel } from './manifest'
import { beginTrustedActivation, clearTrustedActivationMarker } from './trustedAddonBootGuard'
import { loadTrustedAddonModuleGraph, revokeTrustedAddonModuleGraph } from './trustedAddonModuleLoader'

const COMMUNITY_ADDONS_PREF_KEY = 'addons.communityEnabled'
const TRUSTED_SAFE_MODE_PREF_KEY = 'addons.trustedSafeMode'
const TRUSTED_SAFE_MODE_LOCAL_KEY = 'elephantnote:addons:trusted-safe-mode'
const TRUST_APPROVAL_PREFIX = 'addons.trustedApproval.'

const getTauriCore = (target = globalThis) => target?.__TAURI__?.core || null

const invoke = (command, payload = {}, target = globalThis) => {
  const core = getTauriCore(target)
  if (!core?.invoke) throw new Error(`Tauri command API is unavailable for ${command}`)
  return core.invoke(command, payload)
}

const getPreference = (key, target = globalThis) => invoke('tauri_prefs_get', { key }, target)
const setPreference = (key, value, target = globalThis) => invoke('tauri_prefs_set', { key, value }, target)

const safeString = (value, fallback = '') => (typeof value === 'string' ? value.trim() || fallback : fallback)

export const isOfficialTrustedRecord = (record = {}) => (
  record.source === 'official' ||
  record.official === true ||
  record.manifest?.source === 'official' ||
  record.manifest?.official === true
)

export const shouldEnforceCommunityTrust = (record = {}) => !isOfficialTrustedRecord(record)

const approvalKey = (addonId) => `${TRUST_APPROVAL_PREFIX}${addonId}`

export const isTrustedExternalManifest = (manifest = {}) => {
  return manifest?.source === 'external' && getAddonAccessLevel(manifest) === ADDON_ACCESS_LEVEL.trusted
}

export const getTrustedSafeMode = async (target = globalThis) => {
  const localValue = target?.localStorage?.getItem?.(TRUSTED_SAFE_MODE_LOCAL_KEY)
  if (localValue === 'true') return true
  try {
    return await getPreference(TRUSTED_SAFE_MODE_PREF_KEY, target) === true
  } catch {
    return false
  }
}

export const setTrustedSafeMode = async (enabled, target = globalThis) => {
  const value = enabled === true
  target?.localStorage?.setItem?.(TRUSTED_SAFE_MODE_LOCAL_KEY, String(value))
  await setPreference(TRUSTED_SAFE_MODE_PREF_KEY, value, target)
  return value
}

export const getTrustedApproval = async (record, target = globalThis) => {
  const id = safeString(record?.manifest?.id)
  const packageHash = safeString(record?.packageHash || record?.manifest?.packageHash)
  if (!id || !packageHash) return { approved: false, approvedHash: '', packageHash }
  const approvedHash = safeString(await getPreference(approvalKey(id), target))
  return {
    approved: approvedHash === packageHash,
    approvedHash,
    packageHash
  }
}

export const approveTrustedAddon = async (record, target = globalThis) => {
  const id = safeString(record?.manifest?.id)
  const packageHash = safeString(record?.packageHash || record?.manifest?.packageHash)
  if (!id || !packageHash) throw new Error('Trusted addon approval requires an installed package hash')
  await setPreference(approvalKey(id), packageHash, target)
  return { approved: true, approvedHash: packageHash, packageHash }
}

export const revokeTrustedAddon = async (record, target = globalThis) => {
  const id = safeString(record?.manifest?.id)
  if (!id) throw new Error('Trusted addon id is required')
  await setPreference(approvalKey(id), '', target)
  return { approved: false, approvedHash: '', packageHash: safeString(record?.packageHash) }
}

const requireFunction = (value, label) => {
  if (typeof value !== 'function') throw new TypeError(`${label} must be a function`)
  return value
}

const addDisposable = (context, disposables, dispose) => {
  requireFunction(dispose, 'dispose')
  let active = true
  const once = () => {
    if (!active) return
    active = false
    dispose()
  }
  disposables.push(once)
  context.addDisposable?.(once)
  return once
}

export const createTrustedAddonApi = (record, context, sessionDisposables = [], target = globalThis) => {
  const manifest = record.manifest
  const documentRef = target?.document
  const host = context.addonHost
  const register = (area, contribution) => context.registerContribution(area, contribution)
  const track = (dispose) => addDisposable(context, sessionDisposables, dispose)
  const callBroker = (method, params = {}) => invoke('tauri_addons_call', {
    addonId: manifest.id,
    method,
    params
  }, target)

  const api = {
    manifest,
    access: Object.freeze({
      level: ADDON_ACCESS_LEVEL.trusted,
      packageHash: safeString(record.packageHash),
      nativeRequested: manifest.permissions?.native === true
    }),
    app: Object.freeze({
      router: context.router,
      pinia: context.pinia,
      services: context.services,
      runtime: context.runtime,
      addons: context.addons,
      host,
      vueApp: context.vueApp,
      openSettings(section = 'addons') {
        target?.dispatchEvent?.(new CustomEvent('elephantnote:open-settings', { detail: { section } }))
      },
      emit(name, detail) {
        target?.dispatchEvent?.(new CustomEvent(name, { detail }))
      }
    }),
    storage: Object.freeze({
      get: (key) => callBroker('storage.get', { key }),
      set: (key, value) => callBroker('storage.set', { key, value }),
      remove: (key) => callBroker('storage.remove', { key }),
      entries: () => callBroker('storage.entries')
    }),
    native: Object.freeze({
      status: () => invoke('tauri_addons_sidecar_status', { addonId: manifest.id }, target),
      call(method, params = {}, options = {}) {
        const timeoutMs = Number.isFinite(options?.timeoutMs) ? Math.max(1, Math.trunc(options.timeoutMs)) : undefined
        return invoke('tauri_addons_sidecar_call', {
          addonId: manifest.id,
          method: safeString(method),
          params,
          timeoutMs
        }, target)
      },
      service: Object.freeze({
        status: () => invoke('tauri_addons_service_status', { addonId: manifest.id }, target),
        start: () => invoke('tauri_addons_service_start', { addonId: manifest.id }, target),
        call(method, params = {}, options = {}) {
          const timeoutMs = Number.isFinite(options?.timeoutMs) ? Math.max(1, Math.trunc(options.timeoutMs)) : undefined
          return invoke('tauri_addons_service_call', {
            addonId: manifest.id,
            method: safeString(method),
            params,
            timeoutMs
          }, target)
        },
        stop: () => invoke('tauri_addons_service_stop', { addonId: manifest.id }, target)
      })
    }),
    resources: Object.freeze({
      get: (name) => host?.get(name),
      has: (name) => host?.has(name) || false,
      list: () => host?.list() || [],
      provide(name, value) {
        if (!host) throw new Error('Addon host registry is unavailable')
        return track(host.provide(name, value))
      },
      watch(name, listener, options) {
        if (!host) throw new Error('Addon host registry is unavailable')
        return track(host.watch(name, listener, options))
      }
    }),
    patch: Object.freeze({
      method(object, methodName, wrapper) {
        if (!host) throw new Error('Addon host registry is unavailable')
        return track(host.patchMethod(object, methodName, wrapper))
      },
      property(object, propertyName, value) {
        if (!host) throw new Error('Addon host registry is unavailable')
        return track(host.patchProperty(object, propertyName, value))
      },
      hook(name, handler) {
        if (!host) throw new Error('Addon host registry is unavailable')
        return track(host.registerHook(name, handler))
      },
      async runHook(name, payload) {
        if (!host) throw new Error('Addon host registry is unavailable')
        return await host.runHook(name, payload)
      }
    }),
    workspace: Object.freeze({
      registerView: context.addView,
      registerSidebarItem: context.addSidebarItem,
      registerStatusBarItem: context.addStatusBarItem,
      registerContribution: register,
      openView(viewId, params = {}) {
        target?.dispatchEvent?.(new CustomEvent('elephantnote:open-addon-view', {
          detail: { viewId, params, addonId: manifest.id }
        }))
      }
    }),
    editor: Object.freeze({
  get active() {
    return host?.get('editor.runtime') || host?.get('editor') || null
  },
  watch(listener, options) {
    if (!host) throw new Error('Addon host registry is unavailable')
    return track(host.watch('editor.runtime', listener, options))
  },
      registerExtension: context.addEditorExtension,
      registerBlockType(definition) {
        return register('editor.block-types', definition)
      },
      registerInlineType(definition) {
        return register('editor.inline-types', definition)
      },
      registerInputRule(definition) {
        return register('editor.input-rules', definition)
      },
      registerToolbarItem(definition) {
        return register('editor.toolbar-items', definition)
      },
      registerPasteHandler(definition) {
        return register('editor.paste-handlers', definition)
      }
    }),
    markdown: Object.freeze({
      registerPostProcessor(definition) {
        return register('markdown.post-processors', definition)
      },
      registerCodeBlockProcessor(definition) {
        return register('markdown.code-block-processors', definition)
      },
      registerEmbedRenderer(definition) {
        return register('markdown.embed-renderers', definition)
      }
    }),
    settings: Object.freeze({
      registerSection: context.addSettingsSection,
      registerPage(definition) {
        return register('settings.pages', definition)
      }
    }),
    layout: Object.freeze({
      registerItem(definition) {
        return register('layout.items', definition)
      },
      registerZone(definition) {
        return register('layout.zones', definition)
      }
    }),
    commands: Object.freeze({
      register: context.addAction
    }),
    router: Object.freeze({
      addRoute(...args) {
        if (!context.router?.addRoute) throw new Error('Vue Router is unavailable')
        return track(context.router.addRoute(...args))
      },
      beforeEach(guard) {
        if (!context.router?.beforeEach) throw new Error('Vue Router is unavailable')
        return track(context.router.beforeEach(guard))
      },
      afterEach(guard) {
        if (!context.router?.afterEach) throw new Error('Vue Router is unavailable')
        return track(context.router.afterEach(guard))
      }
    }),
    vue: Object.freeze({
      component(name, component) {
        const app = context.vueApp
        if (!app?.component) throw new Error('Vue application is unavailable')
        const previous = app.component(name)
        app.component(name, component)
        return track(() => {
          if (previous) app.component(name, previous)
          else delete app._context?.components?.[name]
        })
      },
      directive(name, directive) {
        const app = context.vueApp
        if (!app?.directive) throw new Error('Vue application is unavailable')
        const previous = app.directive(name)
        app.directive(name, directive)
        return track(() => {
          if (previous) app.directive(name, previous)
          else delete app._context?.directives?.[name]
        })
      },
      provide(key, value) {
        const app = context.vueApp
        if (!app?.provide) throw new Error('Vue application is unavailable')
        const provides = app._context?.provides
        const hadPrevious = provides && Object.prototype.hasOwnProperty.call(provides, key)
        const previous = provides?.[key]
        app.provide(key, value)
        return track(() => {
          if (!provides) return
          if (hadPrevious) provides[key] = previous
          else delete provides[key]
        })
      }
    }),
    ui: Object.freeze({
      registerStyle(cssText, id = '') {
        if (!documentRef?.head) throw new Error('Document is unavailable')
        const style = documentRef.createElement('style')
        style.dataset.elephantAddon = manifest.id
        if (id) style.dataset.elephantAddonStyle = safeString(id)
        style.textContent = String(cssText || '')
        documentRef.head.appendChild(style)
        return track(() => style.remove())
      },
      mount(selectorOrElement, renderer) {
        if (!host) throw new Error('Addon host registry is unavailable')
        return track(host.mount(selectorOrElement, renderer))
      },
      on(eventTarget, eventName, listener, options) {
        if (!eventTarget?.addEventListener || !eventTarget?.removeEventListener) {
          throw new TypeError('Event target must implement addEventListener/removeEventListener')
        }
        requireFunction(listener, 'listener')
        eventTarget.addEventListener(eventName, listener, options)
        return track(() => eventTarget.removeEventListener(eventName, listener, options))
      },
      observe(element, listener, options = { childList: true, subtree: true }) {
        if (!target?.MutationObserver) throw new Error('MutationObserver is unavailable')
        requireFunction(listener, 'listener')
        const observer = new target.MutationObserver(listener)
        observer.observe(element, options)
        return track(() => observer.disconnect())
      }
    }),
    lifecycle: Object.freeze({
      addDisposable(dispose) {
        return track(dispose)
      }
    }),
    experimental: Object.freeze({
      window: target,
      document: documentRef,
      tauri: target?.__TAURI__,
      router: context.router,
      pinia: context.pinia,
      services: context.services,
      vueApp: context.vueApp,
      host,
      rawContext: context
    })
  }

  return Object.freeze(api)
}

const resolvePluginInstance = (module, api) => {
  const exported = module?.default ?? module?.plugin ?? module
  if (typeof exported === 'function') {
    const PluginConstructor = exported
    try {
      return new PluginConstructor(api)
    } catch (error) {
      if (/not a constructor/i.test(error?.message || '')) return exported(api)
      throw error
    }
  }
  return exported
}

export class TrustedAddonSession {
  constructor(record, logger, target = globalThis) {
    this.record = record
    this.logger = logger
    this.target = target
    this.module = null
    this.plugin = null
    this.api = null
    this.moduleUrl = ''
    this.moduleUrls = []
    this.disposables = []
    this.activationDispose = null
  }

  async start(context) {
    const addonId = this.record.manifest.id
    const graph = await loadTrustedAddonModuleGraph({
      addonId,
      entryPath: this.record.manifest.runtime?.entry || 'main.js',
      readModule: (path) => invoke('tauri_addons_read_module', { addonId, path }, this.target)
    })
    this.moduleUrl = graph.entryUrl
    this.moduleUrls = graph.urls
    this.module = await import(/* @vite-ignore */ this.moduleUrl)
    this.api = createTrustedAddonApi(this.record, context, this.disposables, this.target)
    this.plugin = resolvePluginInstance(this.module, this.api)

    const activate = this.plugin?.onload || this.plugin?.activate || this.module?.activate
    if (typeof activate !== 'function') {
      throw new Error('Trusted addon entry must export default { onload(api) {} } or an activate(api) function')
    }
    const dispose = await activate.call(this.plugin, this.api)
    if (typeof dispose === 'function') this.activationDispose = dispose
  }

  async stop() {
    const deactivate = this.plugin?.onunload || this.plugin?.deactivate || this.module?.deactivate
    try {
      if (typeof deactivate === 'function') await deactivate.call(this.plugin, this.api)
      if (typeof this.activationDispose === 'function') await this.activationDispose()
    } finally {
      while (this.disposables.length) {
        const dispose = this.disposables.pop()
        try {
          dispose()
        } catch (error) {
          this.logger?.warn?.('trusted addon cleanup failed', {
            id: this.record.manifest.id,
            error: error?.message || String(error)
          })
        }
      }
      revokeTrustedAddonModuleGraph(this.moduleUrls)
      this.moduleUrls = []
      this.moduleUrl = ''
      this.module = null
      this.plugin = null
      this.api = null
      this.activationDispose = null
    }
  }
}

export const createTrustedAddonDefinition = (record, logger) => {
  let session = null
  return {
    manifest: {
      ...record.manifest,
      source: 'external',
      packageHash: record.packageHash,
      installedAt: record.installedAt,
      defaultEnabled: false
    },
    async activate(context) {
      if (shouldEnforceCommunityTrust(record)) {
        const communityEnabled = await getPreference(COMMUNITY_ADDONS_PREF_KEY)
        if (communityEnabled !== true) {
          throw new Error('Community addons are disabled. Turn them on in Settings → Addons first.')
        }
        if (await getTrustedSafeMode()) {
          throw new Error('Trusted addon safe mode is enabled. Disable safe mode before starting full app access addons.')
        }
        const approval = await getTrustedApproval(record)
        if (!approval.approved) {
          const error = new Error('Full app access approval is required for this exact addon package.')
          error.code = 'TRUST_REQUIRED'
          error.addonId = record.manifest.id
          error.packageHash = record.packageHash
          throw error
        }
      }

      beginTrustedActivation(record)
      session = new TrustedAddonSession(record, logger)
      try {
        await session.start(context)
        clearTrustedActivationMarker()
        await invoke('tauri_addons_set_enabled', { addonId: record.manifest.id, enabled: true })
      } catch (error) {
        clearTrustedActivationMarker()
        await session.stop().catch(() => {})
        session = null
        await invoke('tauri_addons_set_enabled', { addonId: record.manifest.id, enabled: false }).catch(() => {})
        throw error
      }
      return () => session?.stop()
    },
    async deactivate() {
      await session?.stop()
      session = null
      await invoke('tauri_addons_set_enabled', { addonId: record.manifest.id, enabled: false })
    }
  }
}
