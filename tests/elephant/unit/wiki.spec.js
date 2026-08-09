/* @vitest-environment node */

import {
  createWikiMarkdown,
  generateWikiProposals,
  mergeWikiProposals
} from 'common/elephantnote/wiki'

describe('ElephantNote wiki synthesis', () => {
  it('generates cited wiki proposals from tagged note metadata', () => {
    const proposals = generateWikiProposals([
      {
        kind: 'note',
        path: 'Projects/a.md',
        title: 'Alpha',
        tags: ['work', 'atomic'],
        excerpt: 'Alpha summary',
        updatedAt: '2026-05-20T10:00:00.000Z'
      },
      {
        kind: 'note',
        path: 'Projects/b.md',
        title: 'Beta',
        tags: ['work'],
        excerpt: 'Beta summary',
        updatedAt: '2026-05-21T10:00:00.000Z'
      }
    ], new Date('2026-05-22T10:00:00.000Z'))

    expect(proposals.map((proposal) => [proposal.topic, proposal.citations.length])).toEqual([
      ['work', 2],
      ['atomic', 1]
    ])
    expect(proposals[0].citations.map((citation) => citation.path)).toEqual([
      'Projects/b.md',
      'Projects/a.md'
    ])
    expect(proposals[0].summary).toContain('#work')
  })

  it('preserves accepted records while refreshing open proposals', () => {
    const merged = mergeWikiProposals([
      {
        id: 'wiki-work',
        topic: 'work',
        status: 'accepted',
        notePath: 'Wiki/work.md',
        citations: [{ path: 'old.md', title: 'Old' }]
      }
    ], [
      {
        id: 'wiki-work',
        topic: 'work',
        status: 'proposed',
        citations: [{ path: 'new.md', title: 'New' }]
      },
      {
        id: 'wiki-inbox',
        topic: 'inbox',
        status: 'proposed',
        citations: [{ path: 'inbox.md', title: 'Inbox' }]
      }
    ], new Date('2026-05-22T10:00:00.000Z'))

    expect(merged.find((record) => record.id === 'wiki-work')).toMatchObject({
      status: 'accepted',
      notePath: 'Wiki/work.md'
    })
    expect(merged.find((record) => record.id === 'wiki-inbox')).toMatchObject({
      status: 'proposed'
    })
  })

  it('renders accepted wiki markdown with backlinks to cited notes', () => {
    const markdown = createWikiMarkdown({
      topic: 'work',
      title: 'work',
      summary: 'Work summary',
      citations: [
        { path: 'Projects/a.md', title: 'Alpha', excerpt: 'Alpha summary' }
      ]
    }, new Date('2026-05-22T10:00:00.000Z'))

    expect(markdown).toContain('type: "wiki"')
    expect(markdown).toContain('Work summary')
    expect(markdown).toContain('[[Projects/a.md]] - Alpha summary')
  })
})
