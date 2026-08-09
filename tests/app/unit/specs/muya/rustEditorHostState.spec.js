import { describe, expect, it, vi } from 'vitest'

import { applyRustEditorMarkdown } from '../../../../../Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditorState'

const createState = () => {
  const file = { id: 'note-1', markdown: '# Title\n\nalpha', isSaved: true }
  return {
    file,
    editorStore: {
      tabs: [file],
      tabIdToIndex: { 'note-1': 0 }
    }
  }
}

describe('Rust editor host state synchronization', () => {
  it('converts editor Markdown and updates the active tab atomically', () => {
    const { file, editorStore } = createState()
    const persist = vi.fn()

    const changed = applyRustEditorMarkdown({
      editorStore,
      file,
      editorMarkdown: 'alpha!',
      fromEditorMarkdown: (markdown) => `# Title\n\n${markdown}`,
      persist
    })

    expect(changed).toBe(true)
    expect(file.markdown).toBe('# Title\n\nalpha!')
    expect(file.isSaved).toBe(false)
    expect(editorStore.tabs[0].markdown).toBe('# Title\n\nalpha!')
    expect(editorStore.tabs[0].isSaved).toBe(false)
    expect(persist).toHaveBeenCalledTimes(1)
  })

  it('does not persist a duplicate snapshot', () => {
    const { file, editorStore } = createState()
    const persist = vi.fn()

    const changed = applyRustEditorMarkdown({
      editorStore,
      file,
      editorMarkdown: '# Title\n\nalpha',
      persist
    })

    expect(changed).toBe(false)
    expect(file.isSaved).toBe(true)
    expect(persist).not.toHaveBeenCalled()
  })

  it('ignores updates when no active file exists', () => {
    const persist = vi.fn()
    expect(
      applyRustEditorMarkdown({
        editorStore: { tabs: [], tabIdToIndex: {} },
        file: null,
        editorMarkdown: 'orphan',
        persist
      })
    ).toBe(false)
    expect(persist).not.toHaveBeenCalled()
  })
})
