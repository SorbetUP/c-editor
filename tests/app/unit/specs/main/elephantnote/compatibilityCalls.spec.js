import { afterEach, describe, expect, it, vi } from 'vitest'
import { COMPATIBILITY_CALLS } from '../../../../../../Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js'

const originalBridge = globalThis.window?.elephantnote

afterEach(() => {
  globalThis.window.elephantnote = originalBridge
})

describe('core COMPATIBILITY_CALLS', () => {
  it('routes generic search through the core bridge', () => {
    const payload = { query: 'plan', mode: 'text', limit: 5 }
    const query = vi.fn((value) => ({ ok: true, value }))
    globalThis.window.elephantnote = { search: { query } }

    expect(COMPATIBILITY_CALLS['search.query'](payload)).toEqual({ ok: true, value: payload })
    expect(query).toHaveBeenCalledWith(payload)
  })

  it('normalizes simple directory payloads for the legacy bridge shape', () => {
    const listDirectory = vi.fn((value) => value)
    globalThis.window.elephantnote = { listDirectory }

    expect(COMPATIBILITY_CALLS['directory.list']({ relativePath: 'Projects' })).toBe('Projects')
    expect(listDirectory).toHaveBeenCalledWith('Projects')
  })

  it('routes feature flags without exposing optional product actions', () => {
    const set = vi.fn((key, enabled) => ({ key, enabled }))
    globalThis.window.elephantnote = { features: { set } }

    expect(COMPATIBILITY_CALLS['features.set']({ key: 'editor.footer', enabled: false }))
      .toEqual({ key: 'editor.footer', enabled: false })
    expect(COMPATIBILITY_CALLS['sync.plan']).toBeUndefined()
    expect(COMPATIBILITY_CALLS['ai.config.set']).toBeUndefined()
    expect(COMPATIBILITY_CALLS['wiki.list']).toBeUndefined()
  })
})
