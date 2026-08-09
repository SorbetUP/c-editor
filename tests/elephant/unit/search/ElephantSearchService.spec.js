import fs from 'fs-extra'
import os from 'os'
import path from 'path'
import { describe, expect, it, vi } from 'vitest'

const { ElephantSearchService } = await import('main_renderer/elephantnote/search/ElephantSearchService')

describe('ElephantSearchService exact local search', () => {
  it('searches small vaults without loading the semantic model', async() => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), 'elephantnote-search-service-'))
    try {
      await fs.writeFile(path.join(root, 'One.md'), '# Alpha\n\nLocal keyword only', 'utf8')
      const embedText = vi.fn(async() => [1, 0, 0])
      const service = new ElephantSearchService()
      service.setEmbeddingProvider({
        source: 'node-llama-cpp',
        embedText
      })
      await service.registerWindowVault(1, root)
      expect(embedText).not.toHaveBeenCalled()

      const results = await service.search({ query: 'keyword', mode: 'exact', limit: 5 }, 1)

      expect(results).toHaveLength(1)
      expect(results[0].relativePath).toBe('One.md')
      expect(embedText).not.toHaveBeenCalled()
    } finally {
      await fs.remove(root)
    }
  })

  it('keeps inspection passive until a semantic build is requested', async() => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), 'elephantnote-search-inspect-'))
    try {
      await fs.ensureDir(path.join(root, 'Project'))
      await fs.writeFile(
        path.join(root, 'Project', 'Plan.md'),
        '# Plan\n\nLocal AI embeddings connect notes into an embedding graph. Source: https://example.com/plan',
        'utf8'
      )
      await fs.writeFile(
        path.join(root, 'Project', 'Graph.md'),
        '# Graph\n\nLocal AI embedding links create semantic graph edges between connected notes.',
        'utf8'
      )
      const embedText = vi.fn(async() => [1, 0, 0])
      const service = new ElephantSearchService()
      service.setEmbeddingProvider({
        source: 'node-llama-cpp',
        embedText
      })
      await service.registerWindowVault(2, root)

      const inspection = await service.inspectIndex(2)

      expect(inspection.indexPath).toBe('')
      expect(inspection.status.status).toBe('not_initialized')
      expect(inspection.documents).toEqual([])
      expect(inspection.folders).toEqual([])
      expect(inspection.semanticLinks).toEqual([])
      expect(inspection.graph).toMatchObject({
        nodes: [],
        edges: []
      })
      expect(inspection.features).toMatchObject({
        embeddings: true,
        semanticLinks: true,
        automaticSources: true
      })
      expect(embedText).not.toHaveBeenCalled()

      await service.rebuildIndex(2)
      const rebuiltInspection = await service.inspectIndex(2)

      expect(rebuiltInspection.documents).toHaveLength(2)
      expect(rebuiltInspection.documents[0]).toMatchObject({
        relativePath: 'Project/Graph.md',
        chunkCount: 1
      })
      expect(rebuiltInspection.documents.some((document) => document.sourceCount === 1)).toBe(true)
      expect(rebuiltInspection.semanticLinks).toEqual(expect.arrayContaining([
        expect.objectContaining({
          reason: 'embedding-similarity'
        })
      ]))
      expect(rebuiltInspection.graph.nodes).toEqual(expect.arrayContaining([
        expect.objectContaining({ kind: 'folder' }),
        expect.objectContaining({ kind: 'note' })
      ]))
      expect(embedText).toHaveBeenCalled()
    } finally {
      await fs.remove(root)
    }
  })

  it('uses local embeddings for semantic search before external models are configured', async() => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), 'elephantnote-meaning-search-'))
    try {
      await fs.writeFile(
        path.join(root, 'Agents.md'),
        '---\ntitle: "Agents"\ntags: ["ai"]\n---\n\n# Agents\n\nConnect notes to local models and tool calls.',
        'utf8'
      )
      const embedText = vi.fn(async(text) => {
        return String(text).toLowerCase().includes('node-llama-cpp') ||
          String(text).toLowerCase().includes('llama')
          ? [1, 0, 0]
          : [0, 1, 0]
      })
      const service = new ElephantSearchService()
      service.setEmbeddingProvider({
        source: 'node-llama-cpp',
        embedText
      })
      await service.registerWindowVault(3, root)

      const results = await service.search({ query: 'llm', mode: 'semantic', limit: 5 }, 3)

      expect(results).toHaveLength(1)
      expect(results[0]).toMatchObject({
        relativePath: 'Agents.md',
        matchType: 'semantic'
      })
      expect(embedText).toHaveBeenCalled()
    } finally {
      await fs.remove(root)
    }
  })

  it('uses the configured runtime embedding provider for semantic search on demand', async() => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), 'elephantnote-runtime-embedding-search-'))
    try {
      await fs.writeFile(
        path.join(root, 'LocalAI.md'),
        '# Local AI\n\nnode-llama-cpp embeddings power semantic note search.',
        'utf8'
      )
      const embeddedTexts = []
      const service = new ElephantSearchService({
        embeddingProvider: {
          source: 'node-llama-cpp',
          embedText: async(text) => {
            embeddedTexts.push(text)
            return String(text).toLowerCase().includes('node-llama-cpp') ||
              String(text).toLowerCase().includes('llama')
              ? [1, 0, 0]
              : [0, 1, 0]
          }
        }
      })
      await service.registerWindowVault(4, root)
      expect(embeddedTexts).toHaveLength(0)

      const results = await service.search({ query: 'llama embeddings', mode: 'semantic', limit: 5 }, 4)
      const inspection = await service.inspectIndex(4)

      expect(results[0]).toMatchObject({
        relativePath: 'LocalAI.md',
        matchType: 'semantic'
      })
      expect(inspection.features).toMatchObject({
        embeddingSource: 'node-llama-cpp'
      })
      expect(embeddedTexts.some((text) => text.includes('node llama cpp embeddings'))).toBe(true)
      expect(embeddedTexts).toContain('llama embeddings')
    } finally {
      await fs.remove(root)
    }
  })
})
