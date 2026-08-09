import { describe, expect, it, vi } from 'vitest'
import { installAcceptanceTestBridge } from '../../../Elephant/frontend/src/renderer/src/platform/acceptanceTestBridge.js'

describe('renderer acceptance test bridge', () => {
  it('opens, edits, saves and reads the displayed note state', async() => {
    const target = {
      document: document.implementation.createHTMLDocument('acceptance'),
      console,
      __ELEPHANT_DEBUG_LOGS__: []
    }
    const surface = target.document.createElement('div')
    surface.setAttribute('contenteditable', 'true')
    surface.textContent = 'Initial note'
    target.document.body.append(surface)

    const editorStore = {
      currentFile: { id: 'test-1', pathname: '', markdown: '', isSaved: true, history: null, cursor: null },
      FILE_SAVE: vi.fn(function() { this.currentFile.isSaved = true })
    }
    const vaultStore = {
      activeVault: { path: '/vault' },
      openedNotePath: '',
      entries: [{ path: 'Inbox/Acceptance.md', title: 'Acceptance' }],
      rootEntries: [],
      openNote(entry) {
        this.openedNotePath = entry.path
        editorStore.currentFile.pathname = entry.path
        editorStore.currentFile.markdown = '# Initial'
        editorStore.currentFile.id = 'test-1'
      }
    }

    const api = installAcceptanceTestBridge({ target, editorStore, vaultStore })
    await expect(api.openNote('Inbox/Acceptance.md')).resolves.toMatchObject({
      notePath: 'Inbox/Acceptance.md',
      markdown: '# Initial'
    })
    api.setMarkdown('# Updated\n\nSecond line')
    surface.textContent = 'Updated\n\nSecond line'
    await expect(api.save()).resolves.toMatchObject({ isSaved: true, markdown: '# Updated\n\nSecond line' })
    expect(api.readDisplayed()).toMatchObject({
      notePath: 'Inbox/Acceptance.md',
      displayedText: 'Updated\n\nSecond line'
    })
    expect(api.logs().map((entry) => entry.event)).toEqual(expect.arrayContaining([
      'installed', 'open:start', 'open:done', 'edit:set-markdown', 'save:start', 'save:done', 'read:displayed'
    ]))
  })

  it('exposes observable note and directory commands for app-level scenarios', async() => {
    const calls = []
    const target = {
      document: document.implementation.createHTMLDocument('acceptance-commands'),
      console,
      __TAURI__: {
        core: {
          invoke: vi.fn(async(command, payload) => {
            calls.push({ command, payload })
            if (command === 'tauri_directory_list') return [{ type: 'note', path: 'A.md' }, { type: 'folder', path: 'Folder' }]
            if (command === 'tauri_notes_read') return { path: payload.relativePath, content: '# From disk\n' }
            if (command === 'tauri_notes_create') return { path: `${payload.relativePath}/${payload.filename}` }
            throw new Error(`Unexpected command: ${command}`)
          })
        }
      }
    }
    const api = installAcceptanceTestBridge({
      target,
      editorStore: { currentFile: null },
      vaultStore: { activeVault: { path: '/vault' }, entries: [], rootEntries: [] }
    })

    await expect(api.listNotes('')).resolves.toEqual([{ type: 'note', path: 'A.md' }])
    await expect(api.readNote('A.md')).resolves.toMatchObject({ content: '# From disk\n' })
    await expect(api.createNote('Acceptance')).resolves.toEqual({ path: 'Acceptance/Acceptance-created.md' })
    expect(calls.map(({ command }) => command)).toEqual([
      'tauri_directory_list', 'tauri_notes_read', 'tauri_notes_create'
    ])
    expect(api.logs().map((entry) => entry.event)).toEqual(expect.arrayContaining([
      'notes:list', 'note:read', 'note:create', 'tauri:invoke:start', 'tauri:invoke:done'
    ]))
  })

  it('logs backend failures and exposes DOM/capability snapshots', async() => {
    const target = {
      document: document.implementation.createHTMLDocument('acceptance-observability'),
      console,
      Event,
      __TAURI__: { core: { invoke: vi.fn(async() => { throw new Error('backend exploded') }) } }
    }
    const surface = target.document.createElement('section')
    surface.dataset.testid = 'editor-surface'
    surface.textContent = 'Visible content'
    target.document.body.append(surface)
    const input = target.document.createElement('input')
    input.id = 'query'
    target.document.body.append(input)
    const api = installAcceptanceTestBridge({
      target,
      editorStore: { currentFile: { pathname: 'Acceptance.md', markdown: '# Acceptance' } },
      vaultStore: { activeVault: { path: '/vault' }, openedNotePath: '' }
    })

    expect(api.readDom('[data-testid="editor-surface"]').text).toBe('Visible content')
    expect(api.fill('#query', 'Probe').text).toBe('')
    expect(api.capabilities().commands).toContain('invokeTauri')
    await expect(api.readNote('Acceptance.md')).rejects.toThrow('backend exploded')
    expect(api.logs().map((entry) => entry.event)).toEqual(expect.arrayContaining(['dom:read', 'capabilities:read', 'tauri:invoke:start', 'tauri:invoke:error']))
  })

  it('dispatches real beforeinput text through the Rust editor surface', () => {
    const target = {
      document: document.implementation.createHTMLDocument('acceptance-input'),
      console,
      InputEvent,
      __ELEPHANT_DEBUG_LOGS__: []
    }
    const surface = target.document.createElement('div')
    surface.dataset.testid = 'editor-surface'
    const received = []
    surface.addEventListener('beforeinput', (event) => received.push({ inputType: event.inputType, data: event.data }))
    target.document.body.append(surface)
    const api = installAcceptanceTestBridge({
      target,
      editorStore: { currentFile: null },
      vaultStore: { activeVault: { path: '/vault' }, openedNotePath: '' }
    })

    api.insertText('[data-testid="editor-surface"]', 'typed by Tauri')

    expect(received).toEqual([{ inputType: 'insertText', data: 'typed by Tauri' }])
    expect(api.logs().map((entry) => entry.event)).toContain('dom:insert-text')
  })

  it('inserts text into the Muya JS contenteditable surface and emits input', () => {
    const dom = document
    const target = {
      document: dom,
      getSelection: () => globalThis.getSelection(),
      InputEvent,
      Event,
      MouseEvent,
      console,
      __ELEPHANT_DEBUG_LOGS__: []
    }
    const surface = dom.createElement('div')
    surface.setAttribute('contenteditable', 'true')
    surface.dataset.testid = 'muya-runtime-editor'
    surface.textContent = 'Start'
    dom.body.append(surface)
    const input = vi.fn()
    surface.addEventListener('input', input)
    const api = installAcceptanceTestBridge({
      target,
      editorStore: { currentFile: null },
      vaultStore: { activeVault: { path: '/vault' }, openedNotePath: '' }
    })

    api.selectText('[data-testid="muya-runtime-editor"]', 5, 5)
    api.insertText('[data-testid="muya-runtime-editor"]', ' JS input')

    expect(surface.textContent).toBe('Start JS input')
    expect(input).toHaveBeenCalledOnce()
    expect(api.logs().map((entry) => entry.event)).toContain('dom:insert-text:contenteditable')
  })

  it('preserves the selected DOM range while synchronizing the Muya cursor', () => {
    const dom = document
    const selection = globalThis.getSelection()
    const target = {
      document: dom,
      getSelection: () => selection,
      Event,
      MouseEvent,
      console,
      __ELEPHANT_ACTIVE_MUYA__: {
        contentState: {
          cursor: null,
          selectionChange: vi.fn(() => ({
            start: { key: 'paragraph', offset: 0 },
            end: { key: 'paragraph', offset: 6 }
          }))
        },
        dispatchSelectionChange: vi.fn(() => selection.removeAllRanges())
      },
      __ELEPHANT_DEBUG_LOGS__: []
    }
    const surface = dom.createElement('div')
    surface.setAttribute('contenteditable', 'true')
    surface.dataset.testid = 'muya-selection-editor'
    surface.textContent = 'Select this text'
    dom.body.append(surface)
    const api = installAcceptanceTestBridge({
      target,
      editorStore: { currentFile: null },
      vaultStore: { activeVault: { path: '/vault' }, openedNotePath: '' }
    })

    const result = api.selectText('[data-testid="muya-selection-editor"]', 0, 6)

    expect(result.text).toBe('Select')
    expect(selection.toString()).toBe('Select')
    expect(target.__ELEPHANT_ACTIVE_MUYA__.contentState.cursor).toMatchObject({
      start: { key: 'paragraph', offset: 0 },
      end: { key: 'paragraph', offset: 6 },
      isEdit: true
    })
  })

  it('exposes observable addon state, actions, resources and persisted enablement', async() => {
    const calls = []
    const resource = { status: vi.fn(async() => ({ available: true })) }
    const addon = {
      manifest: { id: 'elephant.test', name: 'Test addon', source: 'official' },
      enabled: false,
      status: 'disabled',
      error: null
    }
    const manager = {
      list: vi.fn(() => [addon]),
      get: vi.fn(() => addon),
      enable: vi.fn(async(id) => { addon.enabled = true; addon.status = 'enabled'; return { id } }),
      disable: vi.fn(async(id) => { addon.enabled = false; addon.status = 'disabled'; return { id } }),
      runAction: vi.fn(async(id, payload) => ({ id, payload })),
      getActions: vi.fn(() => [{ contribution: { id: 'elephant.test.action' } }]),
      host: {
        list: vi.fn(() => ['test.resource']),
        get: vi.fn(() => resource)
      }
    }
    const target = {
      document: document.implementation.createHTMLDocument('acceptance-addons'),
      console,
      __ELEPHANT_ADDONS__: manager,
      __TAURI__: { core: { invoke: vi.fn(async(command, payload) => { calls.push({ command, payload }); return null }) } }
    }
    const api = installAcceptanceTestBridge({
      target,
      editorStore: { currentFile: null },
      vaultStore: { activeVault: { path: '/vault' }, entries: [], rootEntries: [] }
    })

    expect(api.addonState()).toMatchObject({
      addons: [{ id: 'elephant.test', enabled: false, status: 'disabled' }],
      resources: ['test.resource'],
      resourceMethods: { 'test.resource': ['status'] },
      actions: ['elephant.test.action']
    })
    await expect(api.enableAddon('elephant.test')).resolves.toEqual({ id: 'elephant.test' })
    await expect(api.runAddonAction('elephant.test.action', { probe: true })).resolves.toEqual({ id: 'elephant.test.action', payload: { probe: true } })
    await expect(api.invokeAddonResource('test.resource', 'status')).resolves.toEqual({ available: true })
    await expect(api.disableAddon('elephant.test')).resolves.toEqual({ id: 'elephant.test' })
    expect(calls.map(({ command }) => command)).toEqual([
      'tauri_addons_set_enabled', 'tauri_addons_set_enabled'
    ])
    expect(api.logs().map((entry) => entry.event)).toEqual(expect.arrayContaining([
      'addons:state', 'addons:enable:done', 'addons:action:done', 'addons:resource:done', 'addons:disable:done'
    ]))
  })

  it('probes native sidecars and services through logged commands', async() => {
    const calls = []
    const target = {
      document: document.implementation.createHTMLDocument('acceptance-native'),
      console,
      __TAURI__: {
        core: {
          invoke: vi.fn(async(command, payload) => {
            calls.push({ command, payload })
            if (command === 'tauri_addons_sidecar_status') return { addonId: payload.addonId, available: true, platform: 'macos-aarch64' }
            if (command === 'tauri_addons_service_status') return { addonId: payload.addonId, running: true, protocol: 'elephant-addon-service-v1' }
            if (command === 'tauri_addons_service_call') return { pairingState: 'not-paired', running: true }
            throw new Error(`Unexpected command: ${command}`)
          })
        }
      }
    }
    const api = installAcceptanceTestBridge({
      target,
      editorStore: { currentFile: null },
      vaultStore: { activeVault: { path: '/vault' }, entries: [], rootEntries: [] }
    })

    await expect(api.addonNativeStatus('elephant.ai-ocr', 'sidecar')).resolves.toMatchObject({ available: true })
    await expect(api.addonNativeStatus('elephant.sync', 'service')).resolves.toMatchObject({ running: true })
    await expect(api.addonNativeCall('elephant.sync', 'sync.status', {}, { service: true })).resolves.toMatchObject({ pairingState: 'not-paired' })
    expect(calls.map(({ command }) => command)).toEqual([
      'tauri_addons_sidecar_status', 'tauri_addons_service_status', 'tauri_addons_service_call'
    ])
    expect(api.logs().map((entry) => entry.event)).toEqual(expect.arrayContaining([
      'addons:native-status', 'addons:native-call:done'
    ]))
  })
})
