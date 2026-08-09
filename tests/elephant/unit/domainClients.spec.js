import { describe, expect, it, vi } from 'vitest'
import { ELEPHANTNOTE_API_ACTIONS as API } from 'common/elephantnote/apiActions'
import { createDomainClients } from '@/elephantnote/services/elephantnoteClient/domainClients'

describe('minimal core domain client serialization', () => {
  it('normalizes string document arguments', async() => {
    const call = vi.fn(async(_action, payload) => payload)
    const clients = createDomainClients(call, () => ({}))

    await expect(clients.directory.list('Projects')).resolves.toEqual({ relativePath: 'Projects' })
    expect(call).toHaveBeenCalledWith(API.DIRECTORY_LIST, { relativePath: 'Projects' })

    await expect(clients.notes.read('Projects/Plan.md')).resolves.toEqual({ relativePath: 'Projects/Plan.md' })
    expect(call).toHaveBeenCalledWith(API.NOTES_READ, { relativePath: 'Projects/Plan.md' })
  })

  it('serializes vault, search and feature mutations', async() => {
    const call = vi.fn(async(_action, payload) => payload)
    const clients = createDomainClients(call, () => ({}))

    await clients.vaults.setName('vault-1', 'Work')
    expect(call).toHaveBeenCalledWith(API.VAULTS_SET_NAME, { vaultId: 'vault-1', name: 'Work' })

    await clients.search.query({ query: 'architecture', mode: 'text', limit: 10 })
    expect(call).toHaveBeenCalledWith(API.SEARCH_QUERY, { query: 'architecture', mode: 'text', limit: 10 })

    await clients.features.set('editor.footer', false)
    expect(call).toHaveBeenCalledWith(API.FEATURES_SET, { key: 'editor.footer', enabled: false })
  })

  it('does not expose optional product clients globally', () => {
    const clients = createDomainClients(vi.fn(), () => ({}))

    for (const domain of ['ai', 'wiki', 'sync', 'calendar', 'models', 'ocr', 'rag', 'plugins', 'tasks']) {
      expect(clients[domain]).toBeUndefined()
    }
  })
})
