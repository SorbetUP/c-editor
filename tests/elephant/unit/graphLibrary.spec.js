import { describe, expect, it } from 'vitest'

const { createSemanticGraph } = await import('main_renderer/elephantnote/search/graphLibrary')

describe('createSemanticGraph', () => {
  it('creates folder, note and semantic graph nodes', () => {
    const graph = createSemanticGraph({
      documents: [
        {
          relativePath: 'Project/Plan.md',
          title: 'Plan',
          tags: ['ai'],
          sources: [],
          chunks: []
        },
        {
          relativePath: 'Project/Graph.md',
          title: 'Graph',
          tags: ['ai'],
          sources: [],
          chunks: []
        }
      ],
      semanticLinks: [
        {
          source: 'Project/Plan.md',
          target: 'Project/Graph.md',
          score: 0.91
        }
      ]
    })

    expect(graph.nodes).toEqual(expect.arrayContaining([
      expect.objectContaining({ kind: 'folder', id: 'Project' }),
      expect.objectContaining({ kind: 'note', id: 'Project/Plan.md' }),
      expect.objectContaining({ kind: 'note', id: 'Project/Graph.md' })
    ]))
    expect(graph.edges).toEqual(expect.arrayContaining([
      expect.objectContaining({ type: 'folder' }),
      expect.objectContaining({ type: 'semantic' })
    ]))
  })
})

