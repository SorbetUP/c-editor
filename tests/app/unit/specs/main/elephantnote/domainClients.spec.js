import { describe, expect, it } from 'vitest'
import { createDomainClients } from 'elephant-front/services/elephantnoteClient/domainClients'
import { ELEPHANTNOTE_API_ACTIONS as API } from 'common/elephantnote/apiActions'

const makeClient = () => {
  const calls = []
  const call = async(action, payload = {}) => {
    calls.push({ action, payload })
    return { action, payload }
  }
  return { calls, clients: createDomainClients(call, () => ({})) }
}

describe('minimal core domain clients', () => {
  it('routes text search through the public core API action', async() => {
    const { calls, clients } = makeClient()
    const payload = { query: 'plan', mode: 'text', limit: 8 }

    await clients.search.query(payload)

    expect(API.SEARCH_QUERY).toBe('search.query')
    expect(calls).toEqual([{ action: API.SEARCH_QUERY, payload }])
  })

  it('does not expose Sync planning as a core client', () => {
    const { clients } = makeClient()
    expect(clients.sync).toBeUndefined()
    expect(API.SYNC_PLAN).toBeUndefined()
  })
})
