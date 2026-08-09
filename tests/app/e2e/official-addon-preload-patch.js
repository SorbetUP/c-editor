'use strict'

const INSERT_AFTER_HELPERS = "const primitivePreference = (store, key, fallback = null) => Object.prototype.hasOwnProperty.call(store, key) ? store[key] : fallback\n"
const EMPTY_ADDON_CASES = `    case 'tauri_addons_list':
    case 'tauri_addons_list_full':
    case 'tauri_addons_catalog_list':
    case 'tauri_official_addons_catalog_list': return []`
const ENABLED_CASES = `    case 'tauri_addons_set_enabled':
    case 'tauri_addons_set_enabled_checked': memory.enabledAddons.set(params.addonId, params.enabled === true); return { ok: true }`
const READ_ENTRY_CASE = `    case 'tauri_addons_read_entry': throw new Error(\`No physical addon is installed in the E2E fixture: \${params.addonId || ''}\`)`

const fixtureSource = String.raw`
const crypto = require('crypto')
const projectRoot = path.resolve(__dirname, '../../..')
const officialAddonsRoot = path.join(projectRoot, 'addons')
const officialAddonStateFile = path.join(configRoot, 'official-addons-e2e-state.json')
const officialAddonRequested = String(process.env.ELEPHANT_E2E_OFFICIAL_ADDONS || '').trim()

const officialAddonFixture = (() => {
  const catalogPath = path.join(officialAddonsRoot, 'catalog.json')
  if (!fs.existsSync(catalogPath)) {
    throw new Error('Official addon E2E fixture requires pnpm addons:sync before launch')
  }
  const catalog = readJson(catalogPath, { addons: [] })
  const entries = new Map((catalog.addons || []).map((entry) => [entry.id, entry]))
  const requestedIds = officialAddonRequested === 'all'
    ? [...entries.keys()]
    : officialAddonRequested.split(',').map((value) => value.trim()).filter(Boolean)
  const persisted = readJson(officialAddonStateFile, null)
  const state = persisted && Array.isArray(persisted.installed)
    ? persisted
    : { installed: requestedIds, enabled: [], storage: {} }
  if (!state.storage || typeof state.storage !== 'object') state.storage = {}
  const storage = new Map(Object.entries(state.storage))
  const runningServices = new Set()

  const save = () => {
    state.storage = Object.fromEntries(storage)
    fs.mkdirSync(path.dirname(officialAddonStateFile), { recursive: true })
    fs.writeFileSync(officialAddonStateFile, JSON.stringify(state, null, 2))
  }
  const addonEntry = (addonId) => {
    const entry = entries.get(addonId)
    if (!entry) throw new Error('Unknown official addon: ' + addonId)
    return entry
  }
  const addonRoot = (entry) => path.dirname(path.join(officialAddonsRoot, entry.manifestPath))
  const manifestFor = (entry) => {
    const manifest = readJson(path.join(officialAddonsRoot, entry.manifestPath), {})
    return {
      ...manifest,
      id: entry.id,
      name: manifest.name || entry.name,
      version: manifest.version || entry.version,
      source: 'official',
      official: true
    }
  }
  const recordFor = (addonId) => {
    const entry = addonEntry(addonId)
    const manifest = manifestFor(entry)
    const entryRelative = String(manifest.runtime?.entry || path.basename(entry.entryPath || 'main.js'))
    const entryPath = path.resolve(addonRoot(entry), entryRelative)
    const packageHash = crypto.createHash('sha256')
      .update(JSON.stringify(manifest))
      .update(fs.existsSync(entryPath) ? fs.readFileSync(entryPath) : Buffer.from('missing-entry'))
      .digest('hex')
    return {
      manifest,
      source: 'official',
      official: true,
      packageHash,
      installedAt: '2026-07-17T00:00:00.000Z',
      enabled: state.enabled.includes(addonId)
    }
  }
  const safeModulePath = (addonId, modulePath) => {
    const entry = addonEntry(addonId)
    const root = path.resolve(addonRoot(entry))
    const normalized = String(modulePath || '').replaceAll('\\', '/').replace(/^\/+/, '')
    const resolved = path.resolve(root, normalized)
    if (resolved !== root && !resolved.startsWith(root + path.sep)) {
      throw new Error('Official addon module escaped package: ' + addonId + '/' + modulePath)
    }
    return resolved
  }
  const setEnabled = (addonId, enabled) => {
    addonEntry(addonId)
    const next = new Set(state.enabled)
    if (enabled) next.add(addonId)
    else next.delete(addonId)
    state.enabled = [...next].sort()
    memory.enabledAddons.set(addonId, enabled === true)
    save()
    return { ok: true, addonId, enabled: enabled === true }
  }
  const normalizeInstallId = (value) => String(value || '').replace(/^official:/, '').trim()
  const install = (value) => {
    const addonId = normalizeInstallId(value)
    addonEntry(addonId)
    if (!state.installed.includes(addonId)) state.installed.push(addonId)
    state.installed.sort()
    save()
    return recordFor(addonId)
  }
  const uninstall = (addonId) => {
    state.installed = state.installed.filter((id) => id !== addonId)
    state.enabled = state.enabled.filter((id) => id !== addonId)
    runningServices.delete(addonId)
    save()
    return { ok: true, addonId }
  }
  const serviceStatus = (addonId) => ({
    addonId,
    available: true,
    running: runningServices.has(addonId),
    mocked: true,
    evidence: 'Renderer E2E uses an explicit service mock; native package presence is validated separately.'
  })
  const callBroker = (addonId, method, params = {}) => {
    const key = addonId + ':' + String(params.key || '')
    if (method === 'storage.get') return storage.get(key) ?? null
    if (method === 'storage.set') { storage.set(key, params.value); save(); return params.value }
    if (method === 'storage.remove') { storage.delete(key); save(); return true }
    if (method === 'storage.entries') {
      return [...storage.entries()]
        .filter(([entryKey]) => entryKey.startsWith(addonId + ':'))
        .map(([entryKey, value]) => [entryKey.slice(addonId.length + 1), value])
    }
    if (method === 'notes.list') return allMarkdownEntries()
    if (method === 'notes.read') return readMarkdown(params.path)
    if (method === 'notes.write') return writeMarkdown(params.path, params.content)
    if (method === 'app.info') return { name: 'Elephant', platform: process.platform, e2e: true }
    return { ok: true, method, params, mocked: true }
  }

  save()
  return {
    listInstalled: () => state.installed.map(recordFor),
    listCatalog: () => (catalog.addons || []).map((entry) => ({
      ...entry,
      manifest: manifestFor(entry),
      installed: state.installed.includes(entry.id)
    })),
    install,
    uninstall,
    setEnabled,
    readEntry: (addonId) => {
      const entry = addonEntry(addonId)
      const manifest = manifestFor(entry)
      const entryRelative = String(manifest.runtime?.entry || path.basename(entry.entryPath || 'main.js'))
      const filename = safeModulePath(addonId, entryRelative)
      return { source: fs.readFileSync(filename, 'utf8'), path: entryRelative }
    },
    readModule: (addonId, modulePath) => {
      const filename = safeModulePath(addonId, modulePath)
      return { source: fs.readFileSync(filename, 'utf8'), path: modulePath }
    },
    callBroker,
    serviceStatus,
    serviceStart: (addonId) => {
      runningServices.add(addonId)
      console.warn('[e2e-addon-service-mock] start ' + addonId)
      return serviceStatus(addonId)
    },
    serviceStop: (addonId) => {
      runningServices.delete(addonId)
      console.warn('[e2e-addon-service-mock] stop ' + addonId)
      return serviceStatus(addonId)
    },
    serviceCall: (addonId, method, params) => ({ ok: true, addonId, method, params, mocked: true })
  }
})()
`

const addonCases = `    case 'tauri_addons_list':
    case 'tauri_addons_list_full': return officialAddonFixture.listInstalled()
    case 'tauri_addons_catalog_list': return officialAddonFixture.listCatalog()
    case 'tauri_official_addons_catalog_list': return officialAddonFixture.listCatalog()
    case 'tauri_addons_install': return officialAddonFixture.install(params.packagePath)
    case 'tauri_addons_catalog_install':
    case 'tauri_addons_install_catalog':
    case 'tauri_addons_install_official': return officialAddonFixture.install(params.addonId || params.id)
    case 'tauri_addons_uninstall': return officialAddonFixture.uninstall(params.addonId)
    case 'tauri_addons_read_module': return officialAddonFixture.readModule(params.addonId, params.path)
    case 'tauri_addons_call': return officialAddonFixture.callBroker(params.addonId, params.method, params.params || {})
    case 'tauri_addons_sidecar_status':
    case 'tauri_addons_service_status': return officialAddonFixture.serviceStatus(params.addonId)
    case 'tauri_addons_service_start': return officialAddonFixture.serviceStart(params.addonId)
    case 'tauri_addons_service_stop': return officialAddonFixture.serviceStop(params.addonId)
    case 'tauri_addons_sidecar_call':
    case 'tauri_addons_service_call': return officialAddonFixture.serviceCall(params.addonId, params.method, params.params || {})`

module.exports = (source) => {
  let patched = String(source)
  if (!patched.includes(INSERT_AFTER_HELPERS)) throw new Error('Unable to locate initialized preload helper boundary')
  patched = patched.replace(INSERT_AFTER_HELPERS, `${INSERT_AFTER_HELPERS}${fixtureSource}\n`)
  if (!patched.includes(EMPTY_ADDON_CASES)) throw new Error('Unable to locate empty addon command cases')
  patched = patched.replace(EMPTY_ADDON_CASES, addonCases)
  if (!patched.includes(ENABLED_CASES)) throw new Error('Unable to locate addon enabled command cases')
  patched = patched.replace(ENABLED_CASES, `    case 'tauri_addons_set_enabled':\n    case 'tauri_addons_set_enabled_checked': return officialAddonFixture.setEnabled(params.addonId, params.enabled === true)`)
  if (!patched.includes(READ_ENTRY_CASE)) throw new Error('Unable to locate addon read-entry fallback')
  patched = patched.replace(READ_ENTRY_CASE, `    case 'tauri_addons_read_entry': return officialAddonFixture.readEntry(params.addonId)`)
  return patched
}
