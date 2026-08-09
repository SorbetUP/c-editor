import { describe, expect, it, vi } from 'vitest'
import { installTauriMarkTextSaveBridge } from '@/platform/tauriMarkTextSaveBridge'

const createTarget = () => {
  const listeners = new Map()
  const sent = []
  const invoke = vi.fn(async(command, payload) => {
    if (command !== 'tauri_marktext_write_file') throw new Error(`unexpected command: ${command}`)
    return { ok: true, changed: true, ...payload }
  })
  const target = {
    __TAURI__: { core: { invoke } },
    path: { join: (...parts) => parts.filter(Boolean).join('/') },
    tauri: {
      ipcRenderer: {
        on: (channel, handler) => listeners.set(channel, handler),
        send: (channel, ...args) => sent.push({ channel, args })
      }
    }
  }
  return { target, invoke, listeners, sent }
}

describe('tauriMarkTextSaveBridge', () => {
  it('writes a normal CmdOrCtrl+S response to disk and acknowledges the tab', async() => {
    const { target, invoke, listeners, sent } = createTarget()
    expect(installTauriMarkTextSaveBridge(target)).toBe(true)

    listeners.get('mt::response-file-save')?.({}, 'tab-1', 'Note.md', '/vault/Note.md', '# Saved')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('tauri_marktext_write_file', {
      pathname: '/vault/Note.md',
      content: '# Saved'
    }))

    await vi.waitFor(() => expect(sent).toContainEqual({ channel: 'mt::tab-saved', args: ['tab-1'] }))
  })

  it('reports a save failure instead of claiming success when the Rust write fails', async() => {
    const { target, invoke, listeners, sent } = createTarget()
    invoke.mockRejectedValueOnce(new Error('permission denied'))
    installTauriMarkTextSaveBridge(target)

    listeners.get('mt::response-file-save')?.({}, 'tab-2', 'Note.md', '/vault/Note.md', '# Not saved')
    await vi.waitFor(() => expect(sent).toContainEqual({
      channel: 'mt::tab-save-failure',
      args: ['tab-2', 'permission denied']
    }))
    expect(sent).not.toContainEqual({ channel: 'mt::tab-saved', args: ['tab-2'] })
  })
})
