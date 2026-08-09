import fs from 'fs-extra'
import os from 'os'
import path from 'path'
import { describe, expect, it, vi } from 'vitest'

const { createSearchLibrary } = await import('main_renderer/elephantnote/search/searchLibrary')

describe('createSearchLibrary', () => {
  it('keeps initialization passive and builds the graph on demand', async() => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), 'elephantnote-search-library-'))
    try {
      await fs.writeFile(path.join(root, 'Alpha.md'), '# Alpha\n\nSemantic graph test note.', 'utf8')
      const embedText = vi.fn(async() => [1, 0, 0])
      const library = createSearchLibrary({
        embeddingProvider: {
          source: 'node-llama-cpp',
          embedText
        }
      })

      await library.registerWindowVault(11, root)
      expect(embedText).not.toHaveBeenCalled()

      const initialInspection = await library.inspectIndex(11)
      expect(initialInspection.documents).toEqual([])
      expect(initialInspection.graph).toMatchObject({ nodes: [], edges: [] })

      await library.search({ query: 'semantic', mode: 'semantic', limit: 5 }, 11)
      expect(embedText).toHaveBeenCalled()

      const rebuiltInspection = await library.inspectIndex(11)
      expect(rebuiltInspection.documents).toHaveLength(1)
      expect(rebuiltInspection.graph.nodes).toEqual(expect.arrayContaining([
        expect.objectContaining({ kind: 'note' })
      ]))
    } finally {
      await fs.remove(root)
    }
  })
})

