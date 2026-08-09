import fs from 'node:fs'
import path from 'node:path'
import { pathToFileURL } from 'node:url'
import { describe, expect, it, vi } from 'vitest'

const root = process.cwd()
const readJson = (relativePath) => JSON.parse(fs.readFileSync(path.join(root, relativePath), 'utf8'))
const catalog = readJson('addons/catalog.json')
const basePack = readJson('addons/packs/base.enaddonpack')
const parityPack = readJson('addons/packs/develop-parity.enaddonpack')
const catalogById = new Map(catalog.addons.map((entry) => [entry.id, entry]))

const createCallableFallback = () => {
  const callable = () => undefined
  return new Proxy(callable, {
    get: (_target, property) => {
      if (property === 'then') return undefined
      if (property === Symbol.toPrimitive) return () => ''
      return createCallableFallback()
    },
    apply: () => undefined
  })
}

const createRuntimeProbe = (addonId) => {
  const registrations = {
    actions: [],
    views: [],
    sidebarItems: [],
    statusItems: [],
    settings: [],
    zones: [],
    resources: new Map(),
    styles: []
  }
  const storage = new Map()
  let aiConfig = {}
  if (addonId === 'elephant.open-models') {
    registrations.resources.set('ai.config', Object.freeze({
      get: vi.fn(async () => aiConfig),
      set: vi.fn(async (value) => {
        aiConfig = value || {}
        return aiConfig
      })
    }))
  }

  const vaultStore = {
    activeVaultId: 'base-addon-probe',
    activeVault: { id: 'base-addon-probe', name: 'Base addon probe', path: '/tmp/elephant-base-addon-probe' },
    workspaceStats: { notes: 3, folders: 1 },
    activeNoteEntries: [],
    recentNoteEntries: [
      { path: 'Probe.md', title: 'Probe', kind: 'note', type: 'note', updatedAt: '2026-07-15T20:00:00Z' }
    ],
    openedNotePath: '',
    openNote: vi.fn((note) => { vaultStore.openedNotePath = note?.path || '' }),
    closeNote: vi.fn(() => { vaultStore.openedNotePath = '' })
  }
  const fakeWindow = window
  fakeWindow.__ELEPHANT_ADDON_VUE__ = {
    createDomComponent: ({ name, mount, className = '' }) => ({ name, className, __mount: mount })
  }
  fakeWindow.__TAURI__ = {
    core: {
      invoke: vi.fn(async (command, payload = {}) => {
        if (command === 'tauri_notes_read') throw new Error('probe note does not exist yet')
        if (command === 'tauri_notes_create') {
          return {
            path: [payload.relativePath, payload.filename || 'Untitled.md'].filter(Boolean).join('/'),
            fullPath: `/tmp/elephant-base-addon-probe/${payload.filename || 'Untitled.md'}`,
            title: payload.title || 'Untitled'
          }
        }
        if (command.includes('status')) return { ok: true, configured: false, running: false }
        if (command.includes('list')) return []
        return { ok: true }
      })
    }
  }
  fakeWindow.elephantnote = {
    getVaults: vi.fn(async () => ({ activeVault: vaultStore.activeVault, activeVaultId: vaultStore.activeVaultId })),
    api: createCallableFallback()
  }

  const track = (collection, value) => {
    collection.push(value)
    return () => {
      const index = collection.indexOf(value)
      if (index >= 0) collection.splice(index, 1)
    }
  }

  const installedRecords = new Map(parityPack.addons.map((addon) => [addon.id, {
    enabled: addon.enabled !== false,
    manifest: catalogById.get(addon.id) || { id: addon.id, version: addon.version }
  }]))
  const addonManager = {
    get: (id) => installedRecords.get(id) || null,
    list: () => [...installedRecords.values()]
  }

  const api = {
    manifest: { id: addonId },
    experimental: { window: fakeWindow },
    app: {
      pinia: { _s: new Map([['elephantnoteVaults', vaultStore]]) },
      router: createCallableFallback(),
      services: createCallableFallback(),
      runtime: 'test',
      addons: addonManager,
      host: createCallableFallback(),
      vueApp: createCallableFallback(),
      emit: vi.fn(),
      openSettings: vi.fn()
    },
    storage: {
      get: vi.fn(async (key) => storage.get(key) ?? null),
      set: vi.fn(async (key, value) => { storage.set(key, value); return value }),
      remove: vi.fn(async (key) => storage.delete(key)),
      entries: vi.fn(async () => [...storage.entries()])
    },
    native: {
      status: vi.fn(async () => ({ ok: true, configured: false, running: false })),
      call: vi.fn(async () => ({ ok: true, configured: false, items: [] })),
      service: {
        status: vi.fn(async () => ({ ok: true, configured: false, running: false })),
        start: vi.fn(async () => ({ ok: true, running: true })),
        call: vi.fn(async () => ({ ok: true, configured: false, items: [], records: [], nodes: [], edges: [] })),
        stop: vi.fn(async () => ({ ok: true, running: false }))
      }
    },
    resources: {
      get: (name) => registrations.resources.get(name),
      has: (name) => registrations.resources.has(name),
      list: () => [...registrations.resources.keys()],
      provide: (name, value) => {
        registrations.resources.set(name, value)
        return () => registrations.resources.delete(name)
      },
      watch: (_name, listener, options = {}) => {
        if (options.immediate !== false) listener({ value: registrations.resources.get(_name), previous: undefined })
        return () => {}
      }
    },
    workspace: {
      registerView: (definition) => track(registrations.views, definition),
      registerSidebarItem: (definition) => track(registrations.sidebarItems, definition),
      registerStatusBarItem: (definition) => track(registrations.statusItems, definition),
      registerContribution: () => () => {},
      openView: vi.fn()
    },
    editor: {
      active: null,
      watch: () => () => {},
      registerExtension: () => () => {},
      registerBlockType: () => () => {},
      registerInlineType: () => () => {},
      registerInputRule: () => () => {},
      registerToolbarItem: () => () => {},
      registerPasteHandler: () => () => {}
    },
    markdown: {
      registerPostProcessor: () => () => {},
      registerCodeBlockProcessor: () => () => {},
      registerEmbedRenderer: () => () => {}
    },
    settings: {
      registerSection: (definition) => track(registrations.settings, definition),
      registerPage: (definition) => track(registrations.settings, definition)
    },
    layout: {
      registerItem: (definition) => track(registrations.zones, definition),
      registerZone: (definition) => track(registrations.zones, definition)
    },
    commands: {
      register: (definition) => track(registrations.actions, definition)
    },
    router: {
      addRoute: () => () => {},
      beforeEach: () => () => {},
      afterEach: () => () => {}
    },
    vue: {
      component: () => () => {},
      directive: () => () => {},
      provide: () => () => {}
    },
    ui: {
      registerStyle: (cssText, id = '') => track(registrations.styles, { cssText, id }),
      mount: (_host, renderer) => {
        const container = document.createElement('div')
        return renderer(container) || (() => {})
      },
      on: () => () => {},
      observe: () => () => {}
    },
    patch: {
      method: () => () => {},
      property: () => () => {},
      hook: () => () => {},
      runHook: async (_name, value) => value
    },
    http: {
      request: vi.fn(async () => ({ ok: true, status: 200, json: {}, text: '' }))
    }
  }

  return { api, registrations, vaultStore }
}

const activate = async (catalogEntry) => {
  const modulePath = path.join(root, 'addons', catalogEntry.entryPath)
  const imported = await import(/* @vite-ignore */ `${pathToFileURL(modulePath).href}?probe=${catalogEntry.id}`)
  expect(imported.default, `${catalogEntry.id} must export a default addon class`).toBeTypeOf('function')
  const probe = createRuntimeProbe(catalogEntry.id)
  const instance = Reflect.construct(imported.default, [probe.api])
  expect(instance.onload, `${catalogEntry.id} must implement onload(api)`).toBeTypeOf('function')
  await instance.onload(probe.api)
  return { ...probe, instance }
}

const assertNoIntegrationError = async (operation, label) => {
  try {
    await operation()
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    expect(message, label).not.toMatch(/ReferenceError|TypeError|is not a function|Cannot read properties|undefined is not/i)
  }
}

describe('first-party base addon installation and runtime probes', () => {
  it('keeps every protected base addon downloadable from the integrated official catalogue', () => {
    const baseIds = basePack.addons.map((addon) => addon.id)
    const parityIds = parityPack.addons.map((addon) => addon.id)
    expect(baseIds).toContain('elephant.dashboard')
    expect(parityIds).toContain('elephant.calendar')
    for (const id of new Set([...baseIds, ...parityIds])) {
      const entry = catalogById.get(id)
      expect(entry, `${id} disappeared from addons/catalog.json`).toBeTruthy()
      expect(entry.official).toBe(true)
      expect(fs.existsSync(path.join(root, 'addons', entry.manifestPath))).toBe(true)
      expect(fs.existsSync(path.join(root, 'addons', entry.entryPath))).toBe(true)
    }
  })

  it('orders every declared dependency before its consumer in protected packs', () => {
    for (const pack of [basePack, parityPack]) {
      const positions = new Map(pack.addons.map((addon, index) => [addon.id, index]))
      for (const addon of pack.addons) {
        const entry = catalogById.get(addon.id)
        const manifest = readJson(path.join('addons', entry.manifestPath))
        for (const dependencyId of Object.keys(manifest.requires || {})) {
          expect(positions.has(dependencyId), `${pack.name}: ${addon.id} is missing ${dependencyId}`).toBe(true)
          expect(positions.get(dependencyId), `${pack.name}: ${dependencyId} must precede ${addon.id}`).toBeLessThan(positions.get(addon.id))
        }
      }
    }
  })

  for (const packedAddon of parityPack.addons) {
    const catalogEntry = catalogById.get(packedAddon.id)
    it(`activates and exercises ${packedAddon.id}`, async () => {
      expect(catalogEntry).toBeTruthy()
      const { registrations } = await activate(catalogEntry)
      const contributionCount = registrations.actions.length + registrations.views.length + registrations.settings.length + registrations.zones.length + registrations.resources.size
      expect(contributionCount, `${packedAddon.id} registered no usable host capability`).toBeGreaterThan(0)

      for (const view of registrations.views) {
        expect(view.id, `${packedAddon.id} registered a view without an id`).toBeTruthy()
        if (view.component?.__mount) {
          const container = document.createElement('div')
          document.body.append(container)
          await assertNoIntegrationError(async () => view.component.__mount(container), `${packedAddon.id}:${view.id} view mount`)
          container.remove()
        }
      }

      for (const [name, resource] of registrations.resources) {
        if (typeof resource?.status === 'function') {
          await assertNoIntegrationError(() => resource.status(), `${packedAddon.id}:${name}.status`)
        }
        if (typeof resource?.list === 'function') {
          await assertNoIntegrationError(() => resource.list(), `${packedAddon.id}:${name}.list`)
        }
      }

      for (const command of registrations.actions) {
        expect(command.id, `${packedAddon.id} registered a command without an id`).toBeTruthy()
        expect(command.run, `${packedAddon.id}:${command.id} has no run function`).toBeTypeOf('function')
        await assertNoIntegrationError(() => command.run({ probe: true }), `${packedAddon.id}:${command.id}`)
      }
    })
  }
})
