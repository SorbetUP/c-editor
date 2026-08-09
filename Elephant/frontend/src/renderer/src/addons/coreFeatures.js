import { ADDON_EXTENSION_POINTS } from './extensionPoints'

const requireId = (value, label = 'core feature id') => {
  const id = typeof value === 'string' ? value.trim() : ''
  if (!id) throw new TypeError(`${label} must be a non-empty string`)
  return id
}

const registerCoreContribution = (manager, featureId, area, contribution, disposables) => {
  const id = requireId(featureId)
  const extensionPoint = requireId(area, 'extension point')
  if (!manager.contributions.has(extensionPoint)) manager.contributions.set(extensionPoint, [])

  const entry = Object.freeze({
    addonId: id,
    coreFeatureId: id,
    source: 'core',
    contribution
  })
  manager.contributions.get(extensionPoint).push(entry)
  manager.emit('contribution:registered', { area: extensionPoint, entry })
  manager.emit('contribution:changed', manager.getContributionMap())

  let active = true
  const dispose = () => {
    if (!active) return
    active = false
    manager.unregisterContribution(extensionPoint, entry)
  }
  disposables.push(dispose)
  return dispose
}

const createCoreContext = (manager, featureId, disposables) => {
  const register = (area, contribution) => registerCoreContribution(
    manager,
    featureId,
    area,
    contribution,
    disposables
  )

  return Object.freeze({
    ...manager.context,
    addons: manager,
    addonHost: manager.host || manager.context?.addonHost,
    coreFeatureId: featureId,
    source: 'core',
    registerContribution: register,
    addAction: (action) => register(ADDON_EXTENSION_POINTS.actions, action),
    addSidebarItem: (item) => register(ADDON_EXTENSION_POINTS.sidebarItems, item),
    addSettingsSection: (section) => register(ADDON_EXTENSION_POINTS.settingsSections, section),
    addView: (view) => register(ADDON_EXTENSION_POINTS.views, view),
    addEditorExtension: (extension) => register(ADDON_EXTENSION_POINTS.editorExtensions, extension),
    addStatusBarItem: (item) => register(ADDON_EXTENSION_POINTS.statusBarItems, item),
    addDisposable(dispose) {
      if (typeof dispose !== 'function') throw new TypeError('dispose must be a function')
      disposables.push(dispose)
      return dispose
    }
  })
}

// Core features use the same contribution bus as addons without becoming
// installable records. They are absent from the addon registry and catalogue.
export const activateCoreFeature = async (manager, definition) => {
  const id = requireId(definition?.id)
  if (!manager) throw new Error(`Cannot activate ${id} without the feature host`)
  if (!manager.coreFeatures) manager.coreFeatures = new Map()
  if (manager.coreFeatures.has(id)) return manager.coreFeatures.get(id)

  const disposables = []
  let active = true
  const handle = Object.freeze({
    id,
    source: 'core',
    dispose() {
      if (!active) return
      active = false
      while (disposables.length) {
        try { disposables.pop()?.() } catch (error) {
          manager.logger?.warn?.('[core-feature] cleanup failed', {
            id,
            error: error instanceof Error ? error.message : String(error)
          })
        }
      }
      manager.coreFeatures.delete(id)
      manager.emit('core-feature:disabled', { id })
    }
  })

  manager.coreFeatures.set(id, handle)
  try {
    const context = createCoreContext(manager, id, disposables)
    const returnedDispose = await definition.activate?.(context)
    if (typeof returnedDispose === 'function') disposables.push(returnedDispose)
    manager.emit('core-feature:enabled', { id })
    manager.logger?.info?.('[core-feature] enabled', { id })
    return handle
  } catch (error) {
    handle.dispose()
    throw error
  }
}
