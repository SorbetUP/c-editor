import { createDomEditor } from './selectionRuntime.js'
import { markdownToJsonState, jsonStateToMarkdown, jsonStateToHtml, renderJsonStateIntoDom } from './jsonStateRuntime.js'
import { applyOperation, createGroupedHistory, pushGroupedHistory, undoGroupedHistory, redoGroupedHistory } from './operationsRuntime.js'
import { clipboardPayloadToMarkdown, copyMarkdownAndHtml } from './clipboardRuntime.js'
import { imageToolbarState, resizeImageMarkdown, tableCommand } from './tableImageRuntime.js'
import { floatingToolbarState, footnotePopupState, previewBlock, slashCommands, upsertFootnote } from './menusPreviewRuntime.js'
import { createLiveRenderScheduler, domToMarkdown } from './liveRenderingRuntime.js'
import { renderCurrentBlockNow } from './blockLiveRuntime.js'
import { renderPreviewBlock } from './previewRenderersRuntime.js'

export const createMuyaFullEditorRuntime = (root, markdown = '', options = {}) => {
  const editor = createDomEditor(root, options.document || globalThis.document)
  const history = createGroupedHistory()
  let state = markdownToJsonState(markdown)
  renderJsonStateIntoDom(root, state, options.document || globalThis.document)

  const setMarkdown = (next, group = 'setMarkdown') => {
    const before = jsonStateToMarkdown(state)
    state = markdownToJsonState(next)
    pushGroupedHistory(history, before, next, group)
    renderJsonStateIntoDom(root, state, options.document || globalThis.document)
    return state
  }

  const live = createLiveRenderScheduler({
    root,
    getState: () => state,
    setState: (nextState) => { state = nextState },
    getDocument: () => options.document || globalThis.document,
    delay: options.liveDelay || 0
  })

  const syncDomToState = (group = 'live') => {
    const before = jsonStateToMarkdown(state)
    const blockResult = renderCurrentBlockNow({
      root,
      setState: (nextState) => { state = nextState },
      getDocument: () => options.document || globalThis.document
    })
    const result = blockResult || live.renderNow()
    // Browser editing can create temporary <div> blocks for Enter. The DOM is
    // the authoritative user input at this point; do not return the previous
    // JSON state when a block-local renderer handled the event.
    const after = domToMarkdown(root)
    state = markdownToJsonState(after)
    const normalized = jsonStateToMarkdown(state)
    console.info('[elephantnote:muya-js] input-sync', {
      group,
      beforeLength: before.length,
      domMarkdownLength: after.length,
      normalizedLength: normalized.length,
      html: root.innerHTML.slice(0, 1000)
    })
    if (result && normalized !== before) pushGroupedHistory(history, before, normalized, group)
    return state
  }

  return {
    root,
    editor,
    history,
    live,
    get state() { return state },
    get markdown() { return jsonStateToMarkdown(state) },
    get html() { return jsonStateToHtml(state) },
    setMarkdown,
    scheduleLiveRender: () => live.schedule(),
    renderLiveNow: syncDomToState,
    renderCurrentBlockNow: () => renderCurrentBlockNow({ root, setState: (nextState) => { state = nextState }, getDocument: () => options.document || globalThis.document }),
    domToMarkdown: () => domToMarkdown(root),
    snapshotSelection: () => editor?.snapshotSelection?.(),
    restoreSelection: (snapshot) => editor?.restoreSelection?.(snapshot),
    applyOperation: (operation, group = 'operation') => setMarkdown(applyOperation(jsonStateToMarkdown(state), operation), group),
    undo: () => {
      const result = undoGroupedHistory(history, jsonStateToMarkdown(state))
      state = markdownToJsonState(result.markdown)
      renderJsonStateIntoDom(root, state, options.document || globalThis.document)
      return state
    },
    redo: () => {
      const result = redoGroupedHistory(history, jsonStateToMarkdown(state))
      state = markdownToJsonState(result.markdown)
      renderJsonStateIntoDom(root, state, options.document || globalThis.document)
      return state
    },
    pasteClipboard: (payload) => setMarkdown(`${jsonStateToMarkdown(state)}\n\n${clipboardPayloadToMarkdown(payload)}`.trim(), 'paste'),
    copy: () => copyMarkdownAndHtml(jsonStateToMarkdown(state), () => jsonStateToHtml(state)),
    table: (command, index = 0) => setMarkdown(tableCommand(jsonStateToMarkdown(state), command, index), `table:${command}`),
    imageToolbar: (cursor) => imageToolbarState(jsonStateToMarkdown(state), cursor),
    resizeImage: (cursor, width) => setMarkdown(resizeImageMarkdown(jsonStateToMarkdown(state), cursor, width), 'image:resize'),
    footnotePopup: (cursor) => footnotePopupState(jsonStateToMarkdown(state), cursor),
    upsertFootnote: (label, text) => setMarkdown(upsertFootnote(jsonStateToMarkdown(state), label, text), 'footnote'),
    slashCommands,
    floatingToolbar: floatingToolbarState,
    previewBlock,
    renderPreviewBlock
  }
}

export * from './selectionRuntime.js'
export * from './jsonStateRuntime.js'
export * from './operationsRuntime.js'
export * from './clipboardRuntime.js'
export * from './tableImageRuntime.js'
export * from './menusPreviewRuntime.js'
export * from './liveRenderingRuntime.js'
export * from './blockLiveRuntime.js'
export * from './inputRulesRuntime.js'
export * from './previewRenderersRuntime.js'
