export const ADDON_API_VERSION = 1
export const SUPPORTED_ADDON_API_VERSIONS = Object.freeze([1])

export const ADDON_ACCESS_LEVEL = Object.freeze({
  isolated: 'isolated',
  trusted: 'trusted',
  system: 'system'
})

export const ADDON_STATUS = Object.freeze({
  disabled: 'disabled',
  enabled: 'enabled',
  activating: 'activating',
  error: 'error'
})

const ADDON_ID_RE = /^[a-z0-9][a-z0-9._-]*$/
const CONTENT_KINDS = new Set(['image', 'embed', 'block'])
const DISABLED_PRESENTATIONS = new Set(['static-preview', 'placeholder', 'hidden'])

const normalizeString = (value, fallback = '') => {
  if (typeof value !== 'string') return fallback
  const normalized = value.trim()
  return normalized || fallback
}

const normalizeStringArray = (value) => {
  if (!Array.isArray(value)) return []
  return value
    .filter((entry) => typeof entry === 'string')
    .map((entry) => entry.trim())
    .filter(Boolean)
}

const normalizeObject = (value) => {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return {}
  return { ...value }
}

const deepFreezeObject = (value) => {
  const object = normalizeObject(value)
  for (const [key, entry] of Object.entries(object)) {
    if (Array.isArray(entry)) object[key] = Object.freeze([...entry])
    else if (entry && typeof entry === 'object') object[key] = deepFreezeObject(entry)
  }
  return Object.freeze(object)
}

const normalizeRequires = (value) => {
  const result = {}
  for (const [id, requirement] of Object.entries(normalizeObject(value))) {
    const normalizedId = normalizeString(id)
    const normalizedRequirement = normalizeString(requirement)
    if (!ADDON_ID_RE.test(normalizedId) || !normalizedRequirement) continue
    result[normalizedId] = normalizedRequirement
  }
  return Object.freeze(result)
}

const normalizeContentTypes = (value) => {
  if (!Array.isArray(value)) return Object.freeze([])
  const ids = new Set()
  const result = []
  for (const raw of value) {
    const entry = normalizeObject(raw)
    const id = normalizeString(entry.id)
    const kind = normalizeString(entry.kind, 'embed').toLowerCase()
    const sourcePattern = normalizeString(entry.sourcePattern)
    if (!id || ids.has(id) || !CONTENT_KINDS.has(kind) || !sourcePattern) continue
    ids.add(id)
    const disabledPresentation = normalizeString(entry.disabledPresentation, 'placeholder').toLowerCase()
    result.push(Object.freeze({
      id,
      kind,
      sourcePattern,
      disabledPresentation: DISABLED_PRESENTATIONS.has(disabledPresentation) ? disabledPresentation : 'placeholder',
      disabledLabel: normalizeString(entry.disabledLabel, id)
    }))
  }
  return Object.freeze(result)
}

const normalizePermissions = (value) => {
  if (value == null || typeof value !== 'object') return Object.freeze([])
  if (Array.isArray(value)) return Object.freeze(normalizeStringArray(value))
  const permissions = normalizeObject(value)
  const notes = normalizeObject(permissions.notes)
  const network = normalizeObject(permissions.network)
  return Object.freeze({
    notes: Object.freeze({
      read: Object.freeze(normalizeStringArray(notes.read)),
      write: Object.freeze(normalizeStringArray(notes.write))
    }),
    network: Object.freeze({
      hosts: Object.freeze(normalizeStringArray(network.hosts))
    }),
    storage: permissions.storage === true,
    commands: permissions.commands === true,
    views: permissions.views === true,
    native: permissions.native === true
  })
}

const normalizeAccessLevel = (manifest, runtime, contributes) => {
  if (normalizeString(manifest.source, 'builtin') === 'builtin') return ADDON_ACCESS_LEVEL.system
  const declared = normalizeString(
    runtime.mode || contributes.runtimeMode || contributes.security?.access,
    ADDON_ACCESS_LEVEL.isolated
  ).toLowerCase()
  if (['trusted', 'full', 'full-app', 'full-app-access'].includes(declared)) {
    return ADDON_ACCESS_LEVEL.trusted
  }
  return ADDON_ACCESS_LEVEL.isolated
}

const normalizeRuntime = (value, manifest = {}, contributes = {}) => {
  const runtime = normalizeObject(value)
  const type = normalizeString(runtime.type)
  const entry = normalizeString(runtime.entry)
  if (!type && !entry) return Object.freeze({})

  const normalized = { type, entry }
  const accessLevel = normalizeAccessLevel(manifest, runtime, contributes)
  if (accessLevel !== ADDON_ACCESS_LEVEL.isolated) normalized.mode = accessLevel
  if (runtime.desktopOnly === true) normalized.desktopOnly = true
  return Object.freeze(normalized)
}

export const getAddonAccessLevel = (manifest = {}) => {
  if (manifest?.runtime?.mode) return manifest.runtime.mode
  const source = normalizeString(manifest.source, 'builtin')
  if (source === 'builtin') return ADDON_ACCESS_LEVEL.system
  const contributes = normalizeObject(manifest.contributes)
  return normalizeAccessLevel(manifest, normalizeObject(manifest.runtime), contributes)
}

export const isTrustedAddonManifest = (manifest = {}) => {
  return getAddonAccessLevel(manifest) === ADDON_ACCESS_LEVEL.trusted
}

export const normalizeAddonManifest = (manifest = {}) => {
  if (!manifest || typeof manifest !== 'object' || Array.isArray(manifest)) {
    throw new TypeError('Addon manifest must be an object')
  }

  const id = normalizeString(manifest.id)
  if (!ADDON_ID_RE.test(id)) {
    throw new TypeError('Addon manifest id must use lowercase letters, numbers, dots, dashes or underscores')
  }

  const apiVersion = Number.isInteger(manifest.apiVersion)
    ? manifest.apiVersion
    : ADDON_API_VERSION

  if (!SUPPORTED_ADDON_API_VERSIONS.includes(apiVersion)) {
    throw new Error(`Unsupported addon apiVersion ${apiVersion}`)
  }

  const name = normalizeString(manifest.name, id)
  const version = normalizeString(manifest.version, '0.0.0')
  const contributes = deepFreezeObject(manifest.contributes)
  const declaredContentTypes = Array.isArray(manifest.contentTypes)
    ? manifest.contentTypes
    : contributes.contentTypes

  return Object.freeze({
    id,
    parentAddonId: normalizeString(manifest.parentAddonId),
    requires: normalizeRequires(manifest.requires),
    name,
    version,
    description: normalizeString(manifest.description),
    author: normalizeString(manifest.author),
    icon: normalizeString(manifest.icon, 'package'),
    apiVersion,
    minAppVersion: normalizeString(manifest.minAppVersion),
    permissions: normalizePermissions(manifest.permissions),
    contributes,
    contentTypes: normalizeContentTypes(declaredContentTypes),
    activationEvents: Object.freeze(normalizeStringArray(manifest.activationEvents)),
    runtime: normalizeRuntime(manifest.runtime, manifest, contributes),
    native: deepFreezeObject(manifest.native),
    platforms: Object.freeze(normalizeStringArray(manifest.platforms)),
    source: normalizeString(manifest.source, 'builtin'),
    official: manifest.official === true,
    packageHash: normalizeString(manifest.packageHash),
    installedAt: normalizeString(manifest.installedAt),
    defaultEnabled: manifest.defaultEnabled === true,
    removable: manifest.removable !== false
  })
}

export const assertAddonDefinition = (addonDefinition) => {
  if (!addonDefinition || typeof addonDefinition !== 'object' || Array.isArray(addonDefinition)) {
    throw new TypeError('Addon definition must be an object')
  }

  return normalizeAddonManifest(addonDefinition.manifest)
}
