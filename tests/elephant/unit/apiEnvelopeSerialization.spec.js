import { describe, expect, it } from 'vitest'

const { createElephantNoteApi } = await import('main_renderer/elephantnote/api')

describe('ElephantNote API envelope serialization', () => {
  it('returns clone-safe response data from handlers', async () => {
    const api = createElephantNoteApi({
      handlers: {
        'example.cloneable': async () =>
          new Proxy(
            {
              nested: { value: 1 },
              list: [1, 2, 3]
            },
            {}
          )
      }
    })

    const response = await api.callEnvelope('example.cloneable', {})

    expect(response.ok).toBe(true)
    expect(response.data).toEqual({
      nested: { value: 1 },
      list: [1, 2, 3]
    })
    expect(() => structuredClone(response)).not.toThrow()
  })
})
