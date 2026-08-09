import { describe, expect, it } from 'vitest'

import { markdownFromClipboard } from '../../../../../Elephant/frontend/src/muya/lib/rust/inputController/clipboard'

const eventWith = (values) => ({
  clipboardData: {
    getData: (type) => values[type] || ''
  }
})

describe('Muya Rust clipboard payload selection', () => {
  it.each([
    'application/x-elephant-markdown',
    'application/x-muya-markdown',
    'application/x-markdown',
    'text/markdown',
    'text/x-markdown'
  ])('prefers %s over lossy HTML and plain text', (type) => {
    const event = eventWith({
      [type]: '**native**\r\n\r\n- item',
      'text/html': '<p>lossy</p>',
      'text/plain': 'plain'
    })

    expect(markdownFromClipboard(event, document)).toBe('**native**\n\n- item')
  })

  it('uses semantic HTML when no native Markdown payload exists', () => {
    const event = eventWith({
      'text/html': '<p><strong>bold</strong></p>',
      'text/plain': 'bold'
    })

    expect(markdownFromClipboard(event, document)).toBe('**bold**')
  })

  it('falls back to normalized plain text', () => {
    const event = eventWith({ 'text/plain': 'one\r\n\r\ntwo' })
    expect(markdownFromClipboard(event, document)).toBe('one\n\ntwo')
  })

  it('returns null when no clipboard API is available', () => {
    expect(markdownFromClipboard({}, document)).toBeNull()
  })
})
