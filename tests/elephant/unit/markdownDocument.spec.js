import { describe, expect, it } from 'vitest'
import {
  deleteMarkdownTag,
  ensureNoteDocument,
  getDocumentTitle,
  getEditorMarkdownStats,
  mergeEditorMarkdown,
  parseMarkdownTags,
  renameDocumentTitle,
  renameMarkdownTag,
  toEditorMarkdown,
  updateMarkdownTags
} from 'common/elephantnote/markdownDocument'

const documentMarkdown = [
  '---',
  'title: "Project plan"',
  'type: "note"',
  'tags: ["work"]',
  'createdAt: "2026-05-17T23:43:04.008Z"',
  'updatedAt: "2026-05-17T23:43:04.008Z"',
  '---',
  '',
  '# Project plan',
  '',
  'Body text'
].join('\n')

describe('portable markdown document contract', () => {
  it('separates editor markdown from persisted note markdown', () => {
    const nextMarkdown = mergeEditorMarkdown(documentMarkdown, 'Updated body')

    expect(toEditorMarkdown(documentMarkdown)).toBe('Body text')
    expect(nextMarkdown).toContain('title: "Project plan"')
    expect(nextMarkdown).toContain('tags: ["work"]')
    expect(nextMarkdown).toContain('# Project plan')
    expect(nextMarkdown).toContain('Updated body')
  })

  it('creates empty note documents without injecting a visible title body', () => {
    const emptyDocument = ensureNoteDocument('', 'Empty note')

    expect(emptyDocument).toContain('title: "Empty note"')
    expect(emptyDocument).not.toContain('# Empty note')
    expect(toEditorMarkdown(emptyDocument)).toBe('')
    expect(getEditorMarkdownStats(toEditorMarkdown(emptyDocument))).toEqual({
      word: 0,
      character: 0
    })
  })

  it('renames document titles in frontmatter and visible heading', () => {
    const nextMarkdown = renameDocumentTitle(documentMarkdown, 'Roadmap')

    expect(getDocumentTitle(nextMarkdown)).toBe('Roadmap')
    expect(nextMarkdown).toContain('title: "Roadmap"')
    expect(nextMarkdown).toContain('# Roadmap')
    expect(nextMarkdown).not.toContain('# Project plan')
  })

  it('parses and mutates inline and block frontmatter tags', () => {
    const blockTags = [
      '---',
      'title: "Research"',
      'tags:',
      '  - "#ideas"',
      '  - "needs, review"',
      'createdAt: "2026-05-17T23:43:04.008Z"',
      '---',
      '',
      '# Research'
    ].join('\n')

    const updated = updateMarkdownTags(blockTags, ['done'], 'Research')
    const renamed = renameMarkdownTag(documentMarkdown, 'work', 'review', 'Project plan')
    const deleted = deleteMarkdownTag(renamed, 'review', 'Project plan')

    expect(parseMarkdownTags(blockTags)).toEqual(['ideas', 'needs, review'])
    expect(parseMarkdownTags(updated)).toEqual(['done'])
    expect(updated).not.toContain('  - "#ideas"')
    expect(parseMarkdownTags(renamed)).toEqual(['review'])
    expect(parseMarkdownTags(deleted)).toEqual([])
  })
})
