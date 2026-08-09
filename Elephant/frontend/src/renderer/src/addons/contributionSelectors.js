import { ADDON_EXTENSION_POINTS } from './extensionPoints'

const normalizeText = (value, fallback = '') => {
  if (typeof value !== 'string') return fallback
  const normalized = value.trim()
  return normalized || fallback
}

const normalizeEntries = (contributionMap = {}, area) => {
  const entries = contributionMap?.[area]
  return Array.isArray(entries) ? entries : []
}

const normalizeContributionEntry = (entry) => {
  if (!entry || typeof entry !== 'object') return null
  const addonId = normalizeText(entry.addonId)
  const contribution = entry.contribution
  if (!addonId || !contribution || typeof contribution !== 'object') return null
  return { addonId, contribution }
}

export const getAddonContributions = (contributionMap = {}, area) => {
  return normalizeEntries(contributionMap, area)
    .map(normalizeContributionEntry)
    .filter(Boolean)
}

export const getAddonActions = (contributionMap = {}) => {
  return getAddonContributions(contributionMap, ADDON_EXTENSION_POINTS.actions)
    .map((entry) => {
      const id = normalizeText(entry.contribution.id)
      if (!id) return null
      return {
        addonId: entry.addonId,
        id,
        title: normalizeText(entry.contribution.title, id),
        description: normalizeText(entry.contribution.description),
        enabled: entry.contribution.enabled !== false,
        run: entry.contribution.run
      }
    })
    .filter(Boolean)
    .sort((a, b) => a.title.localeCompare(b.title))
}

export const getAddonSidebarItems = (contributionMap = {}) => {
  return getAddonContributions(contributionMap, ADDON_EXTENSION_POINTS.sidebarItems)
    .map((entry) => {
      const id = normalizeText(entry.contribution.id)
      if (!id) return null
      return {
        addonId: entry.addonId,
        id,
        title: normalizeText(entry.contribution.title, id),
        tooltip: normalizeText(entry.contribution.tooltip, normalizeText(entry.contribution.title, id)),
        icon: normalizeText(entry.contribution.icon, 'star'),
        actionId: normalizeText(entry.contribution.actionId),
        view: normalizeText(entry.contribution.view),
        order: Number.isFinite(entry.contribution.order) ? entry.contribution.order : 1000
      }
    })
    .filter(Boolean)
    .sort((a, b) => a.order - b.order || a.title.localeCompare(b.title))
}

export const getAddonSettingsSections = (contributionMap = {}) => {
  return getAddonContributions(contributionMap, ADDON_EXTENSION_POINTS.settingsSections)
    .map((entry) => {
      const id = normalizeText(entry.contribution.id)
      if (!id) return null
      return {
        addonId: entry.addonId,
        id,
        title: normalizeText(entry.contribution.title, id),
        description: normalizeText(entry.contribution.description),
        order: Number.isFinite(entry.contribution.order) ? entry.contribution.order : 1000
      }
    })
    .filter(Boolean)
    .sort((a, b) => a.order - b.order || a.title.localeCompare(b.title))
}
