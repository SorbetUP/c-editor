import { afterEach, describe, expect, it, vi } from 'vitest'

import TrustedWorkspaceLab from '../../../../examples/addons/trusted-workspace-lab/main.js'
import { createAddonHostRuntime } from '../../../../Elephant/frontend/src/renderer/src/addons/addonHostRuntime.js'
import {
  approveTrustedAddon,
  createTrustedAddonApi,
  getTrustedApproval,
  getTrustedSafeMode,
  isTrustedExternalManifest,
  revokeTrustedAddon,
  setTrustedSafeMode
} from '../../../../Elephant/frontend/src/renderer/src/addons/trustedAddonRuntime.js'
import {
  beginTrustedActivation,
  readTrustedActivationMarker,
  recoverTrustedActivationCrash
} from '../../../../Elephant/frontend/src/renderer/src/addons/trustedAddonBootGuard.js'
import {
  ADDON_ACCESS_LEVEL,
  getAddonAccessLevel,
  normalizeAddonManifest
} from '../../../../Elephant/frontend/src/renderer/src/addons/manifest.js'

const createPreferenceTarget = () => {
  const preferences = new Map()
  const local = new Map()
  return {
    preferences,
    local,
    target: {
      localStorage: {
        getItem: (key) => local.get(key) ?? null,
        setItem: (key, value) => local.set(key, String(value)),
        removeItem: (key) => local.delete(key)
      },
      __TAURI__: {
        core: {
          invoke: vi.fn(async (command, payload) => {
            if (command === 'tauri_prefs_get') return preferences.get(payload.key) ?? null
            if (command === 'tauri_prefs_set') {
              preferences.set(payload.key, payload.value)
              return { ok: true }
            }
            throw new Error(`Unexpected command: ${command}`)
          })
        }
      }
    }
  }
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('trusted addon manifest model', () => {
  it('keeps legacy isolated addons and recognizes explicit full app access', () => {
    const isolated = normalizeAddonManifest({
      id: 'com.example.isolated',
      name: 'Isolated',
      version: '1.0.0',
      source: 'external',
      runtime: { type: 'javascript-worker', entry: 'main.js' }
    })
    const trusted = normalizeAddonManifest({
      id: 'com.example.trusted',
      name: 'Trusted',
      version: '1.0.0',
      source: 'external',
      runtime: { type: 'javascript-worker', entry: 'main.js' },
      contributes: { runtimeMode: 'trusted' }
    })

    expect(getAddonAccessLevel(isolated)).toBe(ADDON_ACCESS_LEVEL.isolated)
    expect(getAddonAccessLevel(trusted)).toBe(ADDON_ACCESS_LEVEL.trusted)
    expect(isTrustedExternalManifest(trusted)).toBe(true)
  })

  it('treats builtin addons as system addons', () => {
    const manifest = normalizeAddonManifest({
      id: 'elephant.system-test',
      name: 'System test',
      version: '1.0.0'
    })
    expect(getAddonAccessLevel(manifest)).toBe(ADDON_ACCESS_LEVEL.system)
  })
})

describe('hash-bound trusted approval', () => {
  it('requires approval again when the package hash changes', async () => {
    const { target } = createPreferenceTarget()
    const first = {
      manifest: { id: 'com.example.trusted' },
      packageHash: 'hash-v1'
    }
    const updated = {
      manifest: { id: 'com.example.trusted' },
      packageHash: 'hash-v2'
    }

    expect((await getTrustedApproval(first, target)).approved).toBe(false)
    expect((await approveTrustedAddon(first, target)).approved).toBe(true)
    expect((await getTrustedApproval(first, target)).approved).toBe(true)
    expect((await getTrustedApproval(updated, target)).approved).toBe(false)
    expect((await revokeTrustedAddon(first, target)).approved).toBe(false)
  })

  it('persists emergency safe mode independently of addon packages', async () => {
    const { target } = createPreferenceTarget()
    expect(await getTrustedSafeMode(target)).toBe(false)
    await setTrustedSafeMode(true, target)
    expect(await getTrustedSafeMode(target)).toBe(true)
    await setTrustedSafeMode(false, target)
    expect(await getTrustedSafeMode(target)).toBe(false)
  })

  it('enters safe mode after an interrupted trusted activation', async () => {
    const { target, local, preferences } = createPreferenceTarget()
    const record = {
      manifest: { id: 'com.example.crashing' },
      packageHash: 'crashing-hash'
    }

    beginTrustedActivation(record, target)
    expect(readTrustedActivationMarker(target)).toEqual(expect.objectContaining({
      addonId: 'com.example.crashing',
      packageHash: 'crashing-hash'
    }))

    const recovered = await recoverTrustedActivationCrash(target)

    expect(recovered).toEqual(expect.objectContaining({ addonId: 'com.example.crashing' }))
    expect(readTrustedActivationMarker(target)).toBeNull()
    expect(local.get('elephantnote:addons:trusted-safe-mode')).toBe('true')
    expect(preferences.get('addons.trustedSafeMode')).toBe(true)
  })
})

describe('trusted addon host API', () => {
  it('opens addon workspace views through the AppShell event contract', () => {
    const target = { document: {}, dispatchEvent: vi.fn() }
    const context = {
      addonHost: null,
      router: {},
      pinia: {},
      services: {},
      runtime: 'tauri',
      addons: {},
      vueApp: {}
    }
    const api = createTrustedAddonApi({ manifest: { id: 'com.example.view' }, packageHash: 'hash' }, context, [], target)

    api.workspace.openView('com.example.view.workspace', { source: 'acceptance' })

    expect(target.dispatchEvent).toHaveBeenCalledOnce()
    const event = target.dispatchEvent.mock.calls[0][0]
    expect(event.type).toBe('elephantnote:open-addon-view')
    expect(event.detail).toEqual({
      viewId: 'com.example.view.workspace',
      params: { source: 'acceptance' },
      addonId: 'com.example.view'
    })
  })

  it('exposes deep application context, patches resources and restores everything', () => {
    const removed = vi.fn()
    const appended = []
    const fakeDocument = {
      head: { appendChild: (node) => appended.push(node) },
      createElement: () => ({
        dataset: {},
        textContent: '',
        remove: removed
      })
    }
    const target = { document: fakeDocument }
    const service = { value: 2, calculate(number) { return this.value + number } }
    const addonHost = createAddonHostRuntime({ services: { service } }, target)
    const contributions = []
    const contextDisposables = []
    const context = {
      router: { currentRoute: {} },
      pinia: { state: {} },
      services: { service },
      runtime: 'tauri',
      addons: { list: vi.fn() },
      addonHost,
      vueApp: { component: vi.fn() },
      registerContribution: vi.fn((area, contribution) => {
        contributions.push({ area, contribution })
        return vi.fn()
      }),
      addAction: vi.fn(),
      addView: vi.fn(),
      addSidebarItem: vi.fn(),
      addSettingsSection: vi.fn(),
      addEditorExtension: vi.fn(),
      addStatusBarItem: vi.fn(),
      addDisposable: (dispose) => contextDisposables.push(dispose)
    }
    const record = {
      manifest: { id: 'com.example.trusted', permissions: {} },
      packageHash: 'abc123'
    }
    const sessionDisposables = []

    const api = createTrustedAddonApi(record, context, sessionDisposables, target)
    api.ui.registerStyle('.test { display: none; }', 'test')
    api.editor.registerBlockType({ id: 'com.example.trusted.block' })
    api.markdown.registerEmbedRenderer({ id: 'com.example.trusted.embed' })
    api.layout.registerItem({ id: 'com.example.trusted.layout' })
    api.resources.provide('customService', { ready: true })
    api.patch.method(service, 'calculate', (original, number) => original(number) * 10)
    api.patch.property(service, 'value', 5)

    expect(api.access.level).toBe('trusted')
    expect(api.app.router).toBe(context.router)
    expect(api.app.host).toBe(addonHost)
    expect(api.experimental.services).toBe(context.services)
    expect(api.resources.get('customService')).toEqual({ ready: true })
    expect(service.calculate(1)).toBe(60)
    expect(appended).toHaveLength(1)
    expect(contributions.map((entry) => entry.area)).toEqual([
      'editor.block-types',
      'markdown.embed-renderers',
      'layout.items'
    ])
    expect(contextDisposables.length).toBeGreaterThan(0)

    for (const dispose of [...contextDisposables].reverse()) dispose()
    expect(service.value).toBe(2)
    expect(service.calculate(1)).toBe(3)
    expect(api.resources.get('customService')).toBeUndefined()
    expect(removed).toHaveBeenCalledTimes(1)
  })

  it('executes the reference addon lifecycle and deep contributions', async () => {
    const classes = new Set()
    const commandDefinitions = []
    const resources = new Map()
    const api = {
      manifest: { id: 'com.elephantnote.examples.trusted-workspace-lab' },
      ui: {
        registerStyle: vi.fn(() => vi.fn()),
        on: vi.fn(() => vi.fn())
      },
      resources: {
        list: vi.fn(() => ['router', 'services']),
        provide: vi.fn((name, value) => {
          resources.set(name, value)
          return () => resources.delete(name)
        })
      },
      commands: {
        register: vi.fn((definition) => {
          commandDefinitions.push(definition)
          return vi.fn()
        })
      },
      workspace: { registerSidebarItem: vi.fn() },
      settings: { registerSection: vi.fn() },
      editor: { registerExtension: vi.fn() },
      layout: { registerItem: vi.fn() },
      app: { emit: vi.fn() },
      experimental: {
        window: {},
        document: {
          documentElement: {
            classList: {
              toggle: (name, enabled) => enabled ? classes.add(name) : classes.delete(name),
              remove: (name) => classes.delete(name)
            }
          }
        }
      }
    }
    const plugin = new TrustedWorkspaceLab(api)

    await plugin.onload(api)

    expect(api.ui.registerStyle).toHaveBeenCalledOnce()
    expect(api.commands.register).toHaveBeenCalledOnce()
    expect(api.workspace.registerSidebarItem).toHaveBeenCalledOnce()
    expect(api.settings.registerSection).toHaveBeenCalledOnce()
    expect(api.editor.registerExtension).toHaveBeenCalledOnce()
    expect(api.layout.registerItem).toHaveBeenCalledOnce()
    expect(api.resources.provide).toHaveBeenCalledWith('trustedWorkspaceLab', plugin)
    expect(api.app.emit).toHaveBeenCalledWith('elephantnote:trusted-addon-loaded', expect.objectContaining({
      addonId: 'com.elephantnote.examples.trusted-workspace-lab',
      resources: ['router', 'services']
    }))

    const settingsDefinition = api.settings.registerSection.mock.calls[0][0]
    expect(settingsDefinition.section).toBe('addons')
    expect(typeof settingsDefinition.render).toBe('function')

    expect(commandDefinitions).toHaveLength(1)
    expect(commandDefinitions[0].run()).toEqual({ enabled: true })
    expect(classes.has('elephant-trusted-focus')).toBe(true)
    expect(commandDefinitions[0].run()).toEqual({ enabled: false })
    expect(classes.has('elephant-trusted-focus')).toBe(false)

    classes.add('elephant-trusted-focus')
    await plugin.onunload()
    expect(classes.has('elephant-trusted-focus')).toBe(false)
  })
})
