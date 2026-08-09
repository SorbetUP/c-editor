import { describe, expect, it } from 'vitest'
import {
  getNoteCardExcerpt,
  getNoteCardTitle,
  getNoteCardTypeLabel,
  getNoteCardUpdatedLabel
} from '@/elephantnote/utils/noteCardView'

describe('ElephantNote note card view model', () => {
  it('normalizes note card labels', () => {
    expect(getNoteCardTitle({ title: '  Welcome  ' })).toBe('Welcome')
    expect(getNoteCardTitle({ title: '' })).toBe('Untitled')
    expect(getNoteCardTypeLabel({ type: 'article' })).toBe('article')
    expect(getNoteCardTypeLabel({})).toBe('Note')
    expect(getNoteCardExcerpt({ excerpt: '  Preview text  ' })).toBe('Preview text')
    expect(getNoteCardExcerpt({ excerpt: '' })).toBe('No preview yet.')
  })

  it('removes multiline YAML frontmatter and the leading document title from previews', () => {
    expect(getNoteCardExcerpt({
      excerpt: '---\ntitle: "Noteh"\ntype: "note"\ntags: []\n---\n\n# Noteh\n\nVisible preview.\nSecond line.'
    })).toBe('Visible preview.\nSecond line.')
  })

  it('removes compact inline frontmatter from previews', () => {
    expect(getNoteCardExcerpt({
      excerpt: '--- title: "Noteh" type: "note" --- Real content starts here.'
    })).toBe('Real content starts here.')
  })

  it('shows the empty-preview fallback only for empty scaffold content', () => {
    expect(getNoteCardExcerpt({
      excerpt: '---\ntitle: "Empty"\ntype: "note"\n---\n\n# Empty\n'
    })).toBe('No preview yet.')
  })

  it('formats the updated date for the note card footer', () => {
    expect(getNoteCardUpdatedLabel({ updatedAt: '2026-05-18T00:00:00.000Z' })).toBe('2026-05-18')
  })

  it('formats Unix timestamps returned by the Tauri backend', () => {
    expect(getNoteCardUpdatedLabel({ updatedAt: '1700000000' })).not.toBe('')
    expect(getNoteCardUpdatedLabel({ updatedAt: 'not-a-date' })).toBe('')
  })
})
