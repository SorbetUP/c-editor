import { describe, expect, it } from 'vitest'
import fs from 'fs/promises'
import path from 'node:path'

import {
  createConceptProfile,
  createKnowledgeChunkIndex
} from '../../../Elephant/shared/knowledge/knowledgeIndex.js'
import {
  createQueryProfile,
  rankConceptsForQuery
} from '../../../Elephant/shared/knowledge/conceptRouter.js'

const fixtureRoot = path.join(process.cwd(), 'tests/app/fixtures/knowledge/apple')

const readFixture = async (filename) => fs.readFile(path.join(fixtureRoot, filename), 'utf8')

const createAppleConcepts = () => [
  createConceptProfile({
    id: 'wiki:apple-inc',
    title: 'Apple Inc.',
    aliases: ['Apple company', 'iPhone company'],
    positiveTerms: [
      'iPhone',
      'Mac',
      'iPad',
      'App Store',
      'iCloud',
      'revenue',
      'hardware',
      'subscriptions',
      'developer accounts',
      'platform strategy',
      'Tim Cook',
      'Xcode'
    ],
    negativeTerms: ['fruit', 'recipe', 'pie', 'font', 'typeface', 'kerning', 'typography']
  }),
  createConceptProfile({
    id: 'wiki:apple-fruit',
    title: 'Apple fruit',
    aliases: ['apple tree', 'apple pie'],
    positiveTerms: [
      'fruit',
      'tree',
      'nutrition',
      'recipes',
      'pie',
      'juice',
      'orchards',
      'cooking',
      'slices',
      'sugar',
      'cinnamon',
      'pastry'
    ],
    negativeTerms: ['iPhone', 'Mac', 'App Store', 'font', 'typeface', 'SF Pro']
  }),
  createConceptProfile({
    id: 'wiki:apple-typography',
    title: 'Apple typography',
    aliases: ['SF Pro', 'San Francisco font', 'Apple font'],
    positiveTerms: [
      'SF Pro',
      'San Francisco',
      'system font',
      'typeface',
      'font family',
      'kerning',
      'legibility',
      'Human Interface Guidelines',
      'UI text',
      'text styles',
      'iOS interface',
      'macOS interface',
      'design system typography',
      'font weight',
      'optical size'
    ],
    negativeTerms: ['fruit', 'recipe', 'pie', 'juice', 'revenue', 'hardware sales']
  })
]

const buildAppleChunks = async () => {
  const files = [
    'apple-inc.md',
    'apple-fruit.md',
    'apple-typography.md',
    'mixed-apple-design.md',
    'ambiguous-apple.md'
  ]
  const documents = await Promise.all(files.map(async (filename) => ({
    relativePath: `apple/${filename}`,
    markdown: await readFixture(filename),
    maxWords: 48
  })))
  return createKnowledgeChunkIndex(documents, { now: new Date('2026-06-25T00:00:00.000Z') }).chunks
}

describe('conceptRouter', () => {
  it('detects short non-question queries as ambiguity candidates', () => {
    expect(createQueryProfile('apple').isAmbiguousCandidate).toBe(true)
    expect(createQueryProfile('what is apple?').isAmbiguousCandidate).toBe(false)
  })

  it('keeps multiple wiki candidates for the broad apple query', async () => {
    const chunks = await buildAppleChunks()
    const route = rankConceptsForQuery({ query: 'apple', concepts: createAppleConcepts(), chunks, limit: 5 })
    const ids = route.candidates.map((candidate) => candidate.id)

    expect(route.ambiguous).toBe(true)
    expect(ids).toContain('wiki:apple-inc')
    expect(ids).toContain('wiki:apple-fruit')
    expect(ids).toContain('wiki:apple-typography')
  })

  it('routes SF Pro to Apple typography first', async () => {
    const chunks = await buildAppleChunks()
    const route = rankConceptsForQuery({ query: 'SF Pro', concepts: createAppleConcepts(), chunks, limit: 3 })

    expect(route.candidates[0].id).toBe('wiki:apple-typography')
    expect(route.candidates[0].evidenceChunks.length).toBeGreaterThan(0)
  })

  it('routes iPhone revenue to Apple Inc. first', async () => {
    const chunks = await buildAppleChunks()
    const route = rankConceptsForQuery({ query: 'iPhone revenue', concepts: createAppleConcepts(), chunks, limit: 3 })

    expect(route.candidates[0].id).toBe('wiki:apple-inc')
  })

  it('routes apple pie to Apple fruit first', async () => {
    const chunks = await buildAppleChunks()
    const route = rankConceptsForQuery({ query: 'apple pie', concepts: createAppleConcepts(), chunks, limit: 3 })

    expect(route.candidates[0].id).toBe('wiki:apple-fruit')
  })
})
