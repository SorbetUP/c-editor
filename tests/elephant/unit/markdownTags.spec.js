import { describe, expect, it } from 'vitest'
import {
  deleteMarkdownTag,
  parseMarkdownTags,
  renameMarkdownTag,
  updateMarkdownTags
} from 'elephant-front/utils/markdownTags'
import {
  parseMarkdownTags as parsePortableMarkdownTags
} from 'common/elephantnote/markdownDocument'

describe('markdownTags', () => {
  const baseMarkdown = `---
title: "Untitled"
type: "note"
tags: ["getting-started", "draft"]
createdAt: "2026-05-19T12:00:00.000Z"
updatedAt: "2026-05-19T12:00:00.000Z"
---

# Untitled

Body text.
`

  it('parses tags from frontmatter', () => {
    expect(parseMarkdownTags(baseMarkdown)).toEqual(['getting-started', 'draft'])
    expect(parsePortableMarkdownTags(baseMarkdown)).toEqual(['getting-started', 'draft'])
  })

  it('creates and updates the tags list', () => {
    const next = updateMarkdownTags(baseMarkdown, ['getting-started', 'ideas', 'draft'], 'Untitled')
    expect(parseMarkdownTags(next)).toEqual(['getting-started', 'ideas', 'draft'])
    expect(next).toContain('tags: ["getting-started", "ideas", "draft"]')
  })

  it('renames an existing tag', () => {
    const next = renameMarkdownTag(baseMarkdown, 'draft', 'review', 'Untitled')
    expect(parseMarkdownTags(next)).toEqual(['getting-started', 'review'])
  })

  it('deletes a tag', () => {
    const next = deleteMarkdownTag(baseMarkdown, 'getting-started', 'Untitled')
    expect(parseMarkdownTags(next)).toEqual(['draft'])
  })

  it('parses YAML block tags and quoted commas', () => {
    const markdown = `---
title: "Research"
tags:
  - "#ideas"
  - "needs, review"
---

# Research
`

    expect(parseMarkdownTags(markdown)).toEqual(['ideas', 'needs, review'])
  })

  it('replaces YAML block tags without leaving stale list items', () => {
    const markdown = `---
title: "Research"
tags:
  - "#ideas"
  - "needs, review"
createdAt: "2026-05-17T23:43:04.008Z"
---

# Research
`

    const next = updateMarkdownTags(markdown, ['done'], 'Research')

    expect(parseMarkdownTags(next)).toEqual(['done'])
    expect(next).toContain('tags: ["done"]')
    expect(next).not.toContain('  - "#ideas"')
    expect(next).toContain('createdAt: "2026-05-17T23:43:04.008Z"')
  })

  it('parses tags from CRLF frontmatter', () => {
    const markdown = [
      '---',
      'title: "Windows note"',
      'tags: ["work", "urgent"]',
      '---',
      '',
      '# Windows note'
    ].join('\r\n')

    expect(parseMarkdownTags(markdown)).toEqual(['work', 'urgent'])
  })
})
