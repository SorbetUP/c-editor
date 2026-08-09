import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import {
  parseMarkdownTags,
  updateMarkdownTags
} from '../../../Elephant/shared/markdownDocument.js'

vi.mock('electron-log', () => ({
  default: { info: vi.fn(), warn: vi.fn(), error: vi.fn(), debug: vi.fn() }
}))

vi.mock('../../../Elephant/frontend/app/services/elephantnoteClient.js', () => ({
  elephantnoteClient: {
    search: {
      initVault: vi.fn(async(vaultPath) => ({ status: 'ready', vaultPath, indexedDocuments: 0, totalDocuments: 0 })),
      query: vi.fn(async() => []),
      status: vi.fn(async() => ({ status: 'ready', vaultPath: '/vault', indexedDocuments: 0, totalDocuments: 0 })),
      inspect: vi.fn(async() => ({ documents: [], folders: [], semanticLinks: [], graph: null, generatedAt: '' })),
      rebuild: vi.fn(async() => ({ status: 'indexing' })),
      clear: vi.fn(async() => ({ status: 'cleared' })),
      disable: vi.fn(async() => ({ status: 'disabled' })),
      enable: vi.fn(async() => ({ status: 'ready' }))
    }
  }
}))

beforeEach(() => {
  setActivePinia(createPinia())
  window.localStorage.clear()
  window.path = { join: (...parts) => parts.join('/') }
  window.tauri = { ipcRenderer: { send: vi.fn() } }
})

describe('editor runtime regression build/coverage', () => {
  it('updates markdown tags when the UI sends a single tag string', () => {
    const markdown = ['---', 'title: "Alpha"', 'type: "note"', 'tags: []', '---', '', '# Alpha', '', 'Body'].join('\n')
    const next = updateMarkdownTags(markdown, 'urgent', 'Alpha')
    expect(parseMarkdownTags(next)).toEqual(['urgent'])
  })

  it('updates markdown tags when the UI sends an array of tags', () => {
    const markdown = ['---', 'title: "Alpha"', 'type: "note"', 'tags: ["old"]', '---', '', '# Alpha', '', 'Body'].join('\n')
    const next = updateMarkdownTags(markdown, ['old', 'new', 'new'], 'Alpha')
    expect(parseMarkdownTags(next)).toEqual(['old', 'new'])
  })

  it('updates markdown tags when the UI sends a set of tags', () => {
    const markdown = ['---', 'title: "Alpha"', 'type: "note"', 'tags: []', '---', '', '# Alpha', '', 'Body'].join('\n')
    const next = updateMarkdownTags(markdown, new Set(['a', 'b', 'a']), 'Alpha')
    expect(parseMarkdownTags(next)).toEqual(['a', 'b'])
  })

  it('does not crash on invalid tag payloads', () => {
    const markdown = '# Alpha\n\nBody'
    expect(parseMarkdownTags(updateMarkdownTags(markdown, null, 'Alpha'))).toEqual([])
    expect(parseMarkdownTags(updateMarkdownTags(markdown, undefined, 'Alpha'))).toEqual([])
    expect(parseMarkdownTags(updateMarkdownTags(markdown, { bad: true }, 'Alpha'))).toEqual([])
  })

  it('searchStore.updateNoteIndex indexes edited markdown locally', async() => {
    const { useSearchStore } = await import('../../../Elephant/frontend/app/stores/searchStore.js')
    const store = useSearchStore()
    store.vaultPath = '/vault'
    const ok = store.updateNoteIndex('Alpha.md', ['---', 'title: "Alpha"', 'type: "note"', 'tags: ["work"]', '---', '', '# Alpha', '', 'This is edited body text'].join('\n'))
    expect(ok).toBe(true)
    expect(store.indexInspection.documents).toHaveLength(1)
    expect(store.indexInspection.documents[0].title).toBe('Alpha')
    expect(store.indexInspection.documents[0].tags).toEqual(['work'])
    store.setQuery('edited body')
    expect(store.localSearch()).toHaveLength(1)
    expect(store.localSearch()[0].relativePath).toBe('Alpha.md')
  })

  it('searchStore.search falls back to local index when backend returns no result', async() => {
    const { useSearchStore } = await import('../../../Elephant/frontend/app/stores/searchStore.js')
    const store = useSearchStore()
    store.vaultPath = '/vault'
    store.status = { status: 'ready', vaultPath: '/vault', indexedDocuments: 1, totalDocuments: 1 }
    store.updateNoteIndex('Alpha.md', '# Alpha\n\nNeedle in local body')
    store.setQuery('Needle')
    const results = await store.search()
    expect(results).toHaveLength(1)
    expect(results[0].relativePath).toBe('Alpha.md')
  })
})
