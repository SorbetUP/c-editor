import { describe, expect, it } from 'vitest'
import {
  API_PAYLOAD_SCHEMAS,
  ELEPHANTNOTE_API_ACTIONS,
  validateApiPayload
} from 'common/elephantnote/apiContracts'

describe('ElephantNote core API contracts', () => {
  it('validates core note writes and text search', () => {
    const note = { relativePath: 'Notes/A.md', markdown: '# A' }
    const search = { query: 'alpha', mode: 'text', limit: 20 }
    expect(validateApiPayload('notes.write', note)).toBe(note)
    expect(validateApiPayload('search.query', search)).toBe(search)
  })

  it('rejects invalid core payload fields', () => {
    expect(() => validateApiPayload('notes.write', { relativePath: '', markdown: '# A' })).toThrow(/relativePath/)
    expect(() => validateApiPayload('search.query', { query: '', mode: 'semantic' })).toThrow(/query|mode/)
  })

  it('does not publish optional addon actions in the core registry', () => {
    const optionalActions = [
      'sync.plan',
      'sync.run',
      'ai.config.set',
      'models.download',
      'ocr.extract',
      'search.inspect',
      'wiki.propose',
      'graph.rebuild'
    ]
    const registered = new Set(Object.values(ELEPHANTNOTE_API_ACTIONS))
    for (const action of optionalActions) {
      expect(registered.has(action)).toBe(false)
      expect(API_PAYLOAD_SCHEMAS).not.toHaveProperty(action)
    }
  })

  it('leaves addon-specific validation to the installed package API', () => {
    const payload = { operations: ['init', 'pull'] }
    expect(validateApiPayload('sync.plan', payload)).toBe(payload)
    expect(validateApiPayload('ai.config.set', { provider: 'openrouter' })).toEqual({ provider: 'openrouter' })
  })
})
