import RustOwnedMuya from './realMuyaRustAdapter.js'

const TABLE_ROW = /^\s*\|.*\|\s*$/
const TABLE_SEPARATOR = /^\s*\|(?:\s*:?-+:?\s*\|)+\s*$/
const IMAGE_URL = /\.(?:avif|gif|jpe?g|png|svg|webp)(?:[?#].*)?$/i
const ADVANCED_BLOCKS = Object.freeze({
  pre: { kind: 'code', language: '' },
  mathblock: { kind: 'math', language: '' },
  html: { kind: 'html', language: '' },
  flowchart: { kind: 'code', language: 'flowchart' },
  sequence: { kind: 'code', language: 'sequence' },
  plantuml: { kind: 'code', language: 'plantuml' },
  mermaid: { kind: 'code', language: 'mermaid' },
  'vega-lite': { kind: 'code', language: 'vega-lite' },
  hr: { kind: 'thematic-break', language: '' }
})

const markdownTableRanges = (markdown) => {
  const lines = String(markdown || '').split('\n')
  const ranges = []
  let offset = 0
  const offsets = lines.map((line) => {
    const result = offset
    offset += line.length + 1
    return result
  })
  for (let startLine = 0; startLine < lines.length - 1; startLine += 1) {
    if (!TABLE_ROW.test(lines[startLine]) || !TABLE_SEPARATOR.test(lines[startLine + 1])) continue
    let endLine = startLine + 1
    while (endLine + 1 < lines.length && TABLE_ROW.test(lines[endLine + 1])) endLine += 1
    ranges.push({
      start: offsets[startLine],
      end: offsets[endLine] + lines[endLine].length,
      startLine,
      endLine
    })
    startLine = endLine
  }
  return ranges
}

const selectedLineRange = (markdown, selection) => {
  const value = String(markdown || '')
  const selectionStart = Math.min(selection.anchor, selection.focus, value.length)
  const selectionEnd = Math.min(Math.max(selection.anchor, selection.focus), value.length)
  const start = value.lastIndexOf('\n', Math.max(0, selectionStart - 1)) + 1
  const nextBreak = value.indexOf('\n', selectionEnd)
  const end = nextBreak < 0 ? value.length : nextBreak
  return { start, end, content: value.slice(start, end) }
}

const headingKind = (markdown, selection, direction) => {
  const { content } = selectedLineRange(markdown, selection)
  const current = /^(?: {0,3})(#{1,6})\s/.exec(content)?.[1]?.length || 0
  const next = direction === 'upgrade'
    ? (current === 0 ? 6 : Math.max(1, current - 1))
    : (current === 0 ? 0 : current === 6 ? 0 : current + 1)
  return next === 0 ? 'paragraph' : `heading${next}`
}

const tableRangeById = (markdown, tableId) => {
  const tableElements = Array.from(document.querySelectorAll('table.ag-paragraph'))
  const tableIndex = tableElements.findIndex((table) => table.id === tableId)
  if (tableIndex < 0) throw new Error(`Unable to locate Muya table ${tableId}.`)
  const range = markdownTableRanges(markdown)[tableIndex]
  if (!range) throw new Error('Muya DOM table has no matching Markdown table.')
  return range
}

const cellPosition = (tableElement, key) => {
  const cell = tableElement.querySelector(`#${CSS.escape(key)}`)
  if (!cell) throw new Error(`Unable to locate selected table cell ${key}.`)
  const row = cell.closest('tr')
  const rows = Array.from(tableElement.querySelectorAll('tr'))
  return {
    row: rows.indexOf(row),
    column: Array.from(row.children).indexOf(cell)
  }
}

export default class CompleteMuyaWithRustCore extends RustOwnedMuya {
  __installRustHooks () {
    super.__installRustHooks()
    const hooks = this.contentState
    hooks.docEnterHandler = (event) => this.__docEnter(event)
    hooks.docDeleteHandler = (event) => this.__docDelete(event)
    hooks.selectLanguage = (_paragraph, language) => this.__setCodeLanguage(language)
    hooks.updateCodeLanguage = (_block, language) => this.__setCodeLanguage(language)
    hooks.codeBlockUpdate = (_block, code = '', language = '') => (
      this.__insertBlock('code', language, code)
    )
    hooks.switchTableData = () => this.__switchTableData()
    hooks.deleteSelectedTableCells = (isCut = false) => this.__clearSelectedTableCells(isCut)
    hooks.replaceWordInline = (_line, _wordCursor, replacement) => (
      this.replaceWordInline(replacement)
    )
    hooks._replaceCurrentWordInlineUnsafe = (_word, replacement) => {
      void this.replaceWordInline(replacement)
      return true
    }
    hooks.selectAll = () => this.selectAll()
    hooks.selectAllContent = () => this.selectAll()
    hooks.createTable = (tableChecker) => this.createTable(tableChecker)
    hooks.replaceImage = (imageInfo, image) => this.__replaceImage(imageInfo, image)
    hooks.setEmoji = (item) => this.__setEmoji(item)
  }

  __beforeInput (event) {
    this.__onUserMutation?.(`beforeinput:${String(event?.inputType || '')}`)
    if (!this.__rustMirror?.active || this.__rustComposition) return
    const inputType = String(event.inputType || '')
    if (inputType === 'insertFromPaste' || inputType === 'insertCompositionText') return
    const commands = {
      insertText: {
        type: 'smartInput',
        text: String(event.data ?? ''),
        autoPairBracket: this.options.autoPairBracket !== false,
        autoPairMarkdownSyntax: this.options.autoPairMarkdownSyntax !== false
      },
      insertParagraph: { type: 'smartEnter', shiftKey: false },
      insertLineBreak: { type: 'smartEnter', shiftKey: true },
      deleteContentBackward: { type: 'smartDeleteBackward' },
      deleteContentForward: { type: 'smartDeleteForward' }
    }
    const command = commands[inputType]
    if (!command) return super.__beforeInput(event)
    event.preventDefault()
    event.stopImmediatePropagation()
    this.__applyRust(`advanced-${inputType}`, (engine) => engine.complete(command)).catch(() => {})
  }

  __backspace (event) {
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    return this.__applyRust('smart-backspace', (engine) => engine.complete({
      type: 'smartDeleteBackward'
    }))
  }

  __deleteForward (event) {
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    return this.__applyRust('smart-delete-forward', (engine) => engine.complete({
      type: 'smartDeleteForward'
    }))
  }

  __enter (event) {
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    return this.__applyRust('smart-enter', (engine) => engine.complete({
      type: 'smartEnter',
      shiftKey: Boolean(event?.shiftKey)
    }))
  }

  __docEnter (event) {
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    const selectedImage = this.contentState.selectedImage
    if (!selectedImage) return
    const { imageId, ...imageInfo } = selectedImage
    const image = document.querySelector(`#${CSS.escape(imageId)}`)
    if (!image) return
    const rect = image.getBoundingClientRect()
    this.eventCenter.dispatch('muya-image-selector', {
      reference: {
        getBoundingClientRect() {
          return { ...rect, height: 0 }
        }
      },
      imageInfo,
      cb: () => {}
    })
    this.contentState.selectedImage = null
  }

  __docDelete (event) {
    event?.preventDefault?.()
    event?.stopImmediatePropagation?.()
    if (this.contentState.selectedImage) {
      return this.__deleteImage(this.contentState.selectedImage)
    }
    if (this.contentState.selectedTableCells) {
      return this.__clearSelectedTableCells(false)
    }
  }

  __insertBlock (kind, language = '', content = '') {
    return this.__applyRust(`insert-block-${kind}`, (engine) => engine.complete({
      type: 'insertBlock',
      kind: String(kind),
      language: String(language),
      content: String(content)
    }))
  }

  __transformAdvancedParagraph (type) {
    if (type === 'table') {
      return this.contentState.showTablePicker()
    }
    if (type === 'front-matter') {
      const markdown = this.getMarkdown()
      if (/^(?:---|\+\+\+|;;;|\{)\s*\n/.test(markdown)) return undefined
      return this.__applyRust('paragraph-front-matter', async(engine) => {
        await engine.setSelection(0, 0)
        return engine.complete({ type: 'insertBlock', kind: 'frontmatter', language: '', content: '' })
      })
    }
    if (type === 'upgrade heading' || type === 'degrade heading') {
      const current = this.__selection()
      const kind = headingKind(current.markdown, current.selection, type.startsWith('upgrade') ? 'upgrade' : 'degrade')
      return this.__applyRust(`paragraph-${type}`, (engine) => engine.transformBlock(kind))
    }
    const spec = ADVANCED_BLOCKS[type]
    if (!spec) return null
    const current = this.__selection()
    const range = selectedLineRange(current.markdown, current.selection)
    return this.__applyRust(`paragraph-${type}`, async(engine) => {
      await engine.setSelection(range.start, range.end)
      return engine.complete({
        type: 'insertBlock',
        kind: spec.kind,
        language: spec.language,
        content: range.content
      })
    })
  }

  updateParagraph (type) {
    const advanced = this.__transformAdvancedParagraph(String(type))
    if (advanced !== null) return advanced
    return super.updateParagraph(type)
  }

  __setCodeLanguage (language) {
    return this.__applyRust('code-language', (engine) => engine.setCodeLanguage(String(language || '')))
  }

  __setEmoji (item) {
    const alias = Array.isArray(item?.aliases) ? item.aliases[0] : ''
    if (!alias) return undefined
    return this.replaceWordInline(alias)
  }

  __switchTableData () {
    const drag = this.contentState.dragInfo
    if (!drag) return
    const markdown = this.getMarkdown()
    const range = tableRangeById(markdown, drag.tableId)
    const axis = drag.barType === 'bottom' ? 'column' : 'row'
    const mapRow = (index) => index === 0 ? 0 : index + 1
    const from = axis === 'row' ? mapRow(drag.index) : drag.index
    const to = axis === 'row' ? mapRow(drag.curIndex) : drag.curIndex
    return this.__applyRust(`reorder-table-${axis}`, (engine) => engine.complete({
      type: 'reorderTable',
      start: range.start,
      end: range.end,
      axis,
      from,
      to
    }))
  }

  __clearSelectedTableCells (isCut = false) {
    const selected = this.contentState.selectedTableCells
    if (!selected?.tableId || !Array.isArray(selected.cells)) return
    const markdown = this.getMarkdown()
    const range = tableRangeById(markdown, selected.tableId)
    const table = document.querySelector(`#${CSS.escape(selected.tableId)}`)
    if (!table) throw new Error('Unable to locate the selected table.')
    const cells = selected.cells.map(({ key }) => cellPosition(table, key))
    this.contentState.selectedTableCells = null
    return this.__applyRust('clear-table-cells', (engine) => engine.complete({
      type: 'clearTableCells',
      start: range.start,
      end: range.end,
      cells,
      cut: Boolean(isCut)
    }))
  }

  __replaceImage (imageInfo, { alt = '', src = '', title = '' } = {}) {
    const range = this.__imageRange(imageInfo)
    return this.__applyRust('replace-image', async(engine) => {
      await engine.setSelection(range.start, range.end)
      return engine.complete({
        type: 'insertImage',
        alt: String(alt),
        src: String(src),
        title: String(title)
      })
    })
  }

  __drop (event) {
    const uri = String(event?.dataTransfer?.getData?.('text/uri-list') || '')
      .split('\n')
      .map((line) => line.trim())
      .find((line) => line && !line.startsWith('#'))
    if (!uri) return super.__drop(event)
    event.preventDefault()
    event.stopImmediatePropagation()
    if (IMAGE_URL.test(uri)) {
      return this.__applyRust('drop-image-url', (engine) => engine.complete({
        type: 'insertImage',
        alt: '',
        src: uri,
        title: ''
      }))
    }
    return this.__applyRust('drop-link-url', (engine) => engine.complete({
      type: 'insertLink',
      url: uri,
      title: ''
    }))
  }

  replaceWordInline (text) {
    return this.__applyRust('replace-current-word', (engine) => engine.complete({
      type: 'replaceCurrentWord',
      text: String(text || '')
    }))
  }
}
