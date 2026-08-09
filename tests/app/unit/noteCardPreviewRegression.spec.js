import { describe, expect, it } from 'vitest'

import {
  getNoteCardExcerpt,
  getNoteCardTitle,
  getNoteCardUpdatedLabel
} from '../../../Elephant/frontend/app/utils/noteCardView.js'

describe('real note card preview regressions', () => {
  it('does not show raw frontmatter in note card preview', () => {
    const entry = {
      title: 'Alpha',
      markdown: ['---', 'title: "Alpha"', 'type: "note"', 'tags: ["work"]', '---', '', '# Alpha', '', 'Visible body text'].join('\n')
    }
    const excerpt = getNoteCardExcerpt(entry)
    expect(excerpt).not.toContain('title:')
    expect(excerpt).not.toContain('tags:')
    expect(excerpt).toContain('Visible body text')
  })

  it('uses filename fallback when title is missing', () => {
    expect(getNoteCardTitle({ filename: 'Projects/Alpha.md' })).toBe('Projects/Alpha')
    expect(getNoteCardTitle({ name: 'Beta.md' })).toBe('Beta')
  })

  it('does not display Invalid Date for invalid metadata', () => {
    expect(getNoteCardUpdatedLabel({ updatedAt: 'invalid-date-123' })).toBe('')
  })

  it('prefers excerpt over raw markdown when both are available', () => {
    expect(getNoteCardExcerpt({ excerpt: 'Clean preview', markdown: 'Raw markdown body' })).toBe('Clean preview')
  })

  it('falls back to an explicit empty preview only when no content exists', () => {
    expect(getNoteCardExcerpt({})).toBe('No preview yet.')
  })
})
