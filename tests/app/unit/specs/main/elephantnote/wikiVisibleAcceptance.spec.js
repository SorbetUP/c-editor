import { describe, expect, it, vi } from 'vitest'
import ElephantWikiAddon from '../../../../../../addons/official/wiki/main.v2.js'

const draft = {
  id: 'wiki-deep-learning',
  topic: 'Deep learning',
  title: 'Deep Learning',
  slug: 'deep-learning',
  markdown: '# Deep Learning\n\nGenerated and cited.',
  status: 'accepted',
  source_paths: ['Notes/AI.md'],
  citations: [{
    document_path: 'Notes/AI.md',
    document_title: 'AI',
    heading: 'Models',
    chunk_id: 'chunk-1'
  }]
}

const createAddon = () => {
  const acceptWiki = vi.fn(async() => ({
    draft,
    path: '.elephantnote/wiki/deep-learning.md'
  }))
  const api = {
    experimental: { window: { __TAURI__: { core: { invoke: vi.fn() } } } },
    resources: {
      get: vi.fn((id) => id === 'knowledge.provider' ? { acceptWiki } : null),
      has: vi.fn()
    },
    storage: { get: vi.fn(async() => []), set: vi.fn(async() => {}) }
  }
  const addon = new ElephantWikiAddon(api)
  addon.writeNote = vi.fn(async() => ({ ok: true }))
  return { addon, acceptWiki }
}

describe('package-owned Wiki acceptance', () => {
  it('materializes a Knowledge draft under the visible Wiki folder', async() => {
    const { addon, acceptWiki } = createAddon()

    const record = await addon.acceptRecord(draft.id)

    expect(acceptWiki).toHaveBeenCalledWith(draft.id)
    expect(addon.writeNote).toHaveBeenCalledWith('Wiki/deep-learning.md', draft.markdown)
    expect(record).toMatchObject({
      id: draft.id,
      path: 'Wiki/deep-learning.md',
      status: 'accepted',
      sourceCount: 1
    })
  })

  it('normalizes unsafe or accented draft slugs before writing', async() => {
    const { addon } = createAddon()
    addon.api.resources.get = vi.fn(() => ({
      acceptWiki: vi.fn(async() => ({
        draft: { ...draft, slug: '../Réseaux / Neuronaux' }
      }))
    }))

    const record = await addon.acceptRecord(draft.id)

    expect(addon.writeNote).toHaveBeenCalledWith('Wiki/reseaux-neuronaux.md', draft.markdown)
    expect(record.path).toBe('Wiki/reseaux-neuronaux.md')
  })
})
