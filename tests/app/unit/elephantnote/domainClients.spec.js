import { describe, expect, it, vi } from 'vitest'
import { ELEPHANTNOTE_API_ACTIONS as API } from 'common/elephantnote/apiActions'
import { createDomainClients } from '../../../../Elephant/frontend/app/services/elephantnoteClient/domainClients.js'

const createClients = (call = vi.fn()) => createDomainClients(call, () => ({
  describeApi: vi.fn(async() => ({ runtime: 'tauri' })),
  callApi: vi.fn(async() => ({ ok: true })),
  providers: vi.fn(async() => [])
}))

describe('minimal core domain clients', () => {
  it('does not expose chat, RAG or semantic initialization globally', () => {
    const clients = createClients()

    expect(clients.rag).toBeUndefined()
    expect(clients.chat).toBeUndefined()
    expect(clients.ai).toBeUndefined()
    expect(clients.search.initVault).toBeUndefined()
    expect(clients.search.rebuild).toBeUndefined()
    expect(API.RAG_CHAT).toBeUndefined()
    expect(API.SEARCH_INIT_VAULT).toBeUndefined()
    expect(API.SEARCH_REBUILD).toBeUndefined()
  })

  it('keeps generic text search available through the core action', async() => {
    const call = vi.fn(async(_action, payload) => payload)
    const clients = createClients(call)

    await expect(clients.search.query({ query: 'semantic graph', mode: 'text', limit: 6 }))
      .resolves.toEqual({ query: 'semantic graph', mode: 'text', limit: 6 })
    expect(call).toHaveBeenCalledWith(API.SEARCH_QUERY, {
      query: 'semantic graph',
      mode: 'text',
      limit: 6
    })
  })

  it('delegates generic atomic calls without embedding product behavior', async() => {
    const atomic = {
      describeApi: vi.fn(async() => ({ runtime: 'tauri' })),
      callApi: vi.fn(async(request) => request),
      providers: vi.fn(async() => [])
    }
    const clients = createDomainClients(vi.fn(), () => atomic)

    await expect(clients.atomicFeatures.callApi('list', [])).resolves.toEqual({
      action: 'list',
      arguments: []
    })
    expect(atomic.callApi).toHaveBeenCalledWith({ action: 'list', arguments: [] })
  })
})
