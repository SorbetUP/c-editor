import {
  approveTrustedAddon,
  createTrustedAddonDefinition,
  getTrustedApproval,
  getTrustedSafeMode,
  isTrustedExternalManifest,
  revokeTrustedAddon,
  setTrustedSafeMode
} from './trustedAddonRuntime'
import { createIsolatedAddonWorkerSource } from './isolatedAddonWorkerSource'

const COMMUNITY_ADDONS_PREF_KEY = 'addons.communityEnabled'

const getTauriCore = (target = globalThis) => target?.__TAURI__?.core || null
const invoke = (command, payload = {}, target = globalThis) => {
  const core = getTauriCore(target)
  if (!core?.invoke) throw new Error(`Tauri command API is unavailable for ${command}`)
  return core.invoke(command, payload)
}

export const externalAddonApi = Object.freeze({
  list: () => invoke('tauri_addons_list'),
  officialList: () => invoke('tauri_official_addons_catalog_list'),
  officialInstall: (addonId) => invoke('tauri_official_addons_catalog_install', { addonId }),
  install: (packagePath) => invoke('tauri_addons_install', { packagePath }),
  uninstall: (addonId) => invoke('tauri_addons_uninstall', { addonId }),
  setEnabled: (addonId, enabled) => invoke('tauri_addons_set_enabled', { addonId, enabled }),
  readEntry: (addonId) => invoke('tauri_addons_read_entry', { addonId }),
  listNotes: (addonId, prefix) => invoke('tauri_addons_notes_list', { addonId, prefix }),
  readNote: async (addonId, path) => {
    const document = await invoke('tauri_addons_notes_read', { addonId, path })
    return Object.freeze({ ...document, content: document?.markdown ?? '' })
  },
  writeNote: async (addonId, path, content, overwrite = true) => {
    const result = await invoke('tauri_addons_notes_write', {
      addonId,
      path,
      markdown: String(content ?? ''),
      overwrite: overwrite !== false
    })
    return Object.freeze({ ok: true, ...result })
  },
  httpRequest: (addonId, params = {}) => invoke('tauri_addons_http_request', { addonId, params }),
  call: (addonId, method, params = {}) => invoke('tauri_addons_call', { addonId, method, params }),
  getCommunityEnabled: async () => await invoke('tauri_prefs_get', { key: COMMUNITY_ADDONS_PREF_KEY }) === true
})

const safeString = (value, fallback = '') => typeof value === 'string' ? value.trim() || fallback : fallback
const asError = (value, fallback = 'External addon operation failed') => {
  if (value instanceof Error) return value
  if (value && typeof value === 'object' && typeof value.message === 'string') {
    const error = new Error(value.message)
    error.name = value.name || 'Error'
    if (value.stack) error.stack = value.stack
    return error
  }
  return new Error(typeof value === 'string' && value ? value : fallback)
}

const isOfficialRecord = (record = {}) => (
  record.source === 'official' ||
  record.official === true ||
  record.manifest?.source === 'official' ||
  record.manifest?.official === true
)

const dependencyIds = (manifest = {}) => {
  const ids = new Set(Object.keys(manifest.requires || {}))
  if (manifest.parentAddonId) ids.add(manifest.parentAddonId)
  ids.delete(manifest.id)
  return [...ids]
}

export const reconcileOfficialAddonRecords = (records = [], catalogue = []) => {
  const officialIds = new Set(
    catalogue
      .filter((item) => item?.official === true && safeString(item?.id))
      .map((item) => safeString(item.id))
  )
  return records.map((record) => {
    const id = safeString(record?.manifest?.id)
    if (!officialIds.has(id)) return record
    return {
      ...record,
      source: 'official',
      official: true,
      manifest: {
        ...record.manifest,
        source: 'official',
        official: true
      }
    }
  })
}

export const isMissingNativeServiceError = (error) => {
  const message = safeString(error?.message || error)
  return message.includes('Addon service executable is unavailable') ||
    message.includes('Addon sidecar executable is unavailable')
}

const runtimeManifest = (record = {}) => ({
  ...record.manifest,
  source: 'external',
  official: isOfficialRecord(record),
  packageHash: record.packageHash || record.package_hash || record.manifest?.packageHash || '',
  installedAt: record.installedAt || record.installed_at || record.manifest?.installedAt || '',
  defaultEnabled: false
})

const trustedManifest = (manifest = {}) => isTrustedExternalManifest({ ...manifest, source: 'external' })

class IsolatedAddonSession {
  constructor(record, logger) {
    this.record = record
    this.addonId = record.manifest.id
    this.logger = logger
    this.worker = null
    this.workerUrl = ''
    this.context = null
    this.pending = new Map()
    this.nextRequestId = 1
    this.contributionDisposers = new Map()
    this.disposed = false
  }

  async start(context) {
    if (this.worker) return
    this.context = context
    const entry = await externalAddonApi.readEntry(this.addonId)
    const source = safeString(entry?.source)
    if (!source) throw new Error(`External addon ${this.addonId} has an empty entry file`)
    const blob = new Blob([createIsolatedAddonWorkerSource(source, this.addonId, this.record.manifest)], { type: 'text/javascript' })
    this.workerUrl = URL.createObjectURL(blob)
    this.worker = new Worker(this.workerUrl, { name: `elephant-addon:${this.addonId}` })
    this.worker.addEventListener('message', (event) => this.handleMessage(event?.data || {}))
    this.worker.addEventListener('error', (event) => {
      const error = new Error(event?.message || `External addon worker crashed: ${this.addonId}`)
      this.rejectAll(error)
      this.logger?.error?.('external addon worker crashed', { id: this.addonId, error: error.message })
    })
    await this.request('activate', {}, 10_000)
  }

  callBroker(method, params = {}) {
    if (method === 'notes.list') return externalAddonApi.listNotes(this.addonId, safeString(params?.prefix))
    if (method === 'notes.read') return externalAddonApi.readNote(this.addonId, safeString(params?.path))
    if (method === 'notes.write') {
      const content = params?.markdown ?? params?.content ?? ''
      return externalAddonApi.writeNote(this.addonId, safeString(params?.path), content, params?.overwrite !== false)
    }
    if (method === 'http.request') return externalAddonApi.httpRequest(this.addonId, params)
    return externalAddonApi.call(this.addonId, method, params)
  }

  handleMessage(message) {
    if (message.type === 'rpc') {
      void this.callBroker(message.method, message.params || {})
        .then((result) => this.post({ type: 'rpc-result', id: message.id, ok: true, result }))
        .catch((error) => this.post({ type: 'rpc-result', id: message.id, ok: false, error: { message: error?.message || String(error) } }))
      return
    }
    if (message.type === 'log') return this.logFromWorker(message)
    if (message.type === 'register-action') return this.registerAction(message.action)
    if (message.type === 'unregister-action') return this.removeContribution(`action:${safeString(message.id)}`)
    if (message.type === 'register-view') return this.registerView(message.view)
    if (message.type === 'unregister-view') return this.removeContribution(`view:${safeString(message.id)}`)
    if (!message.type?.endsWith('-result')) return
    const pending = this.pending.get(message.id)
    if (!pending) return
    clearTimeout(pending.timeout)
    this.pending.delete(message.id)
    if (message.ok) pending.resolve(message.result)
    else pending.reject(asError(message.error))
  }

  logFromWorker(message = {}) {
    const level = ['debug', 'info', 'warn', 'error'].includes(message.level) ? message.level : 'info'
    const args = Array.isArray(message.args) ? message.args : []
    const logger = this.logger?.[level] || this.logger?.info
    logger?.call(this.logger, `[addon:${this.addonId}]`, ...args)
  }

  removeContribution(key) {
    const dispose = this.contributionDisposers.get(key)
    if (!dispose) return false
    this.contributionDisposers.delete(key)
    dispose()
    return true
  }

  storeContribution(key, dispose) {
    this.removeContribution(key)
    let active = true
    const remove = () => {
      if (!active) return
      active = false
      dispose?.()
    }
    this.contributionDisposers.set(key, remove)
    return remove
  }

  registerAction(action = {}) {
    if (!this.record.manifest.permissions?.commands) return
    const id = safeString(action.id)
    if (!id.startsWith(`${this.addonId}.`)) return
    const dispose = this.context.addAction({
      id,
      title: safeString(action.title, id),
      description: safeString(action.description),
      order: Number.isFinite(action.order) ? action.order : 0,
      run: (payload) => this.request('run-command', { commandId: id, payload }, 60_000)
    })
    this.storeContribution(`action:${id}`, dispose)
  }

  registerView(view = {}) {
    const declaredViews = Array.isArray(this.record.manifest.contributes?.views) ? this.record.manifest.contributes.views : []
    const id = safeString(view.id)
    const declared = declaredViews.some((entry) => entry?.id === id && entry?.kind === view.kind)
    if (!declared || !id.startsWith(`${this.addonId}.`)) return
    const dispose = this.context.addView({
      id,
      title: safeString(view.title, id),
      description: safeString(view.description),
      icon: safeString(view.icon, 'list-todo'),
      kind: safeString(view.kind),
      order: Number.isFinite(view.order) ? view.order : 0,
      getState: (params) => this.request('view-state', { viewId: id, params }, 60_000),
      dispatch: (action, params) => this.request('view-action', { viewId: id, action, params }, 60_000)
    })
    this.storeContribution(`view:${id}`, dispose)
  }

  request(type, payload = {}, timeoutMs = 10_000) {
    if (!this.worker || this.disposed) return Promise.reject(new Error(`External addon worker is not running: ${this.addonId}`))
    const id = this.nextRequestId++
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(id)
        this.dispose()
        reject(new Error(`External addon ${this.addonId} timed out during ${type}`))
      }, timeoutMs)
      this.pending.set(id, { resolve, reject, timeout })
      this.post({ type, id, ...payload })
    })
  }

  post(message) { this.worker?.postMessage(message) }
  rejectAll(error) {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timeout)
      pending.reject(error)
    }
    this.pending.clear()
  }
  async stop() {
    if (!this.worker || this.disposed) return
    try { await this.request('deactivate', {}, 3_000) }
    catch (error) { this.logger?.warn?.('external addon deactivation failed', { id: this.addonId, error: error?.message || String(error) }) }
    finally { this.dispose() }
  }
  dispose() {
    if (this.disposed) return
    this.disposed = true
    this.rejectAll(new Error(`External addon worker stopped: ${this.addonId}`))
    for (const key of [...this.contributionDisposers.keys()]) this.removeContribution(key)
    this.worker?.terminate()
    this.worker = null
    if (this.workerUrl) URL.revokeObjectURL(this.workerUrl)
    this.workerUrl = ''
  }
}

const createIsolatedAddonDefinition = (record, logger) => {
  let session = null
  const official = isOfficialRecord(record)
  const manifest = runtimeManifest(record)
  return {
    manifest,
    async activate(context) {
      if (!official && !await externalAddonApi.getCommunityEnabled()) {
        throw new Error('Community addons are disabled. Turn them on in Settings → Addons first.')
      }
      session = new IsolatedAddonSession({ ...record, manifest }, logger)
      try {
        await session.start(context)
        await externalAddonApi.setEnabled(manifest.id, true)
      } catch (error) {
        session.dispose()
        session = null
        await externalAddonApi.setEnabled(manifest.id, false).catch(() => {})
        throw error
      }
      return () => session?.dispose()
    },
    async deactivate() {
      await session?.stop()
      session = null
      await externalAddonApi.setEnabled(manifest.id, false)
    }
  }
}

const createExternalAddonDefinition = (record, logger) => {
  const manifest = runtimeManifest(record)
  const normalizedRecord = { ...record, manifest }
  return trustedManifest(manifest)
    ? createTrustedAddonDefinition(normalizedRecord, logger)
    : createIsolatedAddonDefinition(normalizedRecord, logger)
}

export class ExternalAddonController {
  constructor(manager, options = {}) {
    this.manager = manager
    this.logger = options.logger
    this.records = new Map()
    this.loaded = false
  }

  async load() {
    if (this.loaded) return [...this.records.values()]
    const installedRecords = await externalAddonApi.list()
    let records = installedRecords
    try {
      records = reconcileOfficialAddonRecords(installedRecords, await externalAddonApi.officialList())
    } catch (error) {
      this.logger?.warn?.('official addon provenance reconciliation failed', {
        error: error?.message || String(error)
      })
    }
    const communityEnabled = await externalAddonApi.getCommunityEnabled()
    const safeMode = await getTrustedSafeMode()
    for (const record of records) {
      if (!this.manager.get(record.manifest.id)) this.register(record)
      this.records.set(record.manifest.id, record)
    }
    const enabledRecords = new Map(records.filter((record) => record.enabled).map((record) => [record.manifest.id, record]))
    const enabling = new Set()
    const enableWithDependencies = async (record) => {
      if (!record?.enabled || this.manager.get(record.manifest.id)?.enabled) return
      if (enabling.has(record.manifest.id)) throw new Error(`Circular addon dependency: ${record.manifest.id}`)
      enabling.add(record.manifest.id)
      for (const dependencyId of dependencyIds(record.manifest)) {
        await enableWithDependencies(enabledRecords.get(dependencyId))
      }
      enabling.delete(record.manifest.id)
      await this.manager.enable(record.manifest.id)
    }
    for (const record of records) {
      if (!record.enabled) continue
      const official = isOfficialRecord(record)
      const trusted = trustedManifest(runtimeManifest(record))
      if ((!official && !communityEnabled) || (!official && safeMode && trusted)) {
        await externalAddonApi.setEnabled(record.manifest.id, false).catch(() => {})
        if (safeMode && trusted) this.logger?.warn?.('trusted addon disabled by safe mode', { id: record.manifest.id })
        continue
      }
      try {
        await enableWithDependencies(record)
      } catch (error) {
        if (official && isMissingNativeServiceError(error)) {
          try {
            await this.repairOfficialPackage(record)
            continue
          } catch (repairError) {
            this.logger?.error?.('official addon native package repair failed', {
              id: record.manifest.id,
              error: repairError?.message || String(repairError)
            })
          }
        }
        this.logger?.error?.('external addon startup failed', { id: record.manifest.id, error: error?.message || String(error) })
      }
    }
    this.loaded = true
    return records
  }

  async reload() {
    this.loaded = false
    return this.load()
  }

  async repairOfficialPackage(record) {
    const addonId = record.manifest.id
    const repaired = await externalAddonApi.officialInstall(addonId)
    if (this.manager.get(addonId)) {
      await this.manager.disable(addonId).catch(() => {})
      this.manager.unregister(addonId)
    }
    const [normalized] = reconcileOfficialAddonRecords([repaired], [{ id: addonId, official: true }])
    this.records.set(addonId, normalized)
    this.register(normalized)
    await this.manager.enable(addonId)
    this.logger?.info?.('official addon native package repaired', { id: addonId })
    return normalized
  }

  register(record) {
    const id = record?.manifest?.id
    if (!id) throw new Error('External addon record has no id')
    if (this.manager.get(id)) return this.manager.get(id)
    this.records.set(id, record)
    return this.manager.register(createExternalAddonDefinition(record, this.logger))
  }

  getRecord(addonId) { return this.records.get(addonId) || null }
  isOfficial(addonId) { return isOfficialRecord(this.getRecord(addonId) || {}) }
  isTrusted(addonId) {
    const record = this.getRecord(addonId)
    return Boolean(record && trustedManifest(runtimeManifest(record)))
  }
  getTrustState(addonId) {
    const record = this.getRecord(addonId)
    if (!record) throw new Error(`Unknown external addon: ${addonId}`)
    if (isOfficialRecord(record)) {
      const packageHash = safeString(record.packageHash || record.package_hash)
      return Promise.resolve({ approved: true, approvedHash: packageHash, packageHash, official: true })
    }
    return getTrustedApproval(record)
  }
  approveTrusted(addonId) {
    const record = this.getRecord(addonId)
    if (!record || !this.isTrusted(addonId)) throw new Error(`Addon does not request full app access: ${addonId}`)
    if (isOfficialRecord(record)) return this.getTrustState(addonId)
    return approveTrustedAddon(record)
  }
  revokeTrusted(addonId) {
    const record = this.getRecord(addonId)
    if (!record) throw new Error(`Unknown external addon: ${addonId}`)
    if (isOfficialRecord(record)) throw new Error('Official addon trust is tied to the verified catalogue package')
    return revokeTrustedAddon(record)
  }
  getSafeMode() { return getTrustedSafeMode() }
  setSafeMode(enabled) { return setTrustedSafeMode(enabled) }

  async installFromPath(packagePath) {
    const record = await externalAddonApi.install(packagePath)
    if (this.manager.get(record.manifest.id)) {
      await this.manager.disable(record.manifest.id).catch(() => {})
      this.manager.unregister(record.manifest.id)
    }
    this.records.set(record.manifest.id, record)
    this.register(record)
    return record
  }

  async uninstall(addonId) {
    const snapshot = this.manager.get(addonId)
    if (snapshot?.enabled || snapshot?.status === 'error') await this.manager.disable(addonId).catch(() => {})
    await externalAddonApi.uninstall(addonId)
    this.manager.unregister(addonId)
    this.records.delete(addonId)
  }
}

export const installExternalAddonRuntime = (manager, options = {}) => {
  const controller = new ExternalAddonController(manager, options)
  manager.external = controller
  const windowObject = options.window || manager.host?.get?.('window') || globalThis.window
  const load = async() => controller.load().catch((error) => {
    const message = error?.message || String(error)
    if (message === 'No active ElephantNote vault.') {
      manager.logger.warn('external addon registry deferred until a vault is active', { error: message })
      return false
    }
    manager.logger.error('external addon registry failed to load', error)
    return false
  })
  controller.retry = load
  const retryOnVaultActivation = (event) => {
    if (controller.loaded || !event?.detail?.activeVaultPath) return
    manager.logger.info('external addon registry retrying after vault activation')
    void load()
  }
  windowObject?.addEventListener?.('elephantnote:vault-active-changed', retryOnVaultActivation)
  void load()
  return controller
}
