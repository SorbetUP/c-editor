import { describe, expect, it, vi } from 'vitest'
import { createApiCaller } from '@/elephantnote/services/elephantnoteClient/apiRuntime'

describe('ipc api runtime serialization', () => {
  it('serializes payloads before sending them to the bridge', async () => {
    const call = vi.fn(async () => ({ ok: true, data: { done: true } }))
    const caller = createApiCaller({
      'example.action': () => null
    })

    window.elephantnote = {
      api: {
        call
      }
    }

    const payload = new Proxy(
      {
        nested: { value: 1 },
        list: [1, 2, 3]
      },
      {}
    )

    await expect(caller('example.action', payload)).resolves.toEqual({ done: true })
    expect(call).toHaveBeenCalledWith('example.action', {
      nested: { value: 1 },
      list: [1, 2, 3]
    })
  })
})
