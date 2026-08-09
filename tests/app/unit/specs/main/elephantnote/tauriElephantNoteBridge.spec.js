import { describe, expect, it } from 'vitest'
import { installTauriElephantNoteBridge } from '@/platform/tauriElephantNoteBridge.js'

const createTarget = () => {
  const calls = []
  const target = {
    __TAURI__: {
      core: {
        invoke: async(command, payload = {}) => {
          calls.push({ command, payload })
          return { command, payload }
        }
      }
    }
  }
  return { target, calls }
}

describe('tauriElephantNoteBridge minimal core surface', () => {
  it('does not expose direct Sync or optional product namespaces', () => {
    const { target } = createTarget()

    expect(installTauriElephantNoteBridge(target)).toBe(true)
    expect(target.elephantnote.sync).toBeUndefined()
    expect(target.elephantnote.ai).toBeUndefined()
    expect(target.elephantnote.wiki).toBeUndefined()
    expect(target.elephantnote.models).toBeUndefined()
  })

  it('rejects optional public API actions instead of routing to removed commands', async() => {
    const { target, calls } = createTarget()

    expect(installTauriElephantNoteBridge(target)).toBe(true)
    await expect(target.elephantnote.api.call('sync.plan', {}))
      .rejects.toThrow(/Unsupported Elephant core API action: sync\.plan/)
    expect(calls).toEqual([])
  })

  it('keeps paged directory.list options when routing through the public API', async() => {
    const { target, calls } = createTarget()
    const payload = { relativePath: 'Projects', offset: 120, limit: 121, includePreview: false }

    expect(installTauriElephantNoteBridge(target)).toBe(true)
    await target.elephantnote.api.call('directory.list', payload)

    expect(calls).toEqual([
      { command: 'tauri_directory_list', payload }
    ])
  })

  it('routes generic text search to the Rust vault command', async() => {
    const { target, calls } = createTarget()
    const payload = { query: 'architecture', mode: 'text', limit: 20 }

    installTauriElephantNoteBridge(target)
    await target.elephantnote.api.call('search.query', payload)

    expect(calls).toEqual([
      { command: 'tauri_search_query', payload: { params: payload } }
    ])
  })
})
