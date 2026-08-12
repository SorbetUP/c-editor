import { describe, expect, it } from 'vitest'
import {
  getNoteCardDrawingPreview,
  getNoteCardExcerpt,
  getNoteCardTitle,
  getNoteCardTypeLabel
} from '@/elephantnote/utils/noteCardView'

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

  it('strips supported inline metadata with and without an inline closing delimiter', () => {
    expect(getNoteCardExcerpt({ markdown: '--- title: "Example" tags: [a, b] --- Inline body' })).toBe('Inline body')
    expect(getNoteCardExcerpt({ markdown: '--- title: Example type: note Inline body' })).toBe('Inline body')
    expect(getNoteCardExcerpt({ markdown: '---' })).toBe('No preview yet.')
  })

  it('preserves text that only resembles unsupported inline metadata', () => {
    expect(getNoteCardExcerpt({ markdown: '--- custom: value Body' })).toBe('--- custom: value Body')
    expect(getNoteCardExcerpt({ markdown: 'Plain body' })).toBe('Plain body')
  })

  it('removes a document H1 without removing ordinary leading text', () => {
    expect(getNoteCardExcerpt({ markdown: '# Title\nBody line' })).toBe('Body line')
    expect(getNoteCardExcerpt({ markdown: '# Title' })).toBe('No preview yet.')
    expect(getNoteCardExcerpt({ markdown: '## Subtitle only' })).toBe('Subtitle only')
  })

  it('returns the drawing fallback instead of parsing drawing files as markdown', () => {
    expect(getNoteCardExcerpt({ path: 'Sketch.excalidraw', markdown: '# Hidden' })).toBe('No preview yet.')
    expect(getNoteCardExcerpt({ filename: 'Sketch.excalidraw.png', content: 'Hidden' })).toBe('No preview yet.')
    expect(getNoteCardExcerpt({})).toBe('No preview yet.')
  })
})

describe('note card title and type contracts', () => {
  it('prefers trimmed metadata and strips note or drawing file extensions', () => {
    expect(getNoteCardTitle({ title: '  Explicit  ', name: 'ignored.md' })).toBe('Explicit')
    expect(getNoteCardTitle({ name: 'Alpha.md' })).toBe('Alpha')
    expect(getNoteCardTitle({ filename: 'Canvas.excalidraw.png' })).toBe('Canvas')
    expect(getNoteCardTitle({ filename: 'Canvas.excalidraw' })).toBe('Canvas')
    expect(getNoteCardTitle({})).toBe('Untitled')
  })

  it('uses an explicit type label and otherwise defaults to Note', () => {
    expect(getNoteCardTypeLabel({ type: '  Drawing  ' })).toBe('Drawing')
    expect(getNoteCardTypeLabel({})).toBe('Note')
  })
})

describe('note card drawing preview contract', () => {
  it('prefers a valid explicit asset preview', () => {
    expect(getNoteCardDrawingPreview({ drawingPreview: '.assets/preview.png', path: 'Ignored.excalidraw' }))
      .toBe('.assets/preview.png')
    expect(getNoteCardDrawingPreview({ drawingPreview: '../.assets/preview.png' }))
      .toBe('../.assets/preview.png')
  })

  it('derives previews directly from drawing paths', () => {
    expect(getNoteCardDrawingPreview({ path: 'Sketch.excalidraw.png' })).toBe('Sketch.excalidraw.png')
    expect(getNoteCardDrawingPreview({ name: 'Sketch.excalidraw' })).toBe('Sketch.png')
  })

  it('finds Excalidraw image references across previewable markdown fields', () => {
    expect(getNoteCardDrawingPreview({
      drawingPreview: 'https://example.invalid/not-local.png',
      excerpt: 'No image here',
      preview: '![Excalidraw: Sketch](.assets/sketch.png "preview")'
    })).toBe('.assets/sketch.png')
    expect(getNoteCardDrawingPreview({ markdown: '![Excalidraw: Nested](../.assets/nested.png)' }))
      .toBe('../.assets/nested.png')
    expect(getNoteCardDrawingPreview({ content: '![Excalidraw: Deep](../../.assets/deep.png)' }))
      .toBe('../../.assets/deep.png')
  })

  it('returns an empty preview when no valid drawing asset is present', () => {
    expect(getNoteCardDrawingPreview({ preview: '![Image](.assets/not-excalidraw.png)' })).toBe('')
    expect(getNoteCardDrawingPreview(null)).toBe('')
  })
})
