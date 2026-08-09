import { describe, expect, it, vi } from 'vitest'

import { dispatchMuyaChange } from '../../../../../Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditorChanges'

describe('Muya editor change forwarding', () => {
  it('still notifies runtime consumers when the active file disappeared', () => {
    const onDocumentChange = vi.fn()
    const onRuntimeChange = vi.fn()

    expect(() => dispatchMuyaChange({
      changes: { markdown: 'visible Markdown' },
      currentFileId: null,
      fromEditorMarkdown: (markdown) => `# Note\n\n${markdown}`,
      onDocumentChange,
      onRuntimeChange
    })).not.toThrow()

    expect(onDocumentChange).not.toHaveBeenCalled()
    expect(onRuntimeChange).toHaveBeenCalledWith({ markdown: '# Note\n\nvisible Markdown' })
  })

  it('updates the active file with document Markdown and preserves runtime Markdown', () => {
    const onDocumentChange = vi.fn()
    const onRuntimeChange = vi.fn()

    dispatchMuyaChange({
      changes: { markdown: 'visible Markdown', wordCount: 2 },
      currentFileId: 'note-1',
      fromEditorMarkdown: (markdown) => `# Note\n\n${markdown}`,
      onDocumentChange,
      onRuntimeChange
    })

    expect(onDocumentChange).toHaveBeenCalledWith({
      markdown: '# Note\n\nvisible Markdown',
      wordCount: 2,
      id: 'note-1'
    })
    expect(onRuntimeChange).toHaveBeenCalledWith({
      markdown: '# Note\n\nvisible Markdown',
      wordCount: 2
    })
  })
})
