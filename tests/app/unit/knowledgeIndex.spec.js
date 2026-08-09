import { describe, expect, it } from 'vitest'
import fs from 'fs/promises'
import path from 'node:path'

import {
  createConceptProfile,
  createKnowledgeChunkIndex,
  createStableKnowledgeHash,
  rankConceptsForChunk,
  scoreChunkAgainstConcept
} from '../../../Elephant/shared/knowledge/knowledgeIndex.js'

const fixtureRoot = path.join(process.cwd(), 'tests/app/fixtures/knowledge/apple')

const readFixture = async (filename) => fs.readFile(path.join(fixtureRoot, filename), 'utf8')

const createAppleConcepts = () => ({
  inc: createConceptProfile({
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
  fruit: createConceptProfile({
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
  typography: createConceptProfile({
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
})

const buildAppleIndex = async () => {
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
  return createKnowledgeChunkIndex(documents, { now: new Date('2026-06-25T00:00:00.000Z') })
}

describe('knowledgeIndex', () => {
  it('creates stable normalized hashes for chunk identity', () => {
    expect(createStableKnowledgeHash('SF Pro system font')).toBe(createStableKnowledgeHash(' sf pro   system font '))
    expect(createStableKnowledgeHash('SF Pro system font')).not.toBe(createStableKnowledgeHash('apple pie recipe'))
  })

  it('preserves heading context on generated chunks', async () => {
    const index = await buildAppleIndex()
    const typographyChunk = index.chunks.find((chunk) =>
      chunk.documentPath === 'apple/mixed-apple-design.md' &&
      chunk.text.includes('San Francisco font family')
    )

    expect(typographyChunk).toBeTruthy()
    expect(typographyChunk.headingPath).toEqual(['Apple design notes', 'Typography'])
    expect(typographyChunk.textHash).toMatch(/^[a-z0-9]+$/)
    expect(typographyChunk.lexicalTerms).toContain('typography')
  })

  it('scores chunks against the right Apple concept instead of the word apple alone', async () => {
    const index = await buildAppleIndex()
    const concepts = createAppleConcepts()
    const typographyChunk = index.chunks.find((chunk) => chunk.documentPath === 'apple/apple-typography.md')
    const fruitChunk = index.chunks.find((chunk) => chunk.documentPath === 'apple/apple-fruit.md')
    const incChunk = index.chunks.find((chunk) => chunk.documentPath === 'apple/apple-inc.md')

    expect(scoreChunkAgainstConcept(typographyChunk, concepts.typography).score)
      .toBeGreaterThan(scoreChunkAgainstConcept(typographyChunk, concepts.inc).score)
    expect(scoreChunkAgainstConcept(fruitChunk, concepts.fruit).score)
      .toBeGreaterThan(scoreChunkAgainstConcept(fruitChunk, concepts.typography).score)
    expect(scoreChunkAgainstConcept(incChunk, concepts.inc).score)
      .toBeGreaterThan(scoreChunkAgainstConcept(incChunk, concepts.fruit).score)
  })

  it('allows one mixed note to contribute different chunks to different concept wikis', async () => {
    const index = await buildAppleIndex()
    const concepts = createAppleConcepts()
    const mixedChunks = index.chunks.filter((chunk) => chunk.documentPath === 'apple/mixed-apple-design.md')
    const typographyChunk = mixedChunks.find((chunk) => chunk.headingPath.includes('Typography'))
    const companyChunk = mixedChunks.find((chunk) => chunk.headingPath.includes('Company context'))

    expect(rankConceptsForChunk(typographyChunk, Object.values(concepts))[0].concept.id).toBe('wiki:apple-typography')
    expect(rankConceptsForChunk(companyChunk, Object.values(concepts))[0].concept.id).toBe('wiki:apple-inc')
  })

  it('keeps broad ambiguous apple queries as multiple plausible concept candidates', async () => {
    const index = await buildAppleIndex()
    const concepts = createAppleConcepts()
    const bestScores = Object.values(concepts).map((concept) => ({
      id: concept.id,
      score: Math.max(...index.chunks.map((chunk) => scoreChunkAgainstConcept(chunk, concept).score))
    }))

    expect(bestScores.find((item) => item.id === 'wiki:apple-inc')?.score).toBeGreaterThan(0)
    expect(bestScores.find((item) => item.id === 'wiki:apple-fruit')?.score).toBeGreaterThan(0)
    expect(bestScores.find((item) => item.id === 'wiki:apple-typography')?.score).toBeGreaterThan(0)
  })
})
