import { describe, expect, it, vi } from 'vitest'
import { installTauriSearchLifecycleBridge } from '../../../Elephant/frontend/src/renderer/src/platform/tauriSearchLifecycleBridge.js'

describe('Tauri search lifecycle bridge', () => {
  it('returns an empty inspection when the optional search addon is unavailable', async() => {
    const target = {
      __TAURI__: {},
      console: { info: vi.fn() },
      elephantnote: { search: {} }
    }
    const client = {
      search: {
        status: vi.fn(async() => ({ status: 'ready', indexedDocuments: 0 }))
      }
    }

    expect(installTauriSearchLifecycleBridge({ target, client })).toBe(true)
    await expect(target.elephantnote.search.inspect()).resolves.toMatchObject({
      documents: [],
      folders: [],
      semanticLinks: [],
      graph: null
    })
    expect(target.console.info).toHaveBeenCalledWith('[tauri-search] inspect:unavailable', { reason: 'optional-search-addon-missing' })
  })
})
