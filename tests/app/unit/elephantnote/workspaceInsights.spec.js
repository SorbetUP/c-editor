import { describe, expect, it } from 'vitest'
import {
  createCalendarBuckets,
  createGraphModel,
  createRecentNoteEntries,
  createTagTopics,
  createWorkspaceStats
} from 'common/elephantnote/workspaceInsights'

const entries = [
  { type: 'folder', path: 'Projects', title: 'Projects', updatedAt: '2026-06-20T10:00:00.000Z' },
  { type: 'note', path: 'Projects/Alpha.md', title: 'Alpha', tags: ['ai', 'project'], updatedAt: '2026-06-22T10:00:00.000Z' },
  { type: 'note', path: 'Projects/Beta.md', title: 'Beta', tags: ['ai'], updatedAt: '2026-06-23T10:00:00.000Z' },
  { type: 'note', path: 'Loose.md', title: 'Loose', tags: ['misc'], updatedAt: '2026-06-21T10:00:00.000Z' }
]

describe('workspace insights', () => {
  it('keeps recent notes pinned first and deduplicated by path', () => {
    const recent = createRecentNoteEntries({
      entries,
      openedNotes: [{ type: 'note', path: 'Projects/Alpha.md', title: 'Opened Alpha', updatedAt: '2026-06-24T10:00:00.000Z' }],
      pinnedNotePaths: ['Loose.md'],
      limit: 3
    })

    expect(recent.map((note) => note.path)).toEqual([
      'Loose.md',
      'Projects/Alpha.md',
      'Projects/Beta.md'
    ])
    expect(recent[1].title).toBe('Opened Alpha')
  })

  it('creates tag topics without leaking internal sort fields', () => {
    const topics = createTagTopics(entries)

    expect(topics[0]).toMatchObject({
      tag: 'ai',
      updatedAt: '2026-06-23T10:00:00.000Z'
    })
    expect(topics[0].notes.map((note) => note.path)).toEqual(['Projects/Alpha.md', 'Projects/Beta.md'])
    expect(topics[0]).not.toHaveProperty('updatedTime')
  })

  it('computes stats, calendar buckets, and graph edges consistently', () => {
    expect(createWorkspaceStats({ entries, recentNoteEntries: entries.slice(1, 3) })).toEqual({
      notes: 3,
      folders: 1,
      tags: 3,
      recent: 2
    })

    expect(createCalendarBuckets(entries).map((bucket) => bucket.date)).toEqual([
      '2026-06-23',
      '2026-06-22',
      '2026-06-21'
    ])

    const graph = createGraphModel({ entries })
    expect(graph.nodes).toHaveLength(4)
    expect(graph.edges).toEqual(expect.arrayContaining([
      { source: 'Projects', target: 'Projects/Alpha.md', reason: 'folder' },
      { source: 'Projects', target: 'Projects/Beta.md', reason: 'folder' },
      { source: 'Projects/Alpha.md', target: 'Projects/Beta.md', reason: '#ai' }
    ]))
  })
})
