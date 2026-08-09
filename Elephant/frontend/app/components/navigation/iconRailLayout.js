export const CORE_ICON_RAIL_ITEMS = Object.freeze([
  { id: 'vault', label: 'Vault', description: 'Open the active vault switcher.' },
  { id: 'sidebar-toggle', label: 'Sidebar', description: 'Show or hide the navigation sidebar.' },
  { id: 'search', label: 'Search', description: 'Open global search.' }
])

export const DEFAULT_ICON_RAIL_ORDER = Object.freeze(
  CORE_ICON_RAIL_ITEMS.map((item) => item.id)
)

export const ICON_RAIL_SEPARATOR_PREFIX = 'separator:'
const LEADING_CORE_ICON_RAIL_IDS = Object.freeze(['vault', 'sidebar-toggle'])

const normalizeIds = (values) => {
  if (!Array.isArray(values)) return []
  const result = []
  const seen = new Set()
  for (const value of values) {
    const id = typeof value === 'string' ? value.trim() : ''
    if (!id || seen.has(id)) continue
    seen.add(id)
    result.push(id)
  }
  return result
}

export const pushIconRailLog = (event, details = {}) => {
  const message = `[icon-rail] ${event}`
  const entry = {
    at: new Date().toISOString(),
    level: 'info',
    message,
    details
  }
  const target = typeof window !== 'undefined' ? window : globalThis
  target.__ELEPHANT_DEBUG_LOGS__ = Array.isArray(target.__ELEPHANT_DEBUG_LOGS__)
    ? target.__ELEPHANT_DEBUG_LOGS__
    : []
  target.__ELEPHANT_DEBUG_LOGS__.push(entry)
  if (target.__ELEPHANT_DEBUG_LOGS__.length > 1000) {
    target.__ELEPHANT_DEBUG_LOGS__.splice(0, target.__ELEPHANT_DEBUG_LOGS__.length - 1000)
  }
  console.info(message, details)
  return entry
}

export const addonViewRailId = (viewId) => `addon-view:${String(viewId || '').trim()}`
export const isIconRailSeparatorId = (id) => String(id || '').startsWith(ICON_RAIL_SEPARATOR_PREFIX)
export const createIconRailSeparatorId = () => `${ICON_RAIL_SEPARATOR_PREFIX}${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`

export const extendIconRailOrder = (order, availableIds) => {
  const extended = normalizeIds(order)
  const seen = new Set(extended)
  for (const id of normalizeIds(availableIds)) {
    if (seen.has(id)) continue
    seen.add(id)
    extended.push(id)
  }
  return extended
}

export const normalizeIconRailOrder = (order, availableIds) => {
  const available = normalizeIds(availableIds)
  const allowed = new Set(available)
  const normalized = normalizeIds(order).filter((id) => allowed.has(id) || isIconRailSeparatorId(id))
  const seen = new Set(normalized)

  for (const id of [...LEADING_CORE_ICON_RAIL_IDS].reverse()) {
    if (allowed.has(id) && !seen.has(id)) {
      normalized.unshift(id)
      seen.add(id)
    }
  }

  for (const id of available) {
    if (!seen.has(id)) normalized.push(id)
  }
  return normalized
}

export const normalizeIconRailHidden = (hidden, availableIds) => {
  const allowed = new Set(normalizeIds(availableIds))
  return normalizeIds(hidden).filter((id) => allowed.has(id) && !isIconRailSeparatorId(id))
}

export const moveIconRailItem = (order, id, targetIndex) => {
  const normalized = normalizeIds(order)
  const currentIndex = normalized.indexOf(id)
  if (currentIndex === -1) return normalized
  const boundedIndex = Math.max(0, Math.min(normalized.length - 1, Number(targetIndex) || 0))
  if (boundedIndex === currentIndex) return normalized
  const next = [...normalized]
  next.splice(currentIndex, 1)
  next.splice(boundedIndex, 0, id)
  return next
}
