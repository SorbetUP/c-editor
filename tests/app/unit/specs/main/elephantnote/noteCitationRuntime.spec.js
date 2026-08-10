import { describe, expect, it, vi } from 'vitest'
import {
  appendCitationToCurrentNote,
  installNoteCitationRuntime,
  resolveInternalEntryLink,
} from '../../../../../../Elephant/frontend/src/renderer/src/platform/noteCitationRuntime.js'

describe('note citation runtime', () => {
  it('keeps the selected text when activating the citation action', async() => {
    const dom = document
    const selection = globalThis.getSelection()
    const clipboardWrite = vi.fn(async() => {})
    const target = {
      document: dom,
      getSelection: () => selection,
      MutationObserver,
      navigator: { clipboard: { writeText: clipboardWrite } },
      console,
      setTimeout,
      location: { origin: 'http://localhost' }
    }
    const host = dom.createElement('div')
    host.className = 'en-editor-host'
    const editor = dom.createElement('div')
    editor.setAttribute('contenteditable', 'true')
    editor.textContent = 'Selected citation text'
    host.append(editor)
    const title = dom.createElement('input')
    title.className = 'en-note-title-input'
    title.value = 'Source'
    const actions = dom.createElement('div')
    actions.className = 'en-note-topbar-actions'
    dom.body.append(host, title, actions)
    const range = dom.createRange()
    range.selectNodeContents(editor)
    selection.removeAllRanges()
    selection.addRange(range)

    const runtime = installNoteCitationRuntime({
      target,
      vaultStore: { openedNotePath: 'Inbox/Source.md' },
      editorStore: { currentFile: { id: 'source-1' } }
    })
    dom.dispatchEvent(new Event('selectionchange', { bubbles: true }))
    const action = dom.querySelector('[data-elephant-citation-selection-action]')
    expect(action).toBeTruthy()

    selection.removeAllRanges()
    action.click()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(clipboardWrite).toHaveBeenCalledWith(expect.stringContaining('> Selected citation text'))
    expect(dom.querySelector('[data-elephant-citation-feedback]')?.textContent).toContain('Citation copiée')
    runtime.dispose()
  })

  it('appends a citation at the last line when the active note has no selection', () => {
    const editorStore = {
      currentFile: {
        id: 'target-note',
        markdown: 'alpha\nomega',
        muyaIndexCursor: {
          anchor: { line: 1, ch: 0 },
          focus: { line: 1, ch: 0 },
        },
        isSaved: true,
      },
    }

    const result = appendCitationToCurrentNote(editorStore, '> quoted text')

    expect(result).toBe('alpha\nomega\n\n> quoted text\n')
    expect(editorStore.currentFile.markdown).toBe(result)
    expect(editorStore.currentFile.isSaved).toBe(false)
  })

  it('resolves app-owned folder links used by drag and drop', () => {
    expect(resolveInternalEntryLink('elephant://entry/Projects%2FAlpha')).toEqual({
      path: 'Projects/Alpha',
      kind: 'folder',
    })
  })
})
