import { describe, expect, it } from 'vitest'
import {
  getDocumentTitle,
  mergeEditorMarkdown,
  renameDocumentTitle,
  toEditorMarkdown,
  updateMarkdownTags
} from 'common/elephantnote/markdownDocument'
import { getNoteCardExcerpt } from 'elephant-front/utils/noteCardView'

describe('ElephantNote markdown document helpers', () => {
  it('shows body text in the editor without frontmatter or duplicated title heading', () => {
    const document = [
      '---',
      'title: "Project note"',
      'type: "note"',
      'tags: []',
      '---',
      '',
      '# Project note',
      '',
      'The visible body must stay editable.'
    ].join('\n')

    expect(toEditorMarkdown(document, 'Project note')).toBe('The visible body must stay editable.')
  })

  it('preserves the body when editor text is merged back into the stored document', () => {
    const currentDocument = [
      '---',
      'title: "Daily"',
      'type: "note"',
      'tags: []',
      '---',
      '',
      '# Daily',
      '',
      'Old body'
    ].join('\n')

    const result = mergeEditorMarkdown(currentDocument, 'New body', 'Daily')

    expect(result).toContain('title: "Daily"')
    expect(result).toContain('# Daily')
    expect(result).toContain('New body')
    expect(result).not.toContain('Old body')
  })

  it('keeps a generated untitled note titleless until the user enters a title', () => {
    expect(getDocumentTitle('', 'Untitled')).toBe('')
    expect(getDocumentTitle('', 'Untitled 2')).toBe('')
    expect(mergeEditorMarkdown('', 'First body line', 'Untitled')).toBe('First body line')
    expect(mergeEditorMarkdown('First body line', 'Updated body', 'Untitled')).toBe('Updated body')
  })

  it('can add tags to a titleless note without synthesizing an Untitled heading', () => {
    const result = updateMarkdownTags('Body', ['draft'], '')

    expect(result).toContain('tags: ["draft"]')
    expect(result).toContain('Body')
    expect(result).not.toContain('Untitled')
    expect(result).not.toContain('# ')
  })

  it('renames frontmatter and heading together to avoid list/editor title drift', () => {
    const currentDocument = [
      '---',
      'title: "Old title"',
      'type: "note"',
      'tags: []',
      '---',
      '',
      '# Old title',
      '',
      'Body'
    ].join('\n')

    const result = renameDocumentTitle(currentDocument, 'New title')

    expect(result).toContain('title: "New title"')
    expect(result).toContain('# New title')
    expect(result).toContain('Body')
    expect(result).not.toContain('# Old title')
  })

  it('updates tags without dropping the note body', () => {
    const currentDocument = [
      '---',
      'title: "Tagged"',
      'type: "note"',
      'tags:',
      '  - old',
      '---',
      '',
      '# Tagged',
      '',
      'Body'
    ].join('\n')

    const result = updateMarkdownTags(currentDocument, ['new', 'urgent'], 'Tagged')

    expect(result).toContain('tags: ["new", "urgent"]')
    expect(result).toContain('# Tagged')
    expect(result).toContain('Body')
    expect(result).not.toContain('  - old')
  })

  it('shows note card body text instead of raw multiline frontmatter', () => {
    const entry = {
      excerpt: [
        '---',
        'title: "Noteh"',
        'type: "note"',
        '---',
        '',
        'This is the real note content.'
      ].join('\n')
    }

    expect(getNoteCardExcerpt(entry)).toBe('This is the real note content.')
  })

  it('shows note card body text instead of the duplicated title heading', () => {
    const entry = {
      excerpt: [
        '---',
        'title: "Noteh"',
        'type: "note"',
        '---',
        '',
        '# Noteh',
        '',
        'This is the real note content after the title.'
      ].join('\n')
    }

    expect(getNoteCardExcerpt(entry)).toBe('This is the real note content after the title.')
  })

  it('hides compact inline frontmatter when no body preview exists', () => {
    expect(getNoteCardExcerpt({ excerpt: '--- title: "Noteh" type: "note"' })).toBe('No preview yet.')
  })

  it('keeps card body text after compact inline frontmatter', () => {
    const entry = {
      excerpt: '--- title: "Noteh" type: "note" --- Real body after metadata.'
    }

    expect(getNoteCardExcerpt(entry)).toBe('Real body after metadata.')
  })
})
