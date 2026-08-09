import { describe, expect, it } from 'vitest'
import { getNoteCardExcerpt } from '@/elephantnote/utils/noteCardView'

describe('note card frontmatter delimiter compatibility', () => {
  it('accepts trailing spaces on the opening delimiter', () => {
    expect(getNoteCardExcerpt({ markdown: '---   \ntitle: Example\n---\nBody' })).toBe('Body')
  })

  it('accepts indentation around the closing delimiter', () => {
    expect(getNoteCardExcerpt({ markdown: '---\ntitle: Example\n   ---   \nBody' })).toBe('Body')
  })

  it('accepts whitespace around CRLF delimiters', () => {
    expect(getNoteCardExcerpt({ markdown: '---\t\r\ntitle: Example\r\n\t--- \r\n# Example\r\nBody' })).toBe('Body')
  })

  it('does not consume a horizontal rule that is not a frontmatter block', () => {
    expect(getNoteCardExcerpt({ markdown: '---\nBody without a closing delimiter' })).toBe(
      '---\nBody without a closing delimiter'
    )
  })
})
