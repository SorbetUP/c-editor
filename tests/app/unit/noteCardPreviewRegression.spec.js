import { describe, expect, it } from 'vitest'

import {
  getNoteCardExcerpt,
  getNoteCardDrawingPreview,
  getNoteCardTitle,
  getNoteCardUpdatedLabel
} from '../../../Elephant/frontend/app/utils/noteCardView.js'
import { serializeDraggedEntry } from '../../../Elephant/frontend/app/utils/entryDragDrop.js'

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
    expect(getNoteCardTitle({ name: 'Drawings/Beta.excalidraw' })).toBe('Drawings/Beta')
    expect(getNoteCardTitle({ title: 'Drawings/Gamma.excalidraw' })).toBe('Drawings/Gamma')
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

  it('extracts Excalidraw image previews from drawing notes', () => {
    expect(getNoteCardDrawingPreview({
      excerpt: '![Excalidraw: Plan](.assets/Acceptance%20drawing.png)'
    })).toBe('.assets/Acceptance%20drawing.png')
    expect(getNoteCardDrawingPreview({
      drawingPreview: './.assets/Acceptance%20drawing.png'
    })).toBe('./.assets/Acceptance%20drawing.png')
    expect(getNoteCardDrawingPreview({
      preview: '![Excalidraw: Nested](../.assets/excalidraw-Nested.png)'
    })).toBe('../.assets/excalidraw-Nested.png')
    expect(getNoteCardDrawingPreview({ excerpt: '![Photo](.assets/photo.png)' })).toBe('')
  })

  it('uses the sidecar PNG for a direct Excalidraw file and never exposes its JSON as a preview', () => {
    const entry = {
      name: 'Visual direct drawing.excalidraw',
      path: 'Visual direct drawing.excalidraw',
      content: '{"type":"excalidraw","elements":[]}'
    }

    expect(getNoteCardDrawingPreview(entry)).toBe('Visual direct drawing.png')
    expect(getNoteCardExcerpt(entry)).toBe('No preview yet.')
  })

  it('keeps drawing previews in drag payloads for editor embedding', () => {
    const payload = JSON.parse(serializeDraggedEntry({
      kind: 'note',
      path: 'Drawings/Plan.md',
      title: 'Plan',
      drawingPreview: '../.assets/Plan.png'
    }))

    expect(payload.path).toBe('Drawings/Plan.md')
    expect(payload.preview).toBe('../.assets/Plan.png')
  })
})
