import { describe, expect, it, vi } from 'vitest'

vi.mock('@electron-toolkit/preload', () => ({
  electronAPI: {
    ipcRenderer: {
      invoke: vi.fn()
    }
  }
}))

import { createElephantNoteAPI } from '../../../Elephant/frontend/src/preload/elephantnoteApi.js'
import { requireAtomicFeatureApi } from '@/elephantnote/services/elephantnoteClient/atomicFeatureApi'

describe('preload api serialization', () => {
  it('serializes ElephantNote API payloads before invoking IPC', async() => {
    const invoke = vi.fn(async(_channel, payload) => payload)
    const api = createElephantNoteAPI({ invoke })
    const payload = new Proxy(
      {
        nested: { value: 1 },
        list: [1, 2, 3]
      },
      {}
    )

    await expect(api.api.call('example.action', payload)).resolves.toEqual({
      action: 'example.action',
      payload: {
        nested: { value: 1 },
        list: [1, 2, 3]
      }
    })
    expect(invoke).toHaveBeenCalledWith('elephantnote:api:call', {
      action: 'example.action',
      payload: {
        nested: { value: 1 },
        list: [1, 2, 3]
      }
    })
  })

  it('serializes atomic feature payloads before invoking IPC', async() => {
    const invoke = vi.fn(async(_channel, payload) => payload)
    window.tauri = {
      ipcRenderer: {
        invoke
      }
    }

    const atomicApi = requireAtomicFeatureApi()
    const payload = new Proxy(
      {
        vaultRoot: '/tmp/vault',
        options: {
          deep: true
        }
      },
      {}
    )

    await expect(atomicApi.callApi(payload)).resolves.toEqual({
      vaultRoot: '/tmp/vault',
      options: {
        deep: true
      }
    })
    expect(invoke).toHaveBeenCalledWith('en:atomic:api:call', {
      vaultRoot: '/tmp/vault',
      options: {
        deep: true
      }
    })
  })

  it('serializes model bridge payloads before invoking IPC', async() => {
    const invoke = vi.fn(async(_channel, payload) => payload)
    const api = createElephantNoteAPI({ invoke })

    const payload = new Proxy(
      {
        downloadId: 'download-123',
        nested: {
          active: true
        }
      },
      {}
    )

    await expect(api.models.downloadStatus(payload)).resolves.toEqual({
      downloadId: 'download-123',
      nested: {
        active: true
      }
    })
    expect(invoke).toHaveBeenCalledWith('elephantnote:models:download-status', {
      downloadId: 'download-123',
      nested: {
        active: true
      }
    })
  })
})
