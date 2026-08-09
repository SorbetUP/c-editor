import { describe, expect, it } from 'vitest'
import {
  isStandardMarkdownImagePath,
  markdownImage,
  parseMarkdownImageDestination
} from 'common/elephantnote/imageAssetContract'

describe('common Markdown image asset contract', () => {
  it('parses standard destinations without requiring an addon', () => {
    expect(parseMarkdownImageDestination('.assets/diagram.png "preview"')).toEqual({
      source: '.assets/diagram.png',
      title: 'preview'
    })
    expect(isStandardMarkdownImagePath('.assets/diagram.png')).toBe(true)
    expect(isStandardMarkdownImagePath('.assets/diagram.excalidraw')).toBe(false)
  })

  it('serializes a portable image that remains renderable without an addon', () => {
    expect(markdownImage('diagram', '.assets/diagram.png', 'preview')).toBe(
      '![diagram](.assets/diagram.png "preview")'
    )
    expect(markdownImage('line\n\n', '.assets/diagram.png')).toBe(
      '![line](.assets/diagram.png)'
    )
  })
})
