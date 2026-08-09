import { ADDON_EXTENSION_POINTS } from './extensionPoints'
import { ADDON_STATUS, assertAddonDefinition } from './manifest'

const createDefaultLogger = () => ({
  info: (...args) => console.info('[addons]', ...args),
  warn: (...args) => console.warn('[addons]', ...args),
  error: (...args) => console.error('[addons]', ...args)
})

const normalizeError = (error) => {
  if (!error) return null
  return {
    name: error.name || 'Error',
    message: error.message || String(error),
    stack: error.stack || ''
  }
}

const createSnapshot = (record) => ({
  manifest: record.manifest,
  enabled: record.enabled,
  status: record.status,
  error: record.error
})

const requireString = (value, label) => {
  if (typeof value !== 'string' || !value.trim()) {
    throw new TypeError(`${label} must be a non-empty string`)
  }
  return value.trim()
}

const getContributionId = (entry) => {
  const id = entry?.contribution?.id
  return typeof id === 'string' ? id.trim() : ''
}

const dependencyIds = (manifest = {}) => {
  const ids = new Set(Object.keys(manifest.requires || {}))
  if (manifest.parentAddonId) ids.add(manifest.parentAddonId)
  ids.delete(manifest.id)
  return [...ids]
}

export class ElephantAddonManager {
  constructor(context = {}) {
    this.context = Object.freeze({ ...context, addons: this })
    this.logger = context.logger || createDefaultLogger()
    this.records = new Map()
    this.contributions = new Map()
    this.listeners = new Map()
    this.activationPromises = new Map()
  }

  register(addonDefinition) {
    const manifest = assertAddonDefinition(addonDefinition)
    if (this.records.has(manifest.id)) {
      throw new Error(`Addon already registered: ${manifest.id}`)
    }

    const record = {
      manifest,
      module: addonDefinition,
      enabled: false,
      status: ADDON_STATUS.disabled,
      error: null,
      disposables: []
    }

    this.records.set(manifest.id, record)
    this.emit('registered', createSnapshot(record))
    this.emit('changed', createSnapshot(record))
    return createSnapshot(record)
  }

  unregister(id) {
    const record = this.requireRecord(id)
    if (record.enabled || record.status === ADDON_STATUS.activating) {
      throw new Error(`Cannot unregister an active addon: ${id}`)
    }
    const dependents = this.getDependents(id)
    if (dependents.length) {
      throw new Error(`Cannot unregister ${id}; installed dependents: ${dependents.join(', ')}`)
    }
    this.disposeRecord(record)
    this.clearContributionsFromAddon(id)
    this.records.delete(id)
    const snapshot = createSnapshot(record)
    this.emit('unregistered', snapshot)
    this.emit('changed', snapshot)
    return snapshot
  }

  get(id) {
    const record = this.records.get(id)
    return record ? createSnapshot(record) : null
  }

  list() {
    return [...this.records.values()].map(createSnapshot)
  }

  getDependents(id, { enabledOnly = false } = {}) {
    return [...this.records.values()]
      .filter((candidate) => candidate.manifest.id !== id)
      .filter((candidate) => !enabledOnly || candidate.enabled)
      .filter((candidate) => dependencyIds(candidate.manifest).includes(id))
      .map((candidate) => candidate.manifest.id)
      .sort()
  }

  assertDependenciesEnabled(record) {
    const missing = []
    const disabled = []
    for (const dependencyId of dependencyIds(record.manifest)) {
      const dependency = this.records.get(dependencyId)
      if (!dependency) missing.push(dependencyId)
      else if (!dependency.enabled) disabled.push(dependencyId)
    }
    if (missing.length) {
      throw new Error(`${record.manifest.id} requires missing addon${missing.length === 1 ? '' : 's'}: ${missing.join(', ')}`)
    }
    if (disabled.length) {
      throw new Error(`${record.manifest.id} requires enabled addon${disabled.length === 1 ? '' : 's'}: ${disabled.join(', ')}`)
    }
  }

  async enable(id) {
    const record = this.requireRecord(id)
    if (record.enabled) return createSnapshot(record)
    const pending = this.activationPromises.get(id)
    if (pending) return pending
    const activation = this.enableRecord(id, record)
    this.activationPromises.set(id, activation)
    try {
      return await activation
    } finally {
      if (this.activationPromises.get(id) === activation) this.activationPromises.delete(id)
    }
  }

  async enableRecord(id, record) {
    this.assertDependenciesEnabled(record)

    record.status = ADDON_STATUS.activating
    record.error = null
    this.emit('changed', createSnapshot(record))

    try {
      const addonContext = this.createAddonContext(record)
      const dispose = await record.module.activate?.(addonContext)
      if (typeof dispose === 'function') record.disposables.push(dispose)
      record.enabled = true
      record.status = ADDON_STATUS.enabled
      this.emit('enabled', createSnapshot(record))
      this.emit('changed', createSnapshot(record))
      return createSnapshot(record)
    } catch (error) {
      this.disposeRecord(record)
      this.clearContributionsFromAddon(id)
      record.enabled = false
      record.status = ADDON_STATUS.error
      record.error = normalizeError(error)
      this.logger.error('addon activation failed', { id, error: record.error })
      this.emit('error', createSnapshot(record))
      this.emit('changed', createSnapshot(record))
      throw error
    }
  }

  async disable(id) {
    const record = this.requireRecord(id)
    if (!record.enabled && record.status !== ADDON_STATUS.error) return createSnapshot(record)
    const activeDependents = this.getDependents(id, { enabledOnly: true })
    if (activeDependents.length) {
      throw new Error(`Cannot disable ${id}; enabled dependents: ${activeDependents.join(', ')}`)
    }

    try {
      await record.module.deactivate?.(this.createAddonContext(record))
    } finally {
      this.disposeRecord(record)
      this.clearContributionsFromAddon(id)
      record.enabled = false
      record.status = ADDON_STATUS.disabled
      record.error = null
      this.emit('disabled', createSnapshot(record))
      this.emit('changed', createSnapshot(record))
    }

    return createSnapshot(record)
  }

  async enableDefaultAddons() {
    const defaults = [...this.records.values()].filter((record) => record.manifest.defaultEnabled)
    for (const record of defaults) {
      await this.enable(record.manifest.id)
    }
  }

  registerContribution(addonId, area, contribution) {
    const normalizedAddonId = requireString(addonId, 'addonId')
    const normalizedArea = requireString(area, 'area')
    const record = this.requireRecord(normalizedAddonId)

    const contributionKey = getContributionId({ contribution })
    const existing = this.getContributions(normalizedArea).find((entry) =>
      entry.addonId === normalizedAddonId &&
      contributionKey &&
      getContributionId(entry) === contributionKey
    )
    if (existing) {
      this.logger.warn('duplicate addon contribution ignored', {
        addonId: normalizedAddonId,
        area: normalizedArea,
        contributionId: contributionKey
      })
      return () => {}
    }

    if (!this.contributions.has(normalizedArea)) {
      this.contributions.set(normalizedArea, [])
    }

    const entry = Object.freeze({
      addonId: record.manifest.id,
      contribution
    })

    this.contributions.get(normalizedArea).push(entry)
    this.emit('contribution:registered', { area: normalizedArea, entry })
    this.emit('contribution:changed', this.getContributionMap())

    return () => {
      this.unregisterContribution(normalizedArea, entry)
    }
  }

  unregisterContribution(area, entry) {
    const entries = this.contributions.get(area)
    if (!entries) return
    const nextEntries = entries.filter((candidate) => candidate !== entry)
    if (nextEntries.length) {
      this.contributions.set(area, nextEntries)
    } else {
      this.contributions.delete(area)
    }
    this.emit('contribution:removed', { area, entry })
    this.emit('contribution:changed', this.getContributionMap())
  }

  getContributions(area) {
    return [...(this.contributions.get(area) || [])]
  }

  getContributionMap() {
    return Object.fromEntries(
      [...this.contributions.entries()].map(([area, entries]) => [area, [...entries]])
    )
  }

  getActions() {
    return this.getContributions(ADDON_EXTENSION_POINTS.actions)
      .filter((entry) => getContributionId(entry))
  }

  getAction(actionId) {
    const normalizedActionId = requireString(actionId, 'actionId')
    return this.getActions().find((entry) => getContributionId(entry) === normalizedActionId) || null
  }

  async runAction(actionId, payload = undefined) {
    const entry = this.getAction(actionId)
    if (!entry) throw new Error(`Unknown addon action: ${actionId}`)

    const run = entry.contribution?.run
    if (typeof run !== 'function') {
      throw new TypeError(`Addon action is not executable: ${actionId}`)
    }

    return await run(payload, {
      addonId: entry.addonId,
      actionId,
      addons: this
    })
  }

  on(eventName, listener) {
    const normalizedEventName = requireString(eventName, 'eventName')
    if (typeof listener !== 'function') {
      throw new TypeError('listener must be a function')
    }

    if (!this.listeners.has(normalizedEventName)) {
      this.listeners.set(normalizedEventName, new Set())
    }

    this.listeners.get(normalizedEventName).add(listener)
    return () => this.listeners.get(normalizedEventName)?.delete(listener)
  }

  createAddonContext(record) {
    const register = (area, contribution) => {
      const dispose = this.registerContribution(record.manifest.id, area, contribution)
      record.disposables.push(dispose)
      return dispose
    }

    return Object.freeze({
      ...this.context,
      manifest: record.manifest,
      registerContribution: register,
      addAction: (action) => register(ADDON_EXTENSION_POINTS.actions, action),
      addSidebarItem: (item) => register(ADDON_EXTENSION_POINTS.sidebarItems, item),
      addSettingsSection: (section) => register(ADDON_EXTENSION_POINTS.settingsSections, section),
      addView: (view) => register(ADDON_EXTENSION_POINTS.views, view),
      addEditorExtension: (extension) => register(ADDON_EXTENSION_POINTS.editorExtensions, extension),
      addStatusBarItem: (item) => register(ADDON_EXTENSION_POINTS.statusBarItems, item),
      addDisposable: (dispose) => {
        if (typeof dispose !== 'function') throw new TypeError('dispose must be a function')
        record.disposables.push(dispose)
        return dispose
      }
    })
  }

  clearContributionsFromAddon(addonId) {
    let changed = false
    for (const [area, entries] of this.contributions.entries()) {
      const nextEntries = entries.filter((entry) => entry.addonId !== addonId)
      if (nextEntries.length !== entries.length) changed = true
      if (nextEntries.length) {
        this.contributions.set(area, nextEntries)
      } else {
        this.contributions.delete(area)
      }
    }

    if (changed) {
      this.emit('contribution:changed', this.getContributionMap())
    }
  }

  disposeRecord(record) {
    while (record.disposables.length) {
      const dispose = record.disposables.pop()
      try {
        dispose()
      } catch (error) {
        this.logger.warn('addon cleanup failed', {
          id: record.manifest.id,
          error: normalizeError(error)
        })
      }
    }
  }

  requireRecord(id) {
    const record = this.records.get(id)
    if (!record) throw new Error(`Unknown addon: ${id}`)
    return record
  }

  emit(eventName, payload) {
    for (const listener of this.listeners.get(eventName) || []) {
      try {
        listener(payload)
      } catch (error) {
        this.logger.warn('addon listener failed', { eventName, error: normalizeError(error) })
      }
    }
  }
}
