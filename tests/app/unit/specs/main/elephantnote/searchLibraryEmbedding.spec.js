import fs from 'fs-extra'
import os from 'os'
import path from 'path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createSearchLibrary } from '../../../../../../Elephant/backend/js/search/searchLibrary.js'
import { SEARCH_MODES } from '../../../../../../Elephant/backend/js/search/searchTypes.js'

const writeNote = async (root, relativePath, markdown) => {
  const absolutePath = path.join(root, relativePath)
  await fs.ensureDir(path.dirname(absolutePath))
  await fs.writeFile(absolutePath, markdown, 'utf8')
}

describe('Search library embedding provider integration', () => {
  let root

  beforeEach(async () => {
    root = await fs.mkdtemp(path.join(os.tmpdir(), 'elephant-search-'))
    await writeNote(root, 'ai/neural-index.md', '# Neural Index\nEmbeddings and model vectors power semantic graph search. #ai')
    await writeNote(root, 'food/recipe.md', '# Recipe\nTomato pasta, garlic, olive oil and cooking timing. #food')
  })

  afterEach(async () => {
    await fs.remove(root)
  })

  it('builds an index with a runtime embedding model and uses it for semantic queries', async () => {
    const calls = []
    const provider = {
      source: 'node-llama-cpp',
      async embedText(text) {
        calls.push(text)
        const lower = String(text || '').toLowerCase()
        if (lower.includes('embedding') || lower.includes('model') || lower.includes('neural')) return new Float32Array([1, 0, 0, 0])
        if (lower.includes('recipe') || lower.includes('tomato') || lower.includes('cooking')) return { embedding: new Float32Array([0, 1, 0, 0]) }
        return [0, 0, 1, 0]
      }
    }

    const library = createSearchLibrary({ embeddingProvider: provider })
    await library.registerWindowVault(42, root)
    const status = await library.rebuildIndex(42)
    const inspection = await library.inspectIndex(42)
    const results = await library.search({ query: 'model embeddings', mode: SEARCH_MODES.SEMANTIC, limit: 2 }, 42)

    expect(status.status).toBe('ready')
    expect(inspection.features.embeddingSource).toBe('node-llama-cpp')
    expect(inspection.documents).toHaveLength(2)
    expect(inspection.graph.nodes.some((node) => node.id === 'ai/neural-index.md')).toBe(true)
    expect(results[0].relativePath).toBe('ai/neural-index.md')
    expect(calls.length).toBeGreaterThanOrEqual(3)
  })

  it('falls back to deterministic embeddings when the runtime provider throws', async () => {
    const library = createSearchLibrary({
      embeddingProvider: {
        source: 'node-llama-cpp',
        async embedText() {
          throw new Error('model unavailable')
        }
      }
    })

    await library.registerWindowVault(7, root)
    const status = await library.rebuildIndex(7)
    const results = await library.search({ query: 'tomato recipe', mode: SEARCH_MODES.SEMANTIC, limit: 1 }, 7)

    expect(status.status).toBe('ready')
    expect(results[0].relativePath).toBe('food/recipe.md')
  })
})
