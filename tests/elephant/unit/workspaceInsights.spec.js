import { describe, expect, it } from 'vitest'
import {
  createCalendarBuckets,
  createGraphModel,
  createRecentNoteEntries,
  createTagTopics,
  createWorkspaceStats
} from 'common/elephantnote/workspaceInsights'

const entries = [
  { kind: 'folder', path: 'Projects', title: 'Projects', updatedAt: '2026-05-17T10:00:00.000Z' },
  {
    kind: 'note',
    path: 'Projects/a.md',
    title: 'A',
    tags: ['work', 'atomic'],
    updatedAt: '2026-05-19T10:00:00.000Z'
  },
  {
    kind: 'note',
    path: 'Projects/b.md',
    title: 'B',
    tags: ['work'],
    updatedAt: '2026-05-18T10:00:00.000Z'
  }
]

describe('workspace insights', () => {
  it('derives stats, tags and calendar buckets from portable entries', () => {
    const recent = createRecentNoteEntries({ entries })

    expect(createWorkspaceStats({ entries, recentNoteEntries: recent })).toEqual({
      notes: 2,
      folders: 1,
      tags: 2,
      recent: 2
    })
    expect(createTagTopics(entries).map((topic) => [topic.tag, topic.notes.length])).toEqual([
      ['work', 2],
      ['atomic', 1]
    ])
    expect(createCalendarBuckets(entries).map((bucket) => bucket.date)).toEqual([
      '2026-05-19',
      '2026-05-18'
    ])
  })

  it('creates graph nodes and folder/tag edges without UI state', () => {
    expect(createGraphModel({ entries }).edges).toEqual([
      { source: 'Projects', target: 'Projects/a.md', reason: 'folder' },
      { source: 'Projects', target: 'Projects/b.md', reason: 'folder' },
      { source: 'Projects/a.md', target: 'Projects/b.md', reason: '#work' }
    ])
  })

  it('keeps pinned and opened notes first in recent notes', () => {
    expect(createRecentNoteEntries({
      entries,
      openedNotes: [
        { kind: 'note', path: 'Scratch.md', title: 'Scratch', updatedAt: '2026-05-16T10:00:00.000Z' }
      ],
      pinnedNotePaths: ['Projects/b.md']
    }).map((note) => note.path)).toEqual([
      'Projects/b.md',
      'Projects/a.md',
      'Scratch.md'
    ])
  })
})
