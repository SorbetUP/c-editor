import { isTrustedAddonManifest } from '../manifest'
import AddonPacksSettings from './ui/AddonPacksSettings.vue'
import { mountSettingsComponent } from './settingsComponentHost'
import { invokeTauri, logAction, readNote, writeNote } from './shared'
import './ui/addonPacksFeedback.css'

const CORE_FEATURE_ID = 'core.addon-packs'
const LEGACY_ADDON_ID = 'elephant.addon-packs'
const CORE_CAPABILITY_IDS = new Set(['elephant.excalidraw', LEGACY_ADDON_ID])
const PACK_DIRECTORY = '.elephantnote/addons/packs'
const DEFAULT_PACK_PATH = `${PACK_DIRECTORY}/default.enaddonpack`
const BASE_PACK_PATH = `${PACK_DIRECTORY}/base.enaddonpack`
const DEVELOP_PARITY_PACK_PATH = `${PACK_DIRECTORY}/develop-parity.enaddonpack`
const REPORT_PATH = 'Reports/Addon Pack.md'
const PACK_FORMAT = 'elephantnote-addon-pack'
const PACK_VERSION = 1
const MAX_PACK_ADDONS = 200

const DEVELOP_PARITY_ADDONS = Object.freeze([
  { id: 'elephant.ai', version: '2.1.0', source: 'official', enabled: true },
  { id: 'elephant.ai-chat', version: '1.1.0', source: 'official', enabled: true },
  { id: 'elephant.ai-search', version: '1.1.0', source: 'official', enabled: true },
  { id: 'elephant.ai-ocr', version: '1.0.0', source: 'official', enabled: true },
  { id: 'elephant.wiki', version: '1.1.0', source: 'official', enabled: true },
  { id: 'elephant.graph', version: '1.1.0', source: 'official', enabled: true },
  { id: 'elephant.open-models', version: '1.2.0', source: 'official', enabled: true },
  { id: 'elephant.codex-connection', version: '1.1.0', source: 'official', enabled: true },
  { id: 'elephant.sync', version: '1.1.0', source: 'official', enabled: true },
  { id: 'elephant.calendar', version: '1.2.0', source: 'official', enabled: true },
  { id: 'elephant.sites', version: '1.1.0', source: 'official', enabled: true },
  { id: 'elephant.code-execution', version: '2.1.0', source: 'official', enabled: true },
  { id: 'elephant.google-keep-import', version: '1.1.0', source: 'official', enabled: true },
  { id: 'elephant.recently-edited', version: '1.1.0', source: 'official', enabled: true }
])

const BASE_ADDONS = Object.freeze(DEVELOP_PARITY_ADDONS
  .filter((entry) => entry.id !== 'elephant.calendar')
  .map((entry) => Object.freeze({ ...entry })))

const readCommunityEnabled = async () => {
  const value = await invokeTauri('tauri_prefs_get', { key: 'addons.communityEnabled' })
  return value === true
}

const normalizePackPath = (value = DEFAULT_PACK_PATH) => {
  const path = String(value || DEFAULT_PACK_PATH).trim().replaceAll('\\', '/')
  if (!path.startsWith(`${PACK_DIRECTORY}/`) || path.includes('/../') || path.endsWith('/..')) {
    throw new Error(`Addon packs must be stored inside ${PACK_DIRECTORY}`)
  }
  if (!path.toLowerCase().endsWith('.enaddonpack')) throw new Error('Addon pack paths must end with .enaddonpack')
  return path
}

const normalizeSource = (value, fallback = 'installed') => {
  const source = String(value || fallback).trim().toLowerCase()
  if (!['official', 'catalog', 'installed'].includes(source)) {
    if (source === 'builtin') return 'official'
    throw new Error(`Unsupported addon pack source: ${source}`)
  }
  return source
}

export const validateAddonPack = (raw, sourcePath = DEFAULT_PACK_PATH) => {
  let parsed
  try { parsed = JSON.parse(raw) } catch (error) {
    throw new Error(`Invalid JSON in ${sourcePath}: ${error.message}`)
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error('Addon pack must be a JSON object')
  if (parsed.format !== PACK_FORMAT) throw new Error(`Unsupported addon pack format: ${parsed.format || 'missing'}`)
  if (parsed.version !== PACK_VERSION) throw new Error(`Unsupported addon pack version: ${parsed.version}`)
  if (!Array.isArray(parsed.addons)) throw new Error('Addon pack must contain an addons array')
  if (parsed.addons.length > MAX_PACK_ADDONS) throw new Error(`Addon pack contains more than ${MAX_PACK_ADDONS} addons`)

  const ids = new Set()
  const addons = parsed.addons.map((entry, index) => {
    const id = typeof entry?.id === 'string' ? entry.id.trim() : ''
    if (!id) throw new Error(`Addon pack entry ${index + 1} has no id`)
    if (CORE_CAPABILITY_IDS.has(id)) return null
    if (ids.has(id)) throw new Error(`Addon pack contains duplicate id: ${id}`)
    ids.add(id)
    if (typeof entry.enabled !== 'boolean') throw new Error(`Addon pack entry ${id} must define enabled as true or false`)
    return {
      id,
      enabled: entry.enabled,
      source: normalizeSource(entry.source),
      version: typeof entry.version === 'string' ? entry.version.trim() : ''
    }
  }).filter(Boolean)

  return {
    format: PACK_FORMAT,
    version: PACK_VERSION,
    name: typeof parsed.name === 'string' && parsed.name.trim() ? parsed.name.trim() : 'Unnamed addon pack',
    description: typeof parsed.description === 'string' ? parsed.description.trim() : '',
    addons
  }
}

const protectedPack = (name, description, addons) => ({
  format: PACK_FORMAT,
  version: PACK_VERSION,
  name,
  description,
  createdAt: new Date().toISOString(),
  protected: true,
  addons: addons.map((entry) => ({ ...entry }))
})

const ensurePack = async (path, expected, name, description) => {
  try {
    const current = validateAddonPack((await readNote(path)).content, path)
    const entries = new Map(current.addons.map((entry) => [entry.id, entry]))
    const unchanged = current.addons.length === expected.length && expected.every((entry) => {
      const value = entries.get(entry.id)
      return value?.source === entry.source && value?.version === entry.version && value?.enabled === entry.enabled
    })
    if (unchanged) return { packPath: path, created: false, updated: false }
  } catch {}
  const pack = protectedPack(name, description, expected)
  await writeNote(path, `${JSON.stringify(pack, null, 2)}\n`)
  return { packPath: path, created: true, updated: true, pack }
}

const ensureFirstPartyPacks = async () => {
  const complete = await ensurePack(
    DEVELOP_PARITY_PACK_PATH,
    DEVELOP_PARITY_ADDONS,
    'Elephant Develop parity',
    'Installs and enables every first-party physical addon.'
  )
  const base = await ensurePack(
    BASE_PACK_PATH,
    BASE_ADDONS,
    'Elephant Base',
    'Installs and enables the first-party Elephant setup without Calendar.'
  )
  return { ...complete, basePackPath: base.packPath, baseCreated: base.created, baseUpdated: base.updated }
}

const catalogueMap = async () => new Map(
  (await invokeTauri('tauri_addons_catalog_list') || []).map((addon) => [addon.id, addon])
)

const createPack = async (ctx, options = {}) => {
  const path = normalizePackPath(options.path)
  const catalogue = await catalogueMap()
  const addons = ctx.addons.list()
    .filter((addon) => addon?.manifest?.id && !CORE_CAPABILITY_IDS.has(addon.manifest.id))
    .map((addon) => {
      const manifest = addon.manifest
      const official = manifest.official === true || (String(manifest.id).startsWith('elephant.') && catalogue.has(manifest.id))
      return {
        id: manifest.id,
        version: manifest.version || '',
        source: official ? 'official' : catalogue.has(manifest.id) ? 'catalog' : 'installed',
        enabled: addon.enabled === true
      }
    })
    .sort((left, right) => left.source.localeCompare(right.source) || left.id.localeCompare(right.id))
  const pack = {
    format: PACK_FORMAT,
    version: PACK_VERSION,
    name: String(options.name || 'Default Elephant pack').trim(),
    description: String(options.description || 'A portable set of official and community addons with their enabled state.').trim(),
    createdAt: new Date().toISOString(),
    addons
  }
  await writeNote(path, `${JSON.stringify(pack, null, 2)}\n`)
  return { path, count: addons.length, pack }
}

const registerCatalogRecord = async (ctx, record) => {
  const external = ctx.addons.external
  const rawManager = external?.manager || ctx.addonHost?.get?.('addonManager')
  if (!external || !rawManager) throw new Error('The physical addon runtime is unavailable')
  const current = ctx.addons.get(record.manifest.id)
  if (current) {
    if (current.enabled || current.status === 'error') await ctx.addons.disable(record.manifest.id).catch(() => {})
    rawManager.unregister(record.manifest.id)
  }
  external.register(record)
}

const setExternalEnabled = (id, enabled) => invokeTauri('tauri_addons_set_enabled', {
  addonId: id,
  enabled: enabled === true
})

const applyPack = async (ctx, options = {}) => {
  const path = normalizePackPath(options.path)
  const pack = validateAddonPack((await readNote(path)).content, path)
  const requiresCommunity = pack.addons.some((entry) => (
    entry.enabled && ['catalog', 'installed'].includes(entry.source)
  ))
  if (requiresCommunity && !await readCommunityEnabled()) {
    throw new Error('Turn on Community Addons before applying a pack that enables third-party addons')
  }

  const catalogue = await catalogueMap()
  const results = []

  for (const entry of pack.addons) {
    let snapshot = ctx.addons.get(entry.id)
    let installed = false
    let updated = false

    if (entry.source === 'official' || entry.source === 'catalog') {
      const catalogued = catalogue.get(entry.id)
      if (!catalogued) throw new Error(`Addon pack references a missing catalogue package: ${entry.id}`)
      if (entry.source === 'official' && catalogued.official !== true) {
        throw new Error(`Addon pack marks a non-official package as official: ${entry.id}`)
      }
      if (!snapshot || snapshot.manifest.version !== catalogued.version) {
        const record = await invokeTauri('tauri_addons_catalog_install', { addonId: entry.id })
        updated = Boolean(snapshot)
        installed = !snapshot
        await registerCatalogRecord(ctx, record)
        snapshot = ctx.addons.get(entry.id)
      }
    } else if (!snapshot) {
      throw new Error(`Locally installed addon is unavailable: ${entry.id}`)
    }

    if (!snapshot) throw new Error(`Unable to register addon from pack: ${entry.id}`)
    const external = snapshot.manifest.source === 'external'
    if (entry.enabled && !snapshot.enabled) {
      if (!snapshot.manifest.official && isTrustedAddonManifest(snapshot.manifest)) {
        await ctx.addons.external?.setSafeMode(false)
        await ctx.addons.external?.approveTrusted(snapshot.manifest.id)
      }
      if (external) await setExternalEnabled(entry.id, true)
      try { await ctx.addons.enable(entry.id) } catch (error) {
        if (external) await setExternalEnabled(entry.id, false).catch(() => {})
        throw error
      }
    } else if (!entry.enabled && (snapshot.enabled || snapshot.status === 'error')) {
      if (external) await setExternalEnabled(entry.id, false)
      await ctx.addons.disable(entry.id)
    }

    const current = ctx.addons.get(entry.id) || snapshot
    results.push({
      id: entry.id,
      source: entry.source,
      version: current.manifest.version || entry.version,
      enabled: current.enabled === true,
      installed,
      updated
    })
  }

  const report = [
    '---',
    'title: "Addon Pack Result"',
    'type: "generated-report"',
    'tags: [addons, pack]',
    `generatedAt: ${JSON.stringify(new Date().toISOString())}`,
    '---', '', '# Addon Pack Result', '',
    `Name: ${pack.name}`, '',
    '| Addon | Source | Version | State | Action |',
    '| --- | --- | --- | --- | --- |',
    ...results.map((result) => `| \`${result.id}\` | ${result.source} | ${result.version || 'unknown'} | ${result.enabled ? 'Enabled' : 'Disabled'} | ${result.updated ? 'Updated' : result.installed ? 'Installed' : 'Kept'} |`),
    ''
  ].join('\n')
  await writeNote(REPORT_PATH, report)
  return { path: REPORT_PATH, packPath: path, applied: results.length, results }
}

export const addonPacksCoreFeature = Object.freeze({
  id: CORE_FEATURE_ID,
  activate(ctx) {
    ctx.addAction({
      id: `${LEGACY_ADDON_ID}.ensure-develop-parity`,
      title: 'Create first-party addon packs',
      async run() {
        const result = await ensureFirstPartyPacks()
        logAction(ctx, 'addon-pack-first-party', result)
        return result
      }
    })
    ctx.addAction({
      id: `${LEGACY_ADDON_ID}.create`,
      title: 'Create addon pack from current setup',
      async run(options = {}) {
        const result = await createPack(ctx, options)
        logAction(ctx, 'addon-pack-create', result)
        return result
      }
    })
    ctx.addAction({
      id: `${LEGACY_ADDON_ID}.apply`,
      title: 'Apply addon pack',
      async run(options = {}) {
        const result = await applyPack(ctx, options)
        logAction(ctx, 'addon-pack-apply', result)
        return result
      }
    })
    ctx.addSettingsSection({
      id: `${CORE_FEATURE_ID}.settings`,
      section: 'addons',
      slot: 'addons.packs',
      chrome: false,
      title: 'Addon packs',
      description: 'Create and apply portable addon configurations.',
      order: 10,
      render: mountSettingsComponent(ctx, AddonPacksSettings)
    })
  }
})

export const addonProfilesCoreFeature = addonPacksCoreFeature
