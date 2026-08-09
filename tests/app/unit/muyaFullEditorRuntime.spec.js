import { describe, expect, it } from 'vitest'
import { JSDOM } from 'jsdom'

import {
  applyOperation,
  clipboardPayloadToMarkdown,
  createGroupedHistory,
  createMuyaFullEditorRuntime,
  floatingToolbarState,
  footnotePopupState,
  imageToolbarState,
  jsonStateToMarkdown,
  markdownToJsonState,
  pastedHtmlToMarkdown,
  previewBlock,
  pushGroupedHistory,
  redoGroupedHistory,
  resizeImageMarkdown,
  slashCommands,
  tableCommand,
  transformOperation,
  undoGroupedHistory,
  upsertFootnote
} from '../../../Elephant/frontend/src/renderer/src/muya/fullEditorRuntime.js'

describe('Muya full editor runtime contracts', () => {
  it('creates a real contenteditable DOM editor and restores browser selection snapshots', () => {
    const dom = new JSDOM('<div id="editor"></div>')
    globalThis.document = dom.window.document
    globalThis.getSelection = dom.window.getSelection.bind(dom.window)
    const root = dom.window.document.getElementById('editor')
    const runtime = createMuyaFullEditorRuntime(root, '# Title\n\nText')
    expect(root.getAttribute('contenteditable')).toBe('true')
    expect(root.getAttribute('data-muya-editor')).toBe('true')
    expect(root.querySelector('h1')?.textContent).toBe('Title')

    const textNode = root.querySelector('p').firstChild
    const range = dom.window.document.createRange()
    range.setStart(textNode, 0)
    range.setEnd(textNode, 4)
    dom.window.getSelection().removeAllRanges()
    dom.window.getSelection().addRange(range)
    const snap = runtime.snapshotSelection()
    expect(snap.collapsed).toBe(false)
    expect(runtime.restoreSelection(snap)).toBe(true)
  })

  it('keeps a Muya-like JSONState and round-trips markdown', () => {
    const state = markdownToJsonState('# A\n\n- [x] task\n\n| A | B |\n| :- | -: |\n| 1 | 2 |')
    expect(state.type).toBe('muya-json-state')
    expect(state.blocks.some((block) => block.type === 'task_list_item' && block.checked)).toBe(true)
    expect(state.blocks.some((block) => block.type === 'table' && block.alignments[0] === 'left')).toBe(true)
    expect(jsonStateToMarkdown(state)).toContain('- [x] task')
  })

  it('applies OT operations and grouped history', () => {
    expect(applyOperation('abc', { type: 'insert', pos: 1, text: 'X' })).toBe('aXbc')
    expect(applyOperation('abc', { type: 'delete', pos: 1, count: 1 })).toBe('ac')
    expect(transformOperation({ type: 'insert', pos: 3, text: '!' }, { type: 'insert', pos: 1, text: 'X' }).pos).toBe(4)

    const history = createGroupedHistory()
    pushGroupedHistory(history, 'a', 'ab', 'typing')
    pushGroupedHistory(history, 'ab', 'abc', 'typing')
    const undone = undoGroupedHistory(history, 'abc')
    expect(undone.markdown).toBe('a')
    const redone = redoGroupedHistory(history, 'a')
    expect(redone.markdown).toBe('abc')
  })

  it('normalizes complex clipboard HTML from Word/Notion/web into markdown', () => {
    const html = '<p class="MsoNormal" style="mso-x:y">Hello <a href="https://x.test">site</a></p><ul><li><input checked> Done</li></ul><p data-block-id="abc">Notion</p>'
    const markdown = pastedHtmlToMarkdown(html)
    expect(markdown).toContain('[site](https://x.test)')
    expect(markdown).toContain('- [x] Done')
    expect(markdown).toContain('Notion')
    expect(clipboardPayloadToMarkdown({ html })).toContain('Hello')
  })

  it('supports table UI commands and image toolbar actions', () => {
    const table = '| A | B |\n| - | - |\n| 1 | 2 |'
    expect(tableCommand(table, 'insert_row', 0).split('\n').length).toBe(4)
    expect(tableCommand(table, 'insert_column', 1)).toContain('| A |  | B |')
    expect(tableCommand(table, 'align_right', 1)).toContain('-:')

    const image = 'before ![Alt](pic.png) after'
    expect(imageToolbarState(image, 10).visible).toBe(true)
    expect(resizeImageMarkdown(image, 10, '50%')).toContain('{width=50%}')
  })

  it('supports footnote popup, slash menu, floating toolbar and preview blocks', () => {
    const markdown = 'Text[^a]\n\n[^a]: note'
    expect(footnotePopupState(markdown, 8).visible).toBe(true)
    expect(upsertFootnote('Text[^b]', 'b', 'new note')).toContain('[^b]: new note')
    expect(slashCommands('/mer').some((command) => command.id === 'mermaid')).toBe(true)
    expect(floatingToolbarState({ collapsed: false }).visible).toBe(true)
    expect(previewBlock({ type: 'math_block', text: 'x+1' }).type).toBe('katex')
    expect(previewBlock({ type: 'code_fence', language: 'mermaid', text: 'graph TD;' }).type).toBe('diagram')
  })

  it('runs the assembled editor runtime end to end', () => {
    const dom = new JSDOM('<div id="editor"></div>')
    globalThis.document = dom.window.document
    globalThis.getSelection = dom.window.getSelection.bind(dom.window)
    const root = dom.window.document.getElementById('editor')
    const runtime = createMuyaFullEditorRuntime(root, '# A')
    runtime.applyOperation({ type: 'insert', pos: runtime.markdown.length, text: '\n\n![Alt](pic.png)' })
    expect(runtime.imageToolbar(runtime.markdown.length - 2).visible).toBe(true)
    runtime.pasteClipboard({ html: '<h2>Paste</h2><p>Body</p>' })
    expect(runtime.markdown).toContain('## Paste')
    runtime.table('insert_row')
    expect(runtime.copy().markdown).toContain('Paste')
    runtime.undo()
    runtime.redo()
    expect(root.getAttribute('data-muya-editor')).toBe('true')
  })
})
