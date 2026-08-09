/* @vitest-environment node */

import {
  createElephantNoteApi,
  ELEPHANTNOTE_API_DOMAINS,
  ELEPHANTNOTE_API_ACTIONS,
  ELEPHANTNOTE_API_VERSION
} from 'main_renderer/elephantnote/api'
import { API_PAYLOAD_SCHEMAS, listApiContracts } from 'common/elephantnote/apiContracts'
import { describe, expect, it, vi } from 'vitest'

const OPTIONAL_ACTION_KEYS = [
  'AI_CONFIG_SET',
  'MODEL_SELECTION_SET',
  'OCR_EXTRACT',
  'CALENDAR_IMPORT_GOOGLE_FROM_PATH',
  'WIKI_ACCEPT',
  'PLUGINS_RUN',
  'PROGRAMS_RUN',
  'TASKS_RUN',
  'RAG_CHAT',
  'SYNC_PLAN'
]

describe('ElephantNote minimal core API contract', () => {
  it('describes only versioned core actions', () => {
    const api = createElephantNoteApi({
      handlers: {
        [ELEPHANTNOTE_API_ACTIONS.NOTES_CREATE]: async() => ({ ok: true })
      }
    })

    expect(api.version).toBe(ELEPHANTNOTE_API_VERSION)
    expect(api.describe()).toEqual({
      version: ELEPHANTNOTE_API_VERSION,
      actions: [ELEPHANTNOTE_API_ACTIONS.API_DESCRIBE, ELEPHANTNOTE_API_ACTIONS.NOTES_CREATE].sort()
    })
  })

  it('keeps domains, constants and schemas synchronized', () => {
    const contracts = listApiContracts()
    const actionNames = new Set(Object.values(ELEPHANTNOTE_API_ACTIONS))

    expect(Object.keys(ELEPHANTNOTE_API_DOMAINS)).toEqual([
      'system',
      'vaults',
      'documents',
      'search',
      'coreFeatures'
    ])
    expect(contracts).toHaveLength(actionNames.size)
    expect(Object.keys(API_PAYLOAD_SCHEMAS).sort()).toEqual([...actionNames].sort())
  })

  it('calls registered handlers with validated payload and context', async() => {
    const handler = vi.fn(async(payload, context) => ({ payload, actor: context.actor }))
    const api = createElephantNoteApi({
      handlers: { [ELEPHANTNOTE_API_ACTIONS.NOTES_WRITE]: handler }
    })

    await expect(api.call(
      ELEPHANTNOTE_API_ACTIONS.NOTES_WRITE,
      { relativePath: 'Inbox/Note.md', markdown: '# Note' },
      { actor: 'test' }
    )).resolves.toEqual({
      payload: { relativePath: 'Inbox/Note.md', markdown: '# Note' },
      actor: 'test'
    })
    expect(handler).toHaveBeenCalledOnce()
  })

  it('returns stable success and error envelopes', async() => {
    const api = createElephantNoteApi({ handlers: { 'notes.ok': async() => 42 } })

    await expect(api.callEnvelope('notes.ok')).resolves.toMatchObject({
      ok: true,
      version: ELEPHANTNOTE_API_VERSION,
      action: 'notes.ok',
      data: 42
    })
    await expect(api.callEnvelope('missing.action')).resolves.toMatchObject({
      ok: false,
      error: { code: 'ELEPHANTNOTE_UNKNOWN_API_ACTION' }
    })
  })

  it('rejects invalid document and search payloads before handlers run', async() => {
    const noteHandler = vi.fn(async(payload) => payload)
    const searchHandler = vi.fn(async(payload) => payload)
    const api = createElephantNoteApi({
      handlers: {
        [ELEPHANTNOTE_API_ACTIONS.NOTES_CREATE]: noteHandler,
        [ELEPHANTNOTE_API_ACTIONS.SEARCH_QUERY]: searchHandler
      }
    })

    await expect(api.callEnvelope(ELEPHANTNOTE_API_ACTIONS.NOTES_CREATE, {
      filename: '../escape.md'
    })).resolves.toMatchObject({
      ok: false,
      error: { code: 'ELEPHANTNOTE_INVALID_API_PAYLOAD' }
    })
    await expect(api.callEnvelope(ELEPHANTNOTE_API_ACTIONS.SEARCH_QUERY, {
      query: '',
      mode: 'semantic'
    })).resolves.toMatchObject({
      ok: false,
      error: { code: 'ELEPHANTNOTE_INVALID_API_PAYLOAD' }
    })
    expect(noteHandler).not.toHaveBeenCalled()
    expect(searchHandler).not.toHaveBeenCalled()
  })

  it('accepts the generic core search and feature payloads', async() => {
    const api = createElephantNoteApi({
      handlers: {
        [ELEPHANTNOTE_API_ACTIONS.SEARCH_QUERY]: async(payload) => payload,
        [ELEPHANTNOTE_API_ACTIONS.FEATURES_SET]: async(payload) => payload
      }
    })

    await expect(api.call(ELEPHANTNOTE_API_ACTIONS.SEARCH_QUERY, {
      query: 'elephant',
      mode: 'text',
      limit: 20
    })).resolves.toEqual({ query: 'elephant', mode: 'text', limit: 20 })
    await expect(api.call(ELEPHANTNOTE_API_ACTIONS.FEATURES_SET, {
      key: 'editor.footer',
      enabled: false
    })).resolves.toEqual({ key: 'editor.footer', enabled: false })
  })

  it('does not advertise optional package actions globally', async() => {
    for (const key of OPTIONAL_ACTION_KEYS) expect(ELEPHANTNOTE_API_ACTIONS[key]).toBeUndefined()

    const api = createElephantNoteApi({ handlers: {} })
    await expect(api.callEnvelope('sync.plan', {})).resolves.toMatchObject({
      ok: false,
      error: { code: 'ELEPHANTNOTE_UNKNOWN_API_ACTION' }
    })
    await expect(api.callEnvelope('ai.config.set', {})).resolves.toMatchObject({
      ok: false,
      error: { code: 'ELEPHANTNOTE_UNKNOWN_API_ACTION' }
    })
  })
})
