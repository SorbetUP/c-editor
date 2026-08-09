import { describe, expect, it } from 'vitest'
import {
  createAtomicDocument,
  createAtomicSemanticIndex,
  createCitedAnswer,
  ensureAtomicSourcesSection,
  extractMarkdownSources,
  searchAtomicSemanticIndex,
  suggestAtomicTags
} from 'common/elephantnote/atomicAiEngine'

describe('atomicAiEngine', () => {
  it('runs the full local embedding sequence with sources and semantic links', () => {
    const index = createAtomicSemanticIndex([
      createAtomicDocument({
        relativePath: 'AI/Embeddings.md',
        markdown: '# Embeddings\n\nLocal AI embeddings connect notes to sources. Source: https://example.com/ai'
      }),
      createAtomicDocument({
        relativePath: 'AI/Graph.md',
        markdown: '# Knowledge Graph\n\nEmbedding links create graph edges between related AI notes.'
      }),
      createAtomicDocument({
        relativePath: 'Recipes/Bread.md',
        markdown: '# Bread\n\nFlour water and yeast make a loaf.'
      })
    ])

    const results = searchAtomicSemanticIndex({
      index,
      query: 'llm embeddings connected graph sources',
      limit: 2
    })

    expect(results[0].relativePath).toBe('AI/Embeddings.md')
    expect(results.some((result) => result.relativePath === 'AI/Graph.md')).toBe(true)
    expect(results[0].sources[0]).toMatchObject({ url: 'https://example.com/ai' })
    expect(index.semanticLinks).toEqual(expect.arrayContaining([
      expect.objectContaining({
        source: 'AI/Embeddings.md',
        target: 'AI/Graph.md',
        reason: 'embedding-similarity'
      })
    ]))
  })

  it('extracts citations from markdown links, source fields and inline urls once', () => {
    const sources = extractMarkdownSources(`
      source: https://example.com/source
      See [Atomic](https://example.com/atomic) and https://example.com/source.
    `)

    expect(sources.map((source) => source.url)).toEqual([
      'https://example.com/atomic',
      'https://example.com/source'
    ])
  })

  it('adds a Sources section when markdown has citations but no source block', () => {
    const markdown = ensureAtomicSourcesSection('# Article\n\nRead https://example.com/research.')

    expect(markdown).toContain('## Sources')
    expect(markdown).toContain('- [https://example.com/research](https://example.com/research)')
    expect(ensureAtomicSourcesSection(markdown)).toBe(markdown.trimEnd())
  })

  it('creates local tags and cited RAG answers without external secrets', () => {
    const tags = suggestAtomicTags('# Local AI\n\n#embedding #vault local local source graph graph graph')
    const answer = createCitedAnswer({
      question: 'How are notes linked?',
      results: [
        {
          title: 'Graph',
          relativePath: 'Graph.md',
          sources: [{ url: 'https://example.com/graph' }]
        }
      ]
    })

    expect(tags).toEqual(expect.arrayContaining(['embedding', 'vault', 'graph']))
    expect(answer.answer).toContain('[1] Graph')
    expect(answer.citations[0]).toMatchObject({
      relativePath: 'Graph.md',
      sourceUrl: 'https://example.com/graph'
    })
  })
})
