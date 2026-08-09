import { createRustMuyaEngineClient, isRustMuyaEngineAvailable } from './rustEngineRuntime.js'

let sessionSequence = 0
const asMarkdown = (value) => String(value ?? '')
const createSessionId = () => `muya:${Date.now()}:${++sessionSequence}`
const clamp = (value, minimum, maximum) => Math.min(Math.max(value, minimum), maximum)

const createStatus = () => ({
  active: false,
  phase: 'disabled',
  reason: '',
  revision: 0,
  markdownLength: 0,
  blocks: 0,
  selection: { anchor: 0, focus: 0 },
  undoDepth: 0,
  redoDepth: 0,
  error: ''
})

const publishStatus = (target, status) => {
  if (!target || typeof target !== 'object') return
  target.__ELEPHANT_MUYA_RUST_MIRROR__ = {
    ...status,
    selection: { ...status.selection }
  }
}

export const muyaIndexCursorToSelection = (markdown, cursor) => {
  const source = asMarkdown(markdown)
  const lines = source.split('\n')
  const pointToOffset = (point) => {
    if (!point || !Number.isInteger(point.line) || !Number.isInteger(point.ch)) return source.length
    const line = clamp(point.line, 0, Math.max(0, lines.length - 1))
    const ch = clamp(point.ch, 0, lines[line]?.length || 0)
    let offset = ch
    for (let index = 0; index < line; index += 1) offset += lines[index].length + 1
    return offset
  }
  return { anchor: pointToOffset(cursor?.anchor), focus: pointToOffset(cursor?.focus) }
}

export const selectionToMuyaIndexCursor = (markdown, selection) => {
  const source = asMarkdown(markdown)
  const lines = source.split('\n')
  const offsetToPoint = (value) => {
    let remaining = clamp(Number.isInteger(value) ? value : source.length, 0, source.length)
    for (let line = 0; line < lines.length; line += 1) {
      const length = lines[line].length
      if (remaining <= length) return { line, ch: remaining }
      remaining -= length
      if (line < lines.length - 1) remaining -= 1
    }
    const line = Math.max(0, lines.length - 1)
    return { line, ch: lines[line]?.length || 0 }
  }
  return { anchor: offsetToPoint(selection?.anchor), focus: offsetToPoint(selection?.focus) }
}

const unavailableFacade = (status) => {
  const unavailable = async() => {
    throw new Error('The canonical Rust Muya engine is unavailable.')
  }
  return new Proxy({
    active: false,
    status,
    ready: Promise.resolve(status),
    destroy: () => {}
  }, {
    get(target, property) {
      if (property in target) return target[property]
      return unavailable
    }
  })
}

export const createRealMuyaRustMirror = ({
  initialMarkdown = '',
  invoke,
  target = globalThis,
  logger = console
} = {}) => {
  const status = createStatus()
  if (typeof invoke !== 'function' && !isRustMuyaEngineAvailable(target)) {
    status.reason = 'tauri-invoke-unavailable'
    publishStatus(target, status)
    return unavailableFacade(status)
  }

  const client = createRustMuyaEngineClient({ invoke, target, sessionId: createSessionId() })
  let destroyed = false
  let initialized = false
  let pending = null
  let draining = null
  let commandQueue = Promise.resolve()
  let lastMarkdown = null
  let lastSelection = null

  status.active = true
  status.phase = 'initializing'
  publishStatus(target, status)

  const fail = (error) => {
    status.phase = 'error'
    status.error = error instanceof Error ? error.message : String(error)
    publishStatus(target, status)
    logger.error?.('[elephantnote:muya-rust] core failed', { error: status.error })
  }

  const refresh = async(reason) => {
    const state = client.state
    const jsonState = await client.jsonState()
    if (jsonState.type !== 'muya-json-state' || !Array.isArray(jsonState.blocks)) {
      throw new Error('Rust Muya core returned an invalid document tree.')
    }
    lastMarkdown = state.markdown
    lastSelection = { ...state.selection }
    Object.assign(status, {
      phase: 'ready',
      reason,
      revision: Number(state.revision) || 0,
      markdownLength: state.markdown.length,
      blocks: jsonState.blocks.length,
      selection: { ...state.selection },
      undoDepth: Number(state.undoDepth) || 0,
      redoDepth: Number(state.redoDepth) || 0,
      error: ''
    })
    publishStatus(target, status)
    return status
  }

  const validate = async(item) => {
    const state = item.kind === 'reset' || !initialized
      ? await client.create(item.markdown)
      : (await client.syncDocument(item.markdown, item.selection, item.continueGroup)).state
    initialized = true
    if (state.markdown !== item.markdown) {
      throw new Error('Rust Muya core changed Markdown during synchronization.')
    }
    if (state.selection.anchor !== item.selection.anchor || state.selection.focus !== item.selection.focus) {
      await client.setSelection(item.selection.anchor, item.selection.focus)
    }
    return refresh(item.reason)
  }

  const drain = async() => {
    if (destroyed || !pending) return
    const item = pending
    pending = null
    const sameSelection = lastSelection &&
      item.selection.anchor === lastSelection.anchor &&
      item.selection.focus === lastSelection.focus
    if (item.kind === 'reset' || item.markdown !== lastMarkdown || !sameSelection) await validate(item)
    if (!destroyed && pending) await drain()
  }

  const ensureDrain = () => {
    if (draining || destroyed) return draining || Promise.resolve(status)
    draining = Promise.resolve()
      .then(drain)
      .catch((error) => {
        fail(error)
        throw error
      })
      .finally(() => {
        draining = null
        if (pending && !destroyed) ensureDrain()
      })
    return draining
  }

  const enqueue = (kind, markdown, reason, options = {}) => {
    if (destroyed) return Promise.reject(new Error('Rust Muya session is destroyed.'))
    const source = asMarkdown(markdown)
    pending = {
      kind,
      markdown: source,
      selection: options.selection || muyaIndexCursorToSelection(source, options.muyaIndexCursor),
      reason: String(reason || kind),
      continueGroup: kind === 'sync' && Boolean(options.continueGroup)
    }
    return ensureDrain()
  }

  const sync = (markdown, reason = 'change', options = {}) => enqueue('sync', markdown, reason, options)
  const reset = (markdown, reason = 'set-markdown', options = {}) => enqueue('reset', markdown, reason, options)
  const flush = async() => {
    if (draining) await draining
    else if (pending) await ensureDrain()
    if (draining || pending) return flush()
    return status
  }

  const execute = (reason, operation, refreshAfter = true) => {
    commandQueue = commandQueue
      .catch(() => null)
      .then(async() => {
        await flush()
        if (status.phase === 'error') throw new Error(status.error)
        if (destroyed || !initialized) throw new Error('Rust Muya session is not initialized.')
        const result = await operation(client)
        if (refreshAfter) await refresh(reason)
        return result
      })
    return commandQueue
  }

  const command = (reason, callback) => (...args) => execute(
    reason,
    (engine) => callback(engine, ...args)
  )
  const query = (reason, callback) => (...args) => execute(
    reason,
    (engine) => callback(engine, ...args),
    false
  )

  const ready = reset(initialMarkdown, 'initial')
  target.__ELEPHANT_ACTIVE_EDITOR_ENGINE__ = 'muya-ui-rust-core'
  logger.info?.('[elephantnote:editor] real Muya UI with Rust-owned core active', {
    engine: 'rust',
    surface: 'muya',
    sessionId: client.sessionId
  })

  return {
    active: true,
    status,
    ready,
    sync,
    reset,
    flush,
    complete: command('rust-complete', (engine, typedCommand) => engine.applyComplete(typedCommand)),
    query: query('rust-query', (engine, typedQuery) => engine.query(typedQuery)),
    clipboard: query('rust-clipboard', (engine) => engine.clipboard()),
    searchMatches: query('rust-search', (engine, queryText, options = {}) => engine.query({
      type: 'search',
      query: String(queryText),
      caseSensitive: Boolean(options.caseSensitive),
      wholeWord: Boolean(options.wholeWord)
    })),
    undo: command('rust-undo', (engine) => engine.undo()),
    redo: command('rust-redo', (engine) => engine.redo()),
    setSelection: command('rust-selection', (engine, anchor, focus = anchor) => (
      engine.setSelection(anchor, focus)
    )),
    toggleInline: command('rust-inline', (engine, marker) => engine.toggleInline(String(marker))),
    transformBlock: command('rust-block', (engine, kind) => engine.transformBlock(String(kind))),
    applyOperation: command('rust-operation', (engine, operation) => engine.applyOperation(operation)),
    keyboardRule: command('rust-keyboard', (engine, key, options = {}) => (
      engine.keyboardRule(String(key), options)
    )),
    tableCommand: command('rust-table', (engine, action, index = 0) => (
      engine.tableCommand(String(action), index)
    )),
    upsertFootnote: command('rust-footnote', (engine, label, text = '') => (
      engine.upsertFootnote(String(label), String(text))
    )),
    insertTemplate: command('rust-template', (engine, id) => engine.insertTemplate(String(id))),
    pasteClipboard: command('rust-paste', (engine, html = '', text = '') => (
      engine.pasteClipboard(String(html), String(text))
    )),
    commitComposition: command('rust-ime', (engine, selection, text) => (
      engine.commitComposition(selection, String(text))
    )),
    replaceRange: command('rust-replace-range', (engine, start, end, text = '') => (
      engine.replaceRange(start, end, text)
    )),
    deleteBackward: command('rust-delete-backward', (engine) => engine.completeDeleteBackward()),
    deleteForward: command('rust-delete-forward', (engine) => engine.completeDeleteForward()),
    insertParagraph: command('rust-insert-paragraph', (engine, location, text = '') => (
      engine.insertParagraph(location, text)
    )),
    duplicateBlock: command('rust-duplicate-block', (engine) => engine.duplicateBlock()),
    deleteBlock: command('rust-delete-block', (engine) => engine.deleteBlock()),
    moveBlock: command('rust-move-block', (engine, fromStart, fromEnd, targetOffset) => (
      engine.moveBlock(fromStart, fromEnd, targetOffset)
    )),
    indentSelection: command('rust-indent', (engine, options = {}) => engine.indentSelection(options)),
    toggleTask: command('rust-task', (engine) => engine.toggleTask()),
    setCodeLanguage: command('rust-code-language', (engine, language) => (
      engine.setCodeLanguage(language)
    )),
    insertLink: command('rust-insert-link', (engine, url, title = '') => (
      engine.insertLink(url, title)
    )),
    removeLink: command('rust-remove-link', (engine) => engine.removeLink()),
    searchReplace: command('rust-search-replace', (engine, options) => engine.searchReplace(options)),
    selectAll: command('rust-select-all', (engine) => engine.selectAll()),
    get state() { return client.state },
    get sessionId() { return client.sessionId },
    destroy: () => {
      destroyed = true
      pending = null
      client.close().catch(fail)
      if (target.__ELEPHANT_ACTIVE_EDITOR_ENGINE__ === 'muya-ui-rust-core') {
        delete target.__ELEPHANT_ACTIVE_EDITOR_ENGINE__
      }
      delete target.__ELEPHANT_MUYA_RUST_MIRROR__
    }
  }
}

