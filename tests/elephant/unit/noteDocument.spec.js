import { describe, expect, it } from 'vitest'
import {
  ensureNoteDocument,
  getDocumentTitle,
  getEditorMarkdownStats,
  mergeEditorMarkdown,
  renameDocumentTitle,
  toEditorMarkdown
} from 'elephant-front/utils/noteDocument'
import {
  toEditorMarkdown as toPortableEditorMarkdown
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

describe('ElephantNote noteDocument', () => {
  it('keeps frontmatter out of the editor markdown', () => {
    expect(toEditorMarkdown(documentMarkdown)).toBe('Body text')
    expect(toPortableEditorMarkdown(documentMarkdown)).toBe('Body text')
  })

  it('rehydrates editor markdown before saving', () => {
    const nextMarkdown = mergeEditorMarkdown(documentMarkdown, 'Updated body')

    expect(nextMarkdown).toContain('title: "Project plan"')
    expect(nextMarkdown).toContain('tags: ["work"]')
    expect(nextMarkdown).toContain('# Project plan')
    expect(nextMarkdown).toContain('Updated body')
  })

  it('keeps frontmatter metadata when source mode saves body markdown', () => {
    const nextMarkdown = mergeEditorMarkdown(documentMarkdown, 'Updated from source')

    expect(nextMarkdown).toContain('tags: ["work"]')
    expect(nextMarkdown).toContain('createdAt: "2026-05-17T23:43:04.008Z"')
    expect(toEditorMarkdown(nextMarkdown)).toBe('Updated from source')
  })

  it('does not inject the title heading into an empty note body', () => {
    const emptyDocument = ensureNoteDocument('', 'Empty note')

    expect(emptyDocument).toContain('title: "Empty note"')
    expect(emptyDocument).not.toContain('# Empty note')
    expect(toEditorMarkdown(emptyDocument)).toBe('')
  })

  it('counts only visible editor markdown for an empty note document', () => {
    const emptyDocument = ensureNoteDocument('', 'Empty note')
    const stats = getEditorMarkdownStats(toEditorMarkdown(emptyDocument))

    expect(stats).toEqual({
      word: 0,
      character: 0
    })
  })

  it('counts visible editor markdown without frontmatter or hidden title', () => {
    const stats = getEditorMarkdownStats(toEditorMarkdown(documentMarkdown))

    expect(stats).toEqual({
      word: 2,
      character: 'Body text'.length
    })
  })

  it('keeps an empty note body empty when saving', () => {
    const emptyDocument = ensureNoteDocument('', 'Empty note')
    const nextMarkdown = mergeEditorMarkdown(emptyDocument, '')

    expect(nextMarkdown).toContain('title: "Empty note"')
    expect(nextMarkdown).not.toContain('# Empty note')
  })

  it('renames both frontmatter and visible heading', () => {
    const nextMarkdown = renameDocumentTitle(documentMarkdown, 'Roadmap')

    expect(getDocumentTitle(nextMarkdown)).toBe('Roadmap')
    expect(nextMarkdown).toContain('title: "Roadmap"')
    expect(nextMarkdown).toContain('# Roadmap')
    expect(nextMarkdown).not.toContain('# Project plan')
  })
})
