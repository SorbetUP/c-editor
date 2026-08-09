import { describe, expect, it } from 'vitest'
import { validateApiPayload } from 'common/elephantnote/apiContracts'
import {
  createConceptProfile,
  createKnowledgeChunkIndex
} from '../../../Elephant/shared/knowledge/knowledgeIndex.js'
import { rankConceptsForQuery } from '../../../Elephant/shared/knowledge/conceptRouter.js'

describe('knowledge concept route contracts', () => {
  it('accepts the public search.concepts payload shape', () => {
    const payload = { query: 'apple', limit: 5, evidenceLimit: 4 }

    expect(validateApiPayload('search.concepts', payload)).toBe(payload)
  })

  it('returns concept candidates with source chunk evidence', () => {
    const index = createKnowledgeChunkIndex([
      {
        relativePath: 'design/apple-typography.md',
        markdown: '# Apple typography\n\nSF Pro, San Francisco, typeface, kerning and UI text belong to the Apple typography concept.',
        maxWords: 48
      },
      {
        relativePath: 'business/apple-inc.md',
        markdown: '# Apple Inc.\n\niPhone, Mac, App Store revenue and developer accounts belong to the Apple company concept.',
        maxWords: 48
      }
    ], { now: new Date('2026-06-25T00:00:00.000Z') })

    const route = rankConceptsForQuery({
      query: 'apple',
      concepts: [
        createConceptProfile({
          id: 'wiki:apple-typography',
          title: 'Apple typography',
          aliases: ['SF Pro', 'Apple font'],
          positiveTerms: ['SF Pro', 'San Francisco', 'typeface', 'kerning', 'UI text']
        }),
        createConceptProfile({
          id: 'wiki:apple-inc',
          title: 'Apple Inc.',
          aliases: ['Apple company'],
          positiveTerms: ['iPhone', 'Mac', 'App Store', 'revenue', 'developer accounts']
        })
      ],
      chunks: index.chunks,
      limit: 5,
      evidenceLimit: 2
    })

    expect(route.ambiguous).toBe(true)
    expect(route.candidates.map((candidate) => candidate.id)).toEqual(
      expect.arrayContaining(['wiki:apple-typography', 'wiki:apple-inc'])
    )
    expect(route.candidates.every((candidate) => candidate.evidenceChunks.length > 0)).toBe(true)
  })
})
