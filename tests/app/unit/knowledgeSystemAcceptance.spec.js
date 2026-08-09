import { afterEach, describe, expect, it } from 'vitest'
import fs from 'fs-extra'
import os from 'node:os'
import path from 'node:path'

import {
  createAtomicDocument,
  createAtomicSemanticIndex
} from '../../../Elephant/shared/atomicAiEngine.js'
import {
  createConceptProfile,
  createKnowledgeChunkIndex,
  scoreChunkAgainstConcept
} from '../../../Elephant/shared/knowledge/knowledgeIndex.js'
import { rankConceptsForQuery } from '../../../Elephant/shared/knowledge/conceptRouter.js'
import { buildAutomaticOrganizationPlan } from '../../../Elephant/shared/knowledge/organizationPlanner.js'
import { createSearchLibrary } from '../../../Elephant/backend/js/search/searchLibrary.js'
import { createSemanticGraph } from '../../../Elephant/backend/js/search/graphLibrary.js'
import {
  buildWikiProposalsFromGraph,
  createGraphBackedWikiMarkdown
} from '../../../Elephant/backend/js/wiki/wikiLibrary.js'

let tempRoots = []

const createTempVault = async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'elephantnote-knowledge-'))
  tempRoots.push(root)
  await fs.ensureDir(path.join(root, 'business'))
  await fs.ensureDir(path.join(root, 'design'))
  await fs.ensureDir(path.join(root, 'cooking'))
  await fs.writeFile(path.join(root, 'business', 'apple-inc.md'), '# Apple Inc.\n\niPhone revenue, Mac hardware, App Store subscriptions and developer accounts define the Apple company topic.\n')
  await fs.writeFile(path.join(root, 'business', 'app-store.md'), '# App Store strategy\n\nDeveloper accounts, subscriptions, iPhone distribution and platform revenue belong with Apple Inc.\n')
  await fs.writeFile(path.join(root, 'design', 'apple-typography.md'), '# Apple typography\n\nSF Pro, San Francisco typeface, kerning, optical size and UI text belong to Apple typography.\n')
  await fs.writeFile(path.join(root, 'design', 'sf-pro.md'), '# SF Pro notes\n\nSan Francisco font family, font weight, legibility and iOS interface text belong with typography.\n')
  await fs.writeFile(path.join(root, 'cooking', 'apple-pie.md'), '# Apple pie\n\nFruit slices, cinnamon, pastry, sugar and cooking recipes belong to the apple fruit topic.\n')
  return root
}

const createAppleConcepts = () => [
  createConceptProfile({
    id: 'wiki:apple-inc',
    title: 'Apple Inc.',
    aliases: ['Apple company', 'iPhone company'],
    positiveTerms: ['iPhone', 'Mac', 'App Store', 'revenue', 'subscriptions', 'developer accounts', 'platform'],
    negativeTerms: ['fruit', 'pastry', 'cinnamon', 'typeface', 'kerning']
  }),
  createConceptProfile({
    id: 'wiki:apple-typography',
    title: 'Apple typography',
    aliases: ['SF Pro', 'San Francisco font', 'Apple font'],
    positiveTerms: ['SF Pro', 'San Francisco', 'typeface', 'font family', 'kerning', 'legibility', 'UI text'],
    negativeTerms: ['revenue', 'subscriptions', 'fruit', 'pastry']
  }),
  createConceptProfile({
    id: 'wiki:apple-fruit',
    title: 'Apple fruit',
    aliases: ['apple pie', 'apple recipe'],
    positiveTerms: ['fruit', 'slices', 'cinnamon', 'pastry', 'sugar', 'cooking', 'recipes'],
    negativeTerms: ['iPhone', 'Mac', 'typeface', 'font']
  })
]

const createGraphFixture = () => {
  const documents = [
    {
      ...createAtomicDocument({ relativePath: 'business/apple-inc.md', markdown: '# Apple Inc.\n\niPhone revenue Mac App Store subscriptions developer accounts.' }),
      tags: ['business', 'apple-company']
    },
    {
      ...createAtomicDocument({ relativePath: 'business/app-store.md', markdown: '# App Store strategy\n\nDeveloper accounts subscriptions iPhone platform revenue.' }),
      tags: ['business', 'apple-company']
    },
    {
      ...createAtomicDocument({ relativePath: 'design/apple-typography.md', markdown: '# Apple typography\n\nSF Pro San Francisco typeface kerning UI text.' }),
      tags: ['typography', 'design']
    },
    {
      ...createAtomicDocument({ relativePath: 'design/sf-pro.md', markdown: '# SF Pro notes\n\nSan Francisco font family legibility iOS interface text.' }),
      tags: ['typography', 'design']
    },
    {
      ...createAtomicDocument({ relativePath: 'cooking/apple-pie.md', markdown: '# Apple pie\n\nFruit slices cinnamon pastry sugar cooking recipes.' }),
      tags: ['fruit', 'cooking']
    }
  ]
  const semanticLinks = [
    {
      id: 'semantic:business/apple-inc.md->business/app-store.md',
      source: 'business/apple-inc.md',
      target: 'business/app-store.md',
      score: 0.82,
      reason: 'business-topic'
    },
    {
      id: 'semantic:design/apple-typography.md->design/sf-pro.md',
      source: 'design/apple-typography.md',
      target: 'design/sf-pro.md',
      score: 0.88,
      reason: 'typography-topic'
    }
  ]
  return { documents, semanticLinks }
}

afterEach(async () => {
  for (const root of tempRoots) {
    await fs.remove(root)
  }
  tempRoots = []
})

describe('knowledge system acceptance', () => {
  it('validates exact, semantic and smart search against a real temporary vault', async () => {
    const root = await createTempVault()
    const search = createSearchLibrary()
    await search.registerWindowVault(1, root)

    const exact = await search.search({ query: 'iPhone revenue', mode: 'exact', limit: 5 }, 1)
    expect(exact[0].relativePath).toBe('business/apple-inc.md')
    expect(exact[0].snippets[0].text).toContain('iPhone revenue')

    const smartBeforeIndex = await search.search({ query: 'iPhone revenue', mode: 'smart', limit: 5 }, 1)
    expect(smartBeforeIndex[0].relativePath).toBe('business/apple-inc.md')

    await search.rebuildIndex(1)
    const semantic = await search.search({ query: 'San Francisco typeface kerning UI text', mode: 'semantic', limit: 5 }, 1)
    expect(semantic[0].relativePath).toBe('design/apple-typography.md')
    expect(semantic[0].matchType).toBe('semantic')

    const smartAfterIndex = await search.search({ query: 'developer accounts subscriptions iPhone platform', mode: 'smart', limit: 5 }, 1)
    expect(smartAfterIndex[0].relativePath).toMatch(/^business\//)

    const inspection = await search.inspectIndex(1)
    expect(inspection.features.chunkLevelKnowledgeIndex).toBe(true)
    expect(inspection.chunkIndex.stats.documents).toBe(5)
    expect(inspection.chunks.length).toBeGreaterThanOrEqual(5)
    expect(inspection.graph.nodes.length).toBeGreaterThanOrEqual(5)
  })

  it('validates chunk-level concept routing and ambiguity evidence', () => {
    const index = createKnowledgeChunkIndex([
      {
        relativePath: 'design/apple-typography.md',
        markdown: '# Apple typography\n\nSF Pro, San Francisco typeface, kerning and UI text belong to Apple typography.'
      },
      {
        relativePath: 'business/apple-inc.md',
        markdown: '# Apple Inc.\n\niPhone revenue, Mac hardware, App Store subscriptions and developer accounts define Apple Inc.'
      },
      {
        relativePath: 'cooking/apple-pie.md',
        markdown: '# Apple pie\n\nFruit slices, cinnamon, pastry and cooking recipes define apple fruit.'
      }
    ], { now: new Date('2026-06-25T00:00:00.000Z') })
    const concepts = createAppleConcepts()
    const typographyChunk = index.chunks.find((chunk) => chunk.documentPath === 'design/apple-typography.md')

    expect(scoreChunkAgainstConcept(typographyChunk, concepts[1]).score).toBeGreaterThan(0.35)
    expect(scoreChunkAgainstConcept(typographyChunk, concepts[0]).score).toBeLessThan(0.3)

    const broadRoute = rankConceptsForQuery({ query: 'apple', concepts, chunks: index.chunks, limit: 5 })
    expect(broadRoute.ambiguous).toBe(true)
    expect(broadRoute.candidates.map((candidate) => candidate.id)).toEqual(expect.arrayContaining([
      'wiki:apple-inc',
      'wiki:apple-typography',
      'wiki:apple-fruit'
    ]))
    expect(broadRoute.candidates.every((candidate) => candidate.evidenceChunks.length > 0)).toBe(true)

    const preciseRoute = rankConceptsForQuery({ query: 'SF Pro kerning', concepts, chunks: index.chunks, limit: 3 })
    expect(preciseRoute.candidates[0].id).toBe('wiki:apple-typography')
  })

  it('validates semantic graph clustering, wiki synthesis and automatic organization proposals', () => {
    const { documents, semanticLinks } = createGraphFixture()
    const graph = createSemanticGraph({ documents, semanticLinks })
    const semanticClusters = graph.clusters.filter((cluster) => cluster.kind === 'semantic')

    expect(semanticClusters.length).toBeGreaterThanOrEqual(2)
    expect(semanticClusters.some((cluster) => cluster.paths.includes('design/apple-typography.md') && cluster.paths.includes('design/sf-pro.md'))).toBe(true)
    expect(semanticClusters.some((cluster) => cluster.paths.includes('business/apple-inc.md') && cluster.paths.includes('business/app-store.md'))).toBe(true)
    expect(semanticClusters.every((cluster) => cluster.cohesion > 0.5)).toBe(true)

    const proposals = buildWikiProposalsFromGraph({ graph, now: new Date('2026-06-25T00:00:00.000Z') })
    expect(proposals.length).toBeGreaterThanOrEqual(2)
    expect(proposals.every((proposal) => proposal.citations.length >= 2)).toBe(true)

    const wikiMarkdown = createGraphBackedWikiMarkdown(proposals[0], new Date('2026-06-25T00:00:00.000Z'))
    expect(wikiMarkdown).toContain('## Related graph')
    expect(wikiMarkdown).toContain('## Sources')
    expect(wikiMarkdown).toContain('Source notes')

    const organization = buildAutomaticOrganizationPlan({ graph, now: new Date('2026-06-25T00:00:00.000Z') })
    expect(organization.safeMode).toBe('propose-only')
    expect(organization.appliesMovesAutomatically).toBe(false)
    expect(organization.proposals.length).toBeGreaterThanOrEqual(2)
    expect(organization.proposals.every((proposal) => proposal.status === 'ready')).toBe(true)
    expect(organization.proposals.every((proposal) => proposal.wikiPath.endsWith('.md'))).toBe(true)
  })

  it('keeps semantic links from the atomic engine usable by graph clustering', () => {
    const documents = [
      createAtomicDocument({ relativePath: 'design/apple-typography.md', markdown: '# Apple typography\n\nSF Pro typeface kerning UI text legibility font family.' }),
      createAtomicDocument({ relativePath: 'design/sf-pro.md', markdown: '# SF Pro\n\nSan Francisco typeface UI text legibility font family kerning.' }),
      createAtomicDocument({ relativePath: 'cooking/apple-pie.md', markdown: '# Apple pie\n\nFruit slices cinnamon pastry sugar cooking.' })
    ]
    const index = createAtomicSemanticIndex(documents, { linkThreshold: 0.2 })
    const graph = createSemanticGraph({ documents, semanticLinks: index.semanticLinks })

    expect(index.semanticLinks.some((link) => link.source === 'design/apple-typography.md' && link.target === 'design/sf-pro.md')).toBe(true)
    expect(graph.clusters.some((cluster) =>
      cluster.kind === 'semantic' &&
      cluster.paths.includes('design/apple-typography.md') &&
      cluster.paths.includes('design/sf-pro.md')
    )).toBe(true)
  })
})
