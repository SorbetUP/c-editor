import { describe, expect, it } from 'vitest'

const { buildWikiChatContext, buildWikiProposalsFromGraph, buildWikiSourceInsight, createGraphBackedWikiMarkdown } = await import(
  'main_renderer/elephantnote/wiki/wikiLibrary'
)

describe('wikiLibrary', () => {
  it('builds wiki proposals from the semantic graph', () => {
    const proposals = buildWikiProposalsFromGraph({
      graph: {
        nodes: [
          {
            kind: 'note',
            relativePath: 'Project/Plan.md',
            title: 'Plan',
            summary: 'A plan for local search',
            sources: [{ id: 's1', title: 'Source', url: 'https://example.com/plan' }],
            tags: ['ai'],
            sourceCount: 1
          },
          {
            kind: 'note',
            relativePath: 'Project/Graph.md',
            title: 'Graph',
            summary: 'Graph details for semantic links',
            sources: [{ id: 's2', title: 'Docs', url: 'https://example.com/graph' }],
            tags: ['ai'],
            sourceCount: 1
          }
        ],
        edges: [
          {
            source: 'Project/Plan.md',
            target: 'Project/Graph.md',
            type: 'semantic',
            weight: 0.83
          }
        ],
        clusters: [
          {
            id: 'cluster-ai',
            label: 'ai',
            paths: ['Project/Plan.md', 'Project/Graph.md']
          }
        ]
      }
    })

    expect(proposals).toHaveLength(1)
    expect(proposals[0]).toMatchObject({
      topic: 'ai',
      citations: expect.arrayContaining([
        expect.objectContaining({ path: 'Project/Plan.md' }),
        expect.objectContaining({ path: 'Project/Graph.md' })
      ])
    })
    expect(proposals[0].summary).toContain('semantic link')
  })

  it('excludes folder graph nodes from wiki proposal citations', () => {
    const proposals = buildWikiProposalsFromGraph({
      graph: {
        nodes: [
          {
            id: 'Project',
            kind: 'folder',
            title: 'Project'
          },
          {
            id: 'Project/Plan.md',
            kind: 'note',
            title: 'Plan',
            summary: 'A plan for local search'
          },
          {
            id: 'Project/Loose.md',
            title: 'Loose',
            summary: 'A note-like graph node without an explicit kind'
          }
        ],
        edges: [
          {
            source: 'Project',
            target: 'Project/Plan.md',
            type: 'folder',
            weight: 0.2
          },
          {
            source: 'Project/Plan.md',
            target: 'Project/Loose.md',
            type: 'semantic',
            weight: 0.9
          }
        ],
        clusters: [
          {
            id: 'cluster-project',
            label: 'Project',
            paths: ['Project', 'Project/Plan.md', 'Project/Loose.md']
          }
        ]
      }
    })

    expect(proposals).toHaveLength(1)
    expect(proposals[0].citations.map((citation) => citation.path)).toEqual([
      'Project/Plan.md',
      'Project/Loose.md'
    ])
    expect(proposals[0].summary).toContain('2 notes')
  })

  it('renders wiki markdown with related graph sources', () => {
    const markdown = createGraphBackedWikiMarkdown({
      topic: 'ai',
      title: 'ai',
      summary: 'Graph-backed summary',
      citations: [
        {
          path: 'Project/Plan.md',
          title: 'Plan',
          excerpt: 'A plan for local search'
        }
      ]
    })

    expect(markdown).toContain('## Related graph')
    expect(markdown).toContain('## Sources')
    expect(markdown).toContain('Project/Plan.md')
  })

  it('builds source insight from graph neighbors and citations', () => {
    const insight = buildWikiSourceInsight({
      graph: {
        nodes: [
          {
            kind: 'note',
            relativePath: 'Project/Plan.md',
            title: 'Plan',
            summary: 'A plan for local search',
            tags: ['ai'],
            sourceCount: 2,
            chunkCount: 4,
            sources: [{ id: 's1', title: 'Source', url: 'https://example.com/plan' }]
          },
          {
            kind: 'note',
            relativePath: 'Project/Graph.md',
            title: 'Graph',
            summary: 'Graph details for semantic links',
            tags: ['ai']
          }
        ],
        edges: [
          {
            source: 'Project/Plan.md',
            target: 'Project/Graph.md',
            type: 'semantic',
            weight: 0.91
          }
        ],
        clusters: [
          {
            id: 'cluster-ai',
            label: 'ai',
            nodeCount: 2,
            paths: ['Project/Plan.md', 'Project/Graph.md']
          }
        ]
      },
      path: 'Project/Plan.md',
      record: {
        citations: [
          {
            path: 'Project/Plan.md',
            title: 'Plan',
            excerpt: 'A plan for local search'
          }
        ]
      }
    })

    expect(insight.source).toMatchObject({
      path: 'Project/Plan.md',
      title: 'Plan',
      sourceCount: 2,
      chunkCount: 4
    })
    expect(insight.relatedNodes).toEqual([
      expect.objectContaining({
        id: 'Project/Graph.md',
        linkType: 'semantic'
      })
    ])
    expect(insight.cluster).toMatchObject({
      label: 'ai',
      nodeCount: 2
    })
    expect(insight.citations).toEqual([
      expect.objectContaining({
        path: 'Project/Plan.md',
        title: 'Plan'
      })
    ])
  })

  it('builds compact wiki chat context with graph summary', () => {
    const context = buildWikiChatContext({
      graph: {
        nodes: [
          {
            kind: 'note',
            relativePath: 'Project/Plan.md',
            title: 'Plan',
            summary: 'A plan for local search',
            tags: ['ai']
          },
          {
            kind: 'note',
            relativePath: 'Project/Graph.md',
            title: 'Graph',
            summary: 'Graph details for semantic links',
            tags: ['ai']
          }
        ],
        edges: [
          {
            source: 'Project/Plan.md',
            target: 'Project/Graph.md',
            type: 'semantic',
            weight: 0.91
          }
        ],
        clusters: [
          {
            id: 'cluster-ai',
            label: 'ai',
            nodeCount: 2,
            paths: ['Project/Plan.md', 'Project/Graph.md']
          }
        ]
      },
      path: 'Project/Plan.md',
      record: {
        citations: [
          {
            path: 'Project/Plan.md',
            title: 'Plan',
            excerpt: 'A plan for local search'
          },
          {
            path: 'Project/Graph.md',
            title: 'Graph',
            excerpt: 'Graph details for semantic links'
          }
        ]
      },
      limit: 1
    })

    expect(context.graphSummary).toEqual({
      nodes: 2,
      semanticLinks: 1,
      clusters: 1
    })
    expect(context.relatedNodes).toHaveLength(1)
    expect(context.citations).toHaveLength(1)
    expect(context.source).toMatchObject({
      path: 'Project/Plan.md',
      title: 'Plan'
    })
  })
})