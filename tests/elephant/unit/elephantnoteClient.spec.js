import { describe, expect, it, vi } from 'vitest'
import { elephantnoteClient } from '@/elephantnote/services/elephantnoteClient'

describe('renderer ElephantNote API client', () => {
  it('unwraps and normalizes successful API envelopes', async() => {
    window.elephantnote = {
      api: {
        call: vi.fn(async() => ({ ok: true, data: { path: 'Note.md' } }))
      }
    }

    await expect(elephantnoteClient.notes.create('Inbox')).resolves.toEqual({
      note: { path: 'Note.md' },
      entries: []
    })
    expect(window.elephantnote.api.call).toHaveBeenCalledWith('notes.create', { relativePath: 'Inbox' })
  })

  it('throws typed errors for failed envelopes', async() => {
    window.elephantnote = {
      api: {
        call: vi.fn(async() => ({
          ok: false,
          error: {
            code: 'BAD_PAYLOAD',
            message: 'Invalid note'
          }
        }))
      }
    }

    await expect(elephantnoteClient.notes.create()).rejects.toMatchObject({
      code: 'BAD_PAYLOAD',
      message: 'Invalid note'
    })
  })
})
