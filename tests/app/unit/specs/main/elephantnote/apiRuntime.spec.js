import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApiCaller } from '../../../../../../Elephant/frontend/app/services/elephantnoteClient/apiRuntime.js'

const originalBridge = globalThis.window?.elephantnote

afterEach(() => {
  globalThis.window.elephantnote = originalBridge
})

describe('ElephantNote core API runtime', () => {
  it('validates payloads before dispatching to the public bridge API', async() => {
    const bridgeCall = vi.fn(async() => ({ ok: true, data: ['Plan.md'] }))
    globalThis.window.elephantnote = { api: { call: bridgeCall } }
    const call = createApiCaller({})

    await expect(call('search.query', { query: 'plan', mode: 'text', limit: 5 }))
      .resolves.toEqual(['Plan.md'])
    expect(bridgeCall).toHaveBeenCalledWith('search.query', {
      query: 'plan',
      mode: 'text',
      limit: 5
    })
  })

  it('rejects invalid core payloads before they reach the public bridge API', () => {
    const bridgeCall = vi.fn()
    globalThis.window.elephantnote = { api: { call: bridgeCall } }
    const call = createApiCaller({})

    expect(() => call('search.query', { query: '', mode: 'semantic' })).toThrow(/query|mode/)
    expect(bridgeCall).not.toHaveBeenCalled()
  })

  it('rejects invalid core payloads before local fallback calls', () => {
    const fallbackDelete = vi.fn()
    globalThis.window.elephantnote = null
    const call = createApiCaller({ 'entries.delete': fallbackDelete })

    expect(() => call('entries.delete', {})).toThrow(/relativePath/)
    expect(fallbackDelete).not.toHaveBeenCalled()
  })

  it('does not invent a local fallback for optional package actions', () => {
    globalThis.window.elephantnote = null
    const call = createApiCaller({})

    expect(() => call('sync.plan', {})).toThrow(/not available for action: sync\.plan/)
  })
})
