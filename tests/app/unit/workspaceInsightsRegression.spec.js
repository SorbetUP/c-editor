import { describe, expect, it } from 'vitest'

import {
  createCalendarBuckets,
  createGraphModel,
  createRecentNoteEntries,
  createTagTopics,
  createWorkspaceStats
} from '../../../Elephant/shared/workspaceInsights.js'

const entries = [
  { kind: 'folder', path: 'Projects', title: 'Projects', updatedAt: '2026-06-01T08:00:00.000Z' },
  { kind: 'note', path: 'Alpha.md', title: 'Alpha', tags: ['work'], updatedAt: '2026-06-03T10:00:00.000Z' },
  { kind: 'note', path: 'Projects/Beta.md', title: 'Beta', tags: ['work', 'beta'], updatedAt: '2026-06-02T10:00:00.000Z' },
  { kind: 'note', path: 'Projects/Gamma.md', title: 'Gamma', tags: ['beta'], updatedAt: '2026-06-01T10:00:00.000Z' }
]

describe('workspace insights regression build/coverage', () => {
  it('graph model contains every visible note and folder', () => {
    const graph = createGraphModel({ entries })
    expect(graph.nodes.map((node) => node.id).sort()).toEqual(['Alpha.md', 'Projects', 'Projects/Beta.md', 'Projects/Gamma.md'].sort())
  })

  it('graph model creates folder edges for nested notes', () => {
    const graph = createGraphModel({ entries })
    expect(graph.edges).toContainEqual({ source: 'Projects', target: 'Projects/Beta.md', reason: 'folder' })
    expect(graph.edges).toContainEqual({ source: 'Projects', target: 'Projects/Gamma.md', reason: 'folder' })
  })

  it('graph model creates tag edges for shared tags', () => {
    const graph = createGraphModel({ entries })
    expect(graph.edges.some((edge) => edge.reason === '#work')).toBe(true)
    expect(graph.edges.some((edge) => edge.reason === '#beta')).toBe(true)
  })

  it('workspace stats count notes folders and tags', () => {
    const recent = createRecentNoteEntries({ entries, pinnedNotePaths: ['Projects/Beta.md'] })
    expect(createWorkspaceStats({ entries, recentNoteEntries: recent })).toEqual({ notes: 3, folders: 1, tags: 2, recent: 3 })
  })

  it('recent notes put pinned notes first', () => {
    const recent = createRecentNoteEntries({ entries, pinnedNotePaths: ['Projects/Gamma.md'] })
    expect(recent[0].path).toBe('Projects/Gamma.md')
  })

  it('calendar buckets never emit Invalid Date labels for missing dates', () => {
    const buckets = createCalendarBuckets([...entries, { kind: 'note', path: 'NoDate.md', title: 'No date' }])
    expect(buckets.map((bucket) => bucket.date)).not.toContain('Invalid Date')
    expect(buckets.map((bucket) => bucket.date)).toContain('No date')
  })

  it('tag topics group notes deterministically', () => {
    const topics = createTagTopics(entries)
    expect(topics.map((topic) => topic.tag)).toEqual(['beta', 'work'])
    expect(topics.find((topic) => topic.tag === 'work').notes).toHaveLength(2)
  })
})
