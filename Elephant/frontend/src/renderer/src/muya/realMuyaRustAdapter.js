import Muya from '../../../muya/lib'
import { createProgrammaticChangeGuard } from './rustProgrammaticChangeGuard.js'
import {
  createRealMuyaRustMirror,
  muyaIndexCursorToSelection,
  selectionToMuyaIndexCursor
} from './realMuyaRustMirrorRuntime.js'

const BLOCK_KINDS = Object.freeze({
  paragraph: 'paragraph',
  'reset-to-paragraph': 'paragraph',
  'heading 1': 'heading1',
  'heading 2': 'heading2',
  'heading 3': 'heading3',
  'heading 4': 'heading4',
  'heading 5': 'heading5',
  'heading 6': 'heading6',
  blockquote: 'quote',
  'ul-bullet': 'bullet',
  'ol-order': 'ordered',
  'ul-task': 'task'
})

const TABLE_ROW = /^\s*\|.*\|\s*$/
const TABLE_SEPARATOR = /^\s*\|(?:\s*:?-+:?\s*\|)+\s*$/
const MUTATING_INPUT = /^(?:insert|delete|history|format)/

const getInvoke = () => {
  const ipc = globalThis?.tauri?.ipcRenderer
  if (typeof ipc?.invoke === 'function') return ipc.invoke.bind(ipc)
  const core = globalThis?.__TAURI__?.core
  if (typeof core?.invoke === 'function') return core.invoke.bind(core)
  return null
}

const offsetForLine = (lines, line) => {
  let offset = 0
  for (let index = 0; index < line; index += 1) offset += lines[index].length + 1
  return offset
}

const tableRangeAtLine = (markdown, activeLine) => {
  const lines = String(markdown || '').split('\n')
  for (let start = 0; start < lines.length - 1; start += 1) {
    if (!TABLE_ROW.test(lines[start]) || !TABLE_SEPARATOR.test(lines[start + 1])) continue
    let end = start + 1
    while (end + 1 < lines.length && TABLE_ROW.test(lines[end + 1])) end += 1
    if (activeLine >= start && activeLine <= end) {
      const startOffset = offsetForLine(lines, start)
      const endOffset = offsetForLine(lines, end) + lines[end].length
      return { start: startOffset, end: endOffset, startLine: start, endLine: end }
    }
    start = end
  }
  return null
}

const fileBytes = async(file) => Array.from(new Uint8Array(await file.arrayBuffer()))

const tokenRangeAtSelection = (markdown, selection, token) => {
  const range = token?.range
  if (!range || !Number.isInteger(range.start) || !Number.isInteger(range.end)) {
    throw new Error('Muya token does not expose a stable range.')
  }
  const count = Math.max(0, range.end - range.start)
  const end = Math.max(selection.anchor, selection.focus)
  const start = Math.max(0, end - count)
  if (start > markdown.length || end > markdown.length || end < start) {
    throw new Error('Muya token range is outside the canonical document.')
  }
  return { start, end }
}

export default class RustOwnedMuya extends Muya {
  constructor (element, options = {}) {
    super(element, options)
    this.__rustProgrammaticChanges = this.__rustProgrammaticChanges || createProgrammaticChangeGuard()
    this.__rustApplying = false
    this.__rustComposition = null
    this.__rustClipboard = { markdown: '', html: '' }
    this.__rustMirror = createRealMuyaRustMirror({
      initialMarkdown: options.markdown || '',
      target: globalThis
    })
    this.__installRustHooks()
    this.__installRustEvents()

    this.__rustChangeListener = ({ markdown, muyaIndexCursor } = {}) => {
      if (typeof markdown !== 'string') return
      if (this.__programmaticGuard().consume()) return
      if (this.__rustComposition) {
        this.__rustComposition.finalMarkdown = markdown
        this.__rustComposition.finalCursor = muyaIndexCursor
        return
      }
      if (this.__rustApplying) return
      const state = this.__rustMirror?.state
      if (!state || state.markdown === markdown) return
      console.error('[elephantnote:muya-rust] rejected non-canonical JavaScript mutation', {
        canonicalLength: state.markdown.length,
        receivedLength: markdown.length,
        revision: state.revision
      })
      this.__setProgrammaticMarkdown(
        state.markdown,
        undefined,
        true,
        selectionToMuyaIndexCursor(state.markdown, state.selection)
      )
    }

    this.__rustSelectionListener = () => {
      if (!this.__rustMirror?.active || this.__rustApplying || this.__rustComposition || this.__programmaticGuard().pending) return
      const markdown = this.getMarkdown()
      const muyaIndexCursor = this.contentState.getMuyaIndexCursor()
      this.__rustMirror.sync(markdown, 'selection-change', {
        muyaIndexCursor,
        continueGroup: false
      }).then(() => this.__refreshClipboard()).catch(this.__reportRustError)
    }

    this.on('change', this.__rustChangeListener)
    this.on('selectionChange', this.__rustSelectionListener)
  }

  __programmaticGuard () {
    if (!this.__rustProgrammaticChanges) {
      this.__rustProgrammaticChanges = createProgrammaticChangeGuard()
    }
    return this.__rustProgrammaticChanges
  }

  __setProgrammaticMarkdown (markdown, cursor, isRenderCursor = true, muyaIndexCursor, blocks) {
    return this.__programmaticGuard().run(() => super.setMarkdown(
      markdown,
      cursor,
      isRenderCursor,
      muyaIndexCursor,
      blocks
    ))
  }

  __reportRustError = (error) => {
    const message = error instanceof Error ? error.message : String(error)
    console.error('[elephantnote:muya-rust] canonical command failed', error)
    this.eventCenter.dispatch('crashed', { engine: 'rust', error: message })
  }

  __requireRust () {
    if (!this.__rustMirror?.active) {
      throw new Error('The canonical Rust editor is required; JavaScript fallback is disabled.')
    }
    return this.__rustMirror
  }

  __selection () {
    const markdown = this.getMarkdown()
    const cursor = this.contentState.getMuyaIndexCursor()
    return {
      markdown,
      cursor,
      selection: muyaIndexCursorToSelection(markdown, cursor)
    }
  }

  async __applyRust (name, operation) {
    const engine = this.__requireRust()
    this.__rustApplying = true
    try {
      await engine.flush()
      const { markdown, cursor } = this.__selection()
      const canonical = engine.state?.markdown
      if (typeof canonical === 'string' && canonical !== markdown) {
        throw new Error(`Refusing ${name}: Muya DOM diverged from the Rust document.`)
      }
      await engine.sync(markdown, `pre-${name}`, {
        muyaIndexCursor: cursor,
        continueGroup: false
      })
      const transaction = await operation(engine)
      return this.__renderRust(transaction)
    } catch (error) {
      this.__reportRustError(error)
      throw error
    } finally {
      this.__rustApplying = false
    }
  }

  __renderRust (transaction) {
    if (!transaction?.state) return transaction
    if (!transaction.documentChanged && !transaction.selectionChanged) return transaction
    const { state } = transaction
    this.__setProgrammaticMarkdown(
      state.markdown,
      undefined,
      true,
      selectionToMuyaIndexCursor(state.markdown, state.selection)
    )
    super.clearHistory()
    return transaction
  }

  __installRustHooks () {
    const hooks = this.contentState
    hooks.updateParagraph = (type) => this.updateParagraph(type)
    hooks.format = (type) => this.format(type)
    hooks.clearBlockFormat = (_block, _cursor, type = 'clear') => this.format(type || 'clear')
    hooks.editTable = (data, key) => this.__editTable(data, key)
    hooks.updateImage = (info, attr, value) => this.__updateImage(info, attr, value)
    hooks.deleteImage = (info) => this.__deleteImage(info)
    hooks.createFootnote = (label) => this.__createFootnote(label)
    hooks.pasteHandler = (event, type = 'normal', text, html) => (
      this.__paste(event, type, text, html)
    )
    hooks.enterHandler = (event) => this.__enter(event)
    hooks.tabHandler = (event) => this.__tab(event)
    hooks.backspaceHandler = (event) => this.__backspace(event)
    hooks.docBackspaceHandler = (event) => this.__backspace(event)
    hooks.deleteHandler = (event) => this.__deleteForward(event)
    hooks.inputHandler = () => undefined
    hooks.duplicate = () => this.duplicate()
    hooks.deleteParagraph = () => this.deleteParagraph()
    hooks.insertParagraph = (location, text = '') => this.insertParagraph(location, text)
    hooks.updateCodeLanguage = (_block, language) => this.__setCodeLanguage(language)
    hooks.unlink = () => this.__unlink()
    hooks.listItemCheckBoxClick = () => this.__toggleTask()
  }

  __installRustEvents () {
    this.__beforeInputListener = (event) => this.__beforeInput(event)
    this.__pasteListener = (event) => this.__paste(event)
    this.__dropListener = (event) => this.__drop(event)
    this.__copyListener = (event) => this.__copy(event)
    this.__cutListener = (event) => this.__cut(event)
    this.__compositionStartListener = () => this.__compositionStart()
    this.__compositionEndListener = (event) => this.__compositionEnd(event)
    this.container.addEventListener('beforeinput', this.__beforeInputListener, true)
    this.container.addEventListener('paste', this.__pasteListener, true)
    this.container.addEventListener('drop', this.__dropListener, true)
    this.container.addEventListener('copy', this.__copyListener, true)
    this.container.addEventListener('cut', this.__cutListener, true)
    this.container.addEventListener('compositionstart', this.__compositionStartListener, true)
    this.container.addEventListener('compositionend', this.__compositionEndListener, true)
  }

  __beforeInput (event) {
    this.__onUserMutation?.(`beforeinput:${String(event?.inputType || '')}`)
    console.info('[elephantnote:muya-rust] beforeinput:received', {
      inputType: String(event?.inputType || ''),
      dataLength: event?.data == null ? 0 : String(event.data).length,
      active: Boolean(this.__rustMirror?.active),
      composition: Boolean(this.__rustComposition)
    })
    if (!this.__rustMirror?.active || this.__rustComposition) return
    const inputType = String(event.inputType || '')
    if (inputType === 'insertFromPaste' || inputType === 'insertCompositionText') return
    const text = event.data == null ? '' : String(event.data)
    const selectionState = this.__selection()
    const selection = selectionState.selection
    console.info('[elephantnote:muya-rust] beforeinput:operation', {
      inputType,
      textLength: text.length,
      markdownLength: selectionState.markdown.length,
      anchor: selection.anchor,
      focus: selection.focus,
      revision: this.__rustMirror?.state?.revision
    })
    const operations = {
      insertText: (engine) => engine.replaceRange(selection.anchor, selection.focus, text),
      insertParagraph: (engine) => engine.replaceRange(selection.anchor, selection.focus, '\n'),
      insertLineBreak: (engine) => engine.replaceRange(selection.anchor, selection.focus, '\n'),
      deleteContentBackward: (engine) => engine.deleteBackward(),
      deleteContentForward: (engine) => engine.deleteForward(),
      deleteByCut: (engine) => engine.replaceRange(selection.anchor, selection.focus, ''),
      historyUndo: (engine) => engine.undo(),
      historyRedo: (engine) => engine.redo(),
      formatBold: (engine) => engine.complete({ type: 'formatInline', format: 'strong' }),
      formatItalic: (engine) => engine.complete({ type: 'formatInline', format: 'em' }),
      formatStrikeThrough: (engine) => engine.complete({ type: 'formatInline', format: 'del' })
    }
    const operation = operations[inputType]
    if (!operation) {
      if (MUTATING_INPUT.test(inputType)) {
        event.preventDefault()
        event.stopImmediatePropagation()
        this.__reportRustError(new Error(`Unsupported browser mutation: ${inputType}`))
      }
      return
    }
    event.preventDefault()
    event.stopImmediatePropagation()
    this.__applyRust(`beforeinput-${inputType}`, operation)
      .then((transaction) => console.info('[elephantnote:muya-rust] beforeinput:applied', {
        inputType,
        documentChanged: Boolean(transaction?.documentChanged),
        markdownLength: transaction?.state?.markdown?.length,
        revision: transaction?.state?.revision
      }))
      .catch((error) => console.error('[elephantnote:muya-rust] beforeinput:failed', error))
  }

  __backspace (event) {
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    return this.__applyRust('backspace', (engine) => engine.deleteBackward())
  }

  __deleteForward (event) {
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    return this.__applyRust('delete-forward', (engine) => engine.deleteForward())
  }

  __enter (event) {
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    return this.__applyRust('enter', async(engine) => {
      const transaction = await engine.keyboardRule('Enter')
      if (transaction.documentChanged) return transaction
      const selection = this.__selection().selection
      return engine.replaceRange(selection.anchor, selection.focus, '\n')
    })
  }

  __tab (event) {
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    return this.__applyRust('tab', (engine) => engine.indentSelection({
      outdent: Boolean(event?.shiftKey),
      width: Number(this.contentState.tabSize) || 2
    }))
  }

  async __persistImage (file) {
    const invoke = getInvoke()
    if (!invoke) throw new Error('Tauri asset writer is unavailable.')
    const result = await invoke('tauri_muya_asset_write', {
      fileName: file.name || null,
      mimeType: file.type,
      bytes: await fileBytes(file)
    })
    await this.__applyRust('asset-image', (engine) => engine.complete({
      type: 'insertImage',
      alt: file.name || 'image',
      src: result.path,
      title: ''
    }))
  }

  __paste (event, type = 'normal', rawText, rawHtml) {
    const files = Array.from(event?.clipboardData?.files || [])
    const images = files.filter((file) => String(file.type || '').startsWith('image/'))
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    if (images.length) {
      Promise.all(images.map((file) => this.__persistImage(file))).catch(this.__reportRustError)
      return
    }
    const text = String(rawText ?? event?.clipboardData?.getData?.('text/plain') ?? '').replace(/\r/g, '')
    const html = type === 'pasteAsPlainText'
      ? ''
      : String(rawHtml ?? event?.clipboardData?.getData?.('text/html') ?? '').replace(/\r/g, '')
    return this.__applyRust('paste', (engine) => engine.pasteClipboard(html, text))
  }

  __drop (event) {
    const files = Array.from(event?.dataTransfer?.files || [])
    const images = files.filter((file) => String(file.type || '').startsWith('image/'))
    event.preventDefault()
    event.stopImmediatePropagation()
    if (images.length) {
      Promise.all(images.map((file) => this.__persistImage(file))).catch(this.__reportRustError)
      return
    }
    const text = String(event?.dataTransfer?.getData?.('text/plain') || '')
    if (!text) return
    this.__applyRust('drop', (engine) => {
      const selection = this.__selection().selection
      return engine.replaceRange(selection.anchor, selection.focus, text)
    }).catch(() => {})
  }

  async __refreshClipboard () {
    this.__rustClipboard = await this.__requireRust().clipboard()
    return this.__rustClipboard
  }

  __copy (event) {
    const payload = this.__rustClipboard
    if (!payload?.markdown && !payload?.html) return
    event.preventDefault()
    event.stopImmediatePropagation()
    event.clipboardData?.setData?.('text/plain', payload.markdown || '')
    event.clipboardData?.setData?.('text/html', payload.html || '')
  }

  __cut (event) {
    this.__copy(event)
    const selection = this.__selection().selection
    if (selection.anchor === selection.focus) return
    this.__applyRust('cut', (engine) => (
      engine.replaceRange(selection.anchor, selection.focus, '')
    )).catch(() => {})
  }

  __compositionStart () {
    if (!this.__rustMirror?.active || this.__rustComposition) return
    const current = this.__selection()
    this.__rustComposition = current
  }

  __compositionEnd (event) {
    const composition = this.__rustComposition
    if (!composition) return
    this.__rustComposition = null
    this.__applyRust('composition', (engine) => (
      engine.commitComposition(composition.selection, String(event?.data || ''))
    )).catch(() => {})
  }

  __tableContext (data, key) {
    const blockKey = key || this.contentState.cursor?.start?.key
    const block = blockKey ? this.contentState.getBlock(blockKey) : null
    if (!block || block.functionType !== 'cellContent') {
      throw new Error('A table cell must be active.')
    }
    const cell = this.contentState.getParent(block)
    const row = this.contentState.getParent(cell)
    const column = row?.children?.indexOf(cell)
    const cursor = this.contentState.getMuyaIndexCursor()
    const range = tableRangeAtLine(this.getMarkdown(), cursor?.anchor?.line ?? -1)
    if (!range || !Number.isInteger(column) || column < 0) {
      throw new Error('Unable to resolve the active table in the Rust document.')
    }
    const visualRow = (cursor?.anchor?.line ?? range.startLine) - range.startLine
    if (data?.target === 'row') {
      if (visualRow <= 1 && data.action === 'remove') {
        throw new Error('The Markdown table header and separator cannot be removed.')
      }
      const bodyIndex = Math.max(0, visualRow - 2)
      if (data.action === 'insert') {
        const index = data.location === 'previous' ? bodyIndex : bodyIndex + 1
        return { range, action: 'insert_row', index }
      }
      if (data.action === 'remove') return { range, action: 'delete_row', index: bodyIndex }
    }
    if (data?.target === 'column') {
      if (data.action === 'insert') {
        return {
          range,
          action: 'insert_column',
          index: data.location === 'left' ? column : column + 1
        }
      }
      if (data.action === 'remove') return { range, action: 'delete_column', index: column }
    }
    throw new Error('Unsupported table operation.')
  }

  __editTable (data, key) {
    const context = this.__tableContext(data, key)
    return this.__applyRust(`table-${context.action}`, (engine) => engine.complete({
      type: 'transformTable',
      start: context.range.start,
      end: context.range.end,
      action: context.action,
      index: context.index
    }))
  }

  __imageRange (info) {
    const current = this.__selection()
    return tokenRangeAtSelection(current.markdown, current.selection, info?.token)
  }

  __updateImage (info, attribute, value) {
    const range = this.__imageRange(info)
    return this.__applyRust(`image-${attribute}`, (engine) => engine.complete({
      type: 'updateImage',
      start: range.start,
      end: range.end,
      attribute: String(attribute),
      value: String(value)
    }))
  }

  __deleteImage (info) {
    const range = this.__imageRange(info)
    return this.__applyRust('image-delete', (engine) => (
      engine.replaceRange(range.start, range.end, '')
    ))
  }

  __createFootnote (label) {
    const value = String(label || '')
    if (!value || /[\]\r\n]/.test(value)) throw new Error('Invalid footnote label.')
    return this.__applyRust('footnote-create', (engine) => engine.upsertFootnote(value, ''))
  }

  __setCodeLanguage (language) {
    return this.__applyRust('code-language', (engine) => engine.setCodeLanguage(String(language || '')))
  }

  __unlink () {
    return this.__applyRust('unlink', (engine) => engine.removeLink())
  }

  __toggleTask () {
    return this.__applyRust('task-toggle', (engine) => engine.toggleTask())
  }

  setMarkdown (markdown, ...args) {
    const result = this.__setProgrammaticMarkdown(markdown, ...args)
    this.__rustComposition = null
    this.__rustMirror?.reset(markdown, 'set-markdown', { muyaIndexCursor: args[2] })
      .then(() => this.__refreshClipboard())
      .catch(this.__reportRustError)
    return result
  }

  undo () {
    return this.__applyRust('undo', (engine) => engine.undo())
  }

  redo () {
    return this.__applyRust('redo', (engine) => engine.redo())
  }

  format (type) {
    return this.__applyRust(`format-${type}`, (engine) => engine.complete({
      type: 'formatInline',
      format: String(type)
    }))
  }

  updateParagraph (type) {
    const kind = BLOCK_KINDS[type]
    if (!kind) throw new Error(`Unsupported block transformation: ${type}`)
    return this.__applyRust(`paragraph-${type}`, (engine) => engine.transformBlock(kind))
  }

  duplicate () {
    return this.__applyRust('duplicate', (engine) => engine.duplicateBlock())
  }

  deleteParagraph () {
    return this.__applyRust('delete-paragraph', (engine) => engine.deleteBlock())
  }

  insertParagraph (location, text = '') {
    return this.__applyRust(`insert-${location}`, (engine) => engine.insertParagraph(location, text))
  }

  editTable (data) {
    return this.__editTable(data)
  }

  createTable ({ rows = 2, columns = 2 } = {}) {
    return this.__applyRust('create-table', (engine) => engine.complete({
      type: 'createTable',
      rows: Number(rows) || 2,
      columns: Number(columns) || 2
    }))
  }

  insertImage ({ alt = '', src = '', title = '' } = {}) {
    return this.__applyRust('insert-image', (engine) => engine.complete({
      type: 'insertImage',
      alt: String(alt),
      src: String(src),
      title: String(title)
    }))
  }

  selectAll () {
    return this.__applyRust('select-all', (engine) => engine.selectAll())
  }

  async search (value, options = {}) {
    const query = String(value || '')
    const result = await this.__requireRust().searchMatches(query, {
      caseSensitive: Boolean(options.caseSensitive),
      wholeWord: Boolean(options.wholeWord)
    })
    const markdown = this.getMarkdown()
    const matches = (result?.matches || []).map(({ start, end }) => {
      const indexCursor = selectionToMuyaIndexCursor(markdown, { anchor: start, focus: end })
      const cursor = this.contentState.convertMuyaIndexCursortoCursor(indexCursor)
      return { start: cursor.start, end: cursor.end, active: false }
    })
    this.contentState.searchMatches = { value: query, matches, index: matches.length ? 0 : -1 }
    if (matches.length) matches[0].active = true
    this.contentState.render(Boolean(options.selectHighlight))
    return this.contentState.searchMatches
  }

  replace (value, options = {}) {
    return this.__applyRust('replace', (engine) => engine.searchReplace({
      query: String(value || ''),
      replacement: String(options.replaceValue ?? options.replacement ?? ''),
      replaceAll: Boolean(options.all || options.replaceAll),
      caseSensitive: Boolean(options.caseSensitive),
      wholeWord: Boolean(options.wholeWord)
    }))
  }

  find (action) {
    const search = this.contentState.searchMatches
    if (!search?.matches?.length) return search
    const delta = action === 'pre' ? -1 : 1
    search.index = (search.index + delta + search.matches.length) % search.matches.length
    search.matches.forEach((match, index) => { match.active = index === search.index })
    this.contentState.render(true)
    return search
  }

  async __writeClipboard () {
    const payload = await this.__refreshClipboard()
    if (globalThis.ClipboardItem && navigator.clipboard?.write) {
      const item = new ClipboardItem({
        'text/plain': new Blob([payload.markdown || ''], { type: 'text/plain' }),
        'text/html': new Blob([payload.html || ''], { type: 'text/html' })
      })
      await navigator.clipboard.write([item])
      return payload
    }
    await navigator.clipboard?.writeText?.(payload.markdown || '')
    return payload
  }

  copyAsRich () {
    return this.__writeClipboard()
  }

  copyAsHtml () {
    return this.__writeClipboard()
  }

  async pasteAsPlainText () {
    const text = await navigator.clipboard?.readText?.()
    return this.__applyRust('paste-plain-text', (engine) => engine.pasteClipboard('', text || ''))
  }

  replaceWordInline (text) {
    const selection = this.__selection().selection
    return this.__applyRust('replace-word', (engine) => (
      engine.replaceRange(selection.anchor, selection.focus, String(text || ''))
    ))
  }

  destroy () {
    this.off('change', this.__rustChangeListener)
    this.off('selectionChange', this.__rustSelectionListener)
    this.container.removeEventListener('beforeinput', this.__beforeInputListener, true)
    this.container.removeEventListener('paste', this.__pasteListener, true)
    this.container.removeEventListener('drop', this.__dropListener, true)
    this.container.removeEventListener('copy', this.__copyListener, true)
    this.container.removeEventListener('cut', this.__cutListener, true)
    this.container.removeEventListener('compositionstart', this.__compositionStartListener, true)
    this.container.removeEventListener('compositionend', this.__compositionEndListener, true)
    this.__rustMirror?.destroy()
    return super.destroy()
  }
}
