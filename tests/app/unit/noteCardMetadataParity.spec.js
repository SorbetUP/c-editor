import { describe, expect, it } from 'vitest'

import { getNoteCardExcerpt, getNoteCardTitle } from '../../../Elephant/frontend/app/utils/noteCardView.js'

describe('note card metadata cleanup parity', () => {
  it('hides a leading metadata section and keeps body text', () => {
    const delimiter = String.fromCharCode(45) + String.fromCharCode(45) + String.fromCharCode(45)
    const entry = {
      title: 'Visible title',
      excerpt: [delimiter, 'alpha beta', delimiter, '', '# Visible title', '', 'Body text'].join('\n')
    }
    const preview = getNoteCardExcerpt(entry)
    expect(getNoteCardTitle(entry)).toBe('Visible title')
    expect(preview).toContain('Body text')
    expect(preview).not.toContain(delimiter)
    expect(preview).not.toContain('alpha beta')
  })
})
