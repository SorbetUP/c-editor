import { describe, expect, it } from 'vitest'

import {
  getNoteCardExcerpt,
  getNoteCardTitle,
  getNoteCardTypeLabel,
  getNoteCardUpdatedLabel
} from '../../../Elephant/frontend/app/utils/noteCardView.js'

describe('note card basic Electron/Tauri parity', () => {
  it('uses title first and filename as fallback', () => {
    expect(getNoteCardTitle({ title: 'Real title', filename: 'file.md' })).toBe('Real title')
    expect(getNoteCardTitle({ filename: 'Fallback.md' })).toBe('Fallback')
  })

  it('uses clean preview text and fallback preview', () => {
    expect(getNoteCardExcerpt({ excerpt: 'Visible body content.' })).toBe('Visible body content.')
    expect(getNoteCardExcerpt({ excerpt: '' })).toBe('No preview yet.')
  })

  it('uses safe type and date labels', () => {
    expect(getNoteCardTypeLabel({ type: 'note' })).toBe('note')
    expect(getNoteCardTypeLabel({})).toBe('Note')
    expect(getNoteCardUpdatedLabel({ updatedAt: 'bad-date' })).toBe('')
  })
})
