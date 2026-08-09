import { describe, expect, it, vi } from 'vitest'
import { installTauriLocalIpcBridge } from '@/platform/tauriLocalIpcBridge'

const flushPromises = () => new Promise((resolve) => setTimeout(resolve, 0))

const createTarget = ({ vaultRoot = '/Users/sorbet/Documents/New project 2', notePayload = { markdown: 'Hello from disk' } } = {}) => {
  const target = new EventTarget()
  const nativeSend = vi.fn()
  const read = vi.fn(async() => notePayload)

  target.__TAURI__ = { core: { invoke: vi.fn() } }
  target.tauri = { ipcRenderer: { send: nativeSend } }
  target.elephantnote = {
    getVaults: vi.fn(async() => ({ activeVault: { path: vaultRoot } })),
    notes: { read }
  }

  return { target, nativeSend, read }
}

describe('Tauri local IPC bridge', () => {
  it('opens a vault note through notes.read even when target.path is unavailable', async() => {
    const { target, nativeSend, read } = createTarget()
    const opened = []
    target.addEventListener('mt::open-new-tab', (event) => opened.push(event.detail))

    expect(installTauriLocalIpcBridge(target)).toBe(true)
    target.tauri.ipcRenderer.send('mt::open-file', '/Users/sorbet/Documents/New project 2/test_keep/Note.md', {})
    await flushPromises()

    expect(nativeSend).not.toHaveBeenCalled()
    expect(read).toHaveBeenCalledWith({ relativePath: 'test_keep/Note.md' })
    expect(opened).toHaveLength(1)
    expect(opened[0][0]).toMatchObject({
      pathname: '/Users/sorbet/Documents/New project 2/test_keep/Note.md',
      filename: 'Note.md',
      markdown: 'Hello from disk',
      isMixedLineEndings: false
    })
  })

  it('falls back to native IPC for files outside the active vault', async() => {
    const { target, nativeSend, read } = createTarget({ vaultRoot: '/Users/sorbet/Documents/Vault' })
    const opened = []
    target.addEventListener('mt::open-new-tab', (event) => opened.push(event.detail))

    installTauriLocalIpcBridge(target)
    target.tauri.ipcRenderer.send('mt::open-file', '/Users/sorbet/Documents/Other/Note.md', { flag: true })
    await flushPromises()

    expect(read).not.toHaveBeenCalled()
    expect(opened).toHaveLength(0)
    expect(nativeSend).toHaveBeenCalledWith('mt::open-file', '/Users/sorbet/Documents/Other/Note.md', { flag: true })
  })

  it('keeps renderer-local save events local and does not call native IPC', () => {
    const { target, nativeSend } = createTarget()
    const dispatched = []
    target.addEventListener('mt::response-file-save', (event) => dispatched.push(event.detail))

    installTauriLocalIpcBridge(target)
    target.tauri.ipcRenderer.send('mt::response-file-save', 'tab-1', 'Note.md', '/vault/Note.md', 'body')

    expect(nativeSend).not.toHaveBeenCalled()
    expect(dispatched).toEqual([['tab-1', 'Note.md', '/vault/Note.md', 'body']])
  })
})
