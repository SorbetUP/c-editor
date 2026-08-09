import { afterEach, describe, expect, it, vi } from 'vitest'

import { addonPacksCoreFeature } from '../../../../Elephant/frontend/src/renderer/src/addons/builtin/addonProfiles.js'

const PACK_PATH = '.elephantnote/addons/packs/default.enaddonpack'
const BASE_PACK_PATH = '.elephantnote/addons/packs/base.enaddonpack'
const DEVELOP_PARITY_PACK_PATH = '.elephantnote/addons/packs/develop-parity.enaddonpack'
const REPORT_PATH = 'Reports/Addon Pack.md'

const catalog = [
  { id: 'elephant.ai', name: 'AI', version: '2.1.0', official: true },
  { id: 'elephant.calendar', name: 'Calendar', version: '1.2.0', official: true },
  { id: 'elephant.google-keep-import', name: 'Google Keep Import', version: '1.1.0', official: true },
  { id: 'elephant.recently-edited', name: 'Recently edited', version: '1.1.0', official: true },
  { id: 'com.example.alpha', name: 'Alpha', version: '1.0.0', official: false },
  { id: 'com.example.beta', name: 'Beta', version: '2.0.0', official: false }
]

const snapshot = (id, options = {}) => ({
  manifest: {
    id,
    name: options.name || id,
    version: options.version || '1.0.0',
    source: options.source || 'external',
    official: options.official === true,
    permissions: {},
    runtime: { type: 'javascript-worker', entry: 'main.js' }
  },
  enabled: options.enabled === true,
  status: options.enabled === true ? 'enabled' : 'disabled'
})

const createManager = (initial = []) => {
  const records = new Map(initial.map((record) => [record.manifest.id, record]))
  const manager = {
    list: () => [...records.values()],
    get: (id) => records.get(id) || null,
    async enable(id) {
      const record = records.get(id)
      if (!record) throw new Error(`Unknown addon: ${id}`)
      record.enabled = true
      record.status = 'enabled'
      return record
    },
    async disable(id) {
      const record = records.get(id)
      if (record) {
        record.enabled = false
        record.status = 'disabled'
      }
      return record
    },
    unregister(id) { records.delete(id) }
  }
  manager.external = {
    manager,
    register(record) {
      records.set(record.manifest.id, {
        manifest: {
          ...record.manifest,
          source: 'external',
          official: record.source === 'official' || record.manifest.official === true
        },
        enabled: record.enabled === true,
        status: record.enabled === true ? 'enabled' : 'disabled'
      })
    },
    setSafeMode: vi.fn(async () => false),
    approveTrusted: vi.fn(async () => ({ approved: true }))
  }
  return { manager, records }
}

const activateActions = (manager) => {
  const actions = new Map()
  addonPacksCoreFeature.activate({
    addons: manager,
    addonHost: { get: (key) => key === 'addonManager' ? manager : null },
    logger: { info: vi.fn() },
    addAction(action) { actions.set(action.id, action) },
    addSettingsSection: vi.fn()
  })
  return actions
}

const createInvoke = (files, communityEnabled = true) => vi.fn(async (command, payload = {}) => {
  if (command === 'tauri_addons_catalog_list') return catalog
  if (command === 'tauri_prefs_get') return communityEnabled
  if (command === 'tauri_addons_set_enabled') return { addonId: payload.addonId, enabled: payload.enabled === true }
  if (command === 'tauri_notes_write') {
    files.set(payload.relativePath, payload.content)
    return { ok: true, path: payload.relativePath }
  }
  if (command === 'tauri_notes_read') {
    if (!files.has(payload.relativePath)) throw new Error('not found')
    return { path: payload.relativePath, content: files.get(payload.relativePath) }
  }
  if (command === 'tauri_addons_catalog_install') {
    const addon = catalog.find((entry) => entry.id === payload.addonId)
    if (!addon) throw new Error(`Unknown catalogue addon: ${payload.addonId}`)
    return {
      manifest: {
        id: addon.id,
        name: addon.name,
        version: addon.version,
        official: addon.official === true,
        permissions: {},
        runtime: { type: 'javascript-worker', entry: 'main.js' }
      },
      enabled: false,
      packageHash: `hash-${addon.id}`,
      installedAt: '2026-07-13T00:00:00Z',
      source: addon.official ? 'official' : 'catalog'
    }
  }
  throw new Error(`Unexpected command: ${command}`)
})

afterEach(() => {
  vi.unstubAllGlobals()
  vi.clearAllMocks()
})

describe('Addon Packs core feature', () => {
  it('captures official and community packages with distinct sources', async () => {
    const files = new Map()
    const { manager } = createManager([
      snapshot('com.example.alpha', { name: 'Alpha', enabled: true }),
      snapshot('elephant.calendar', { version: '1.2.0', official: true, enabled: false })
    ])
    vi.stubGlobal('__TAURI__', { core: { invoke: createInvoke(files) } })
    const actions = activateActions(manager)
    const created = await actions.get('elephant.addon-packs.create').run()

    expect(created.path).toBe(PACK_PATH)
    const exported = JSON.parse(files.get(PACK_PATH))
    expect(exported.addons).toEqual(expect.arrayContaining([
      expect.objectContaining({ id: 'com.example.alpha', source: 'catalog', enabled: true }),
      expect.objectContaining({ id: 'elephant.calendar', source: 'official', enabled: false })
    ]))
  })

  it('installs official packages without requiring Community Addons', async () => {
    const files = new Map([[PACK_PATH, JSON.stringify({
      format: 'elephantnote-addon-pack',
      version: 1,
      name: 'Official setup',
      addons: [
        { id: 'elephant.ai', source: 'official', version: '2.1.0', enabled: true },
        { id: 'elephant.calendar', source: 'official', version: '1.2.0', enabled: true }
      ]
    })]])
    const { manager, records } = createManager()
    vi.stubGlobal('__TAURI__', { core: { invoke: createInvoke(files, false) } })
    const actions = activateActions(manager)
    const result = await actions.get('elephant.addon-packs.apply').run()

    expect(result.applied).toBe(2)
    expect(records.get('elephant.ai').enabled).toBe(true)
    expect(records.get('elephant.calendar').enabled).toBe(true)
    expect(records.get('elephant.ai').manifest.official).toBe(true)
    expect(files.get(REPORT_PATH)).toContain('| `elephant.calendar` | official | 1.2.0 | Enabled | Installed |')
  })

  it('still requires Community Addons for enabled third-party catalogue packages', async () => {
    const files = new Map([[PACK_PATH, JSON.stringify({
      format: 'elephantnote-addon-pack',
      version: 1,
      addons: [{ id: 'com.example.beta', source: 'catalog', version: '2.0.0', enabled: true }]
    })]])
    const { manager } = createManager()
    vi.stubGlobal('__TAURI__', { core: { invoke: createInvoke(files, false) } })
    const actions = activateActions(manager)
    await expect(actions.get('elephant.addon-packs.apply').run())
      .rejects.toThrow('Turn on Community Addons')
  })

  it('creates protected complete and base packs containing only physical addons', async () => {
    const files = new Map()
    const { manager } = createManager()
    vi.stubGlobal('__TAURI__', { core: { invoke: createInvoke(files) } })
    const actions = activateActions(manager)
    const result = await actions.get('elephant.addon-packs.ensure-develop-parity').run()

    expect(result.packPath).toBe(DEVELOP_PARITY_PACK_PATH)
    expect(result.basePackPath).toBe(BASE_PACK_PATH)
    const completePack = JSON.parse(files.get(DEVELOP_PARITY_PACK_PATH))
    const basePack = JSON.parse(files.get(BASE_PACK_PATH))
    expect(completePack.protected).toBe(true)
    expect(completePack.addons).toEqual(expect.arrayContaining([
      { id: 'elephant.ai', version: '2.1.0', source: 'official', enabled: true },
      { id: 'elephant.calendar', version: '1.2.0', source: 'official', enabled: true },
      { id: 'elephant.google-keep-import', version: '1.1.0', source: 'official', enabled: true },
      { id: 'elephant.recently-edited', version: '1.1.0', source: 'official', enabled: true }
    ]))
    expect(completePack.addons).not.toEqual(expect.arrayContaining([
      expect.objectContaining({ id: 'elephant.addon-packs' }),
      expect.objectContaining({ id: 'elephant.excalidraw' }),
      expect.objectContaining({ source: 'builtin' })
    ]))
    expect(basePack).toMatchObject({ name: 'Elephant Base', protected: true })
    expect(basePack.addons).not.toContainEqual(expect.objectContaining({ id: 'elephant.calendar' }))
  })

  it('rejects packages that are absent from the catalogue', async () => {
    const files = new Map([[PACK_PATH, JSON.stringify({
      format: 'elephantnote-addon-pack',
      version: 1,
      addons: [{ id: 'unknown.addon', source: 'catalog', enabled: true }]
    })]])
    const { manager } = createManager()
    vi.stubGlobal('__TAURI__', { core: { invoke: createInvoke(files) } })
    const actions = activateActions(manager)
    await expect(actions.get('elephant.addon-packs.apply').run())
      .rejects.toThrow('missing catalogue package')
  })
})
