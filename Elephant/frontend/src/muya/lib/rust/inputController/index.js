import { editorCommands } from '../bridge'
import { markdownFromClipboard } from './clipboard'
import {
  cancelComposition,
  finishComposition,
  replaceComposition,
  startComposition
} from './composition'
import { commandForBeforeInput, commandForTableKey } from './commands'
import { handleCopy, handleCut } from './copyCut'
import { handleDeleteForward } from './deleteForward'
import { DELETE_UNIT_INPUTS, handleDeleteUnit } from './deleteUnits'
import { handleDragOver, handleDrop } from './drop'
import { handleImageClick } from './image'
import { readDomSelection } from './selection'
import { handleTaskClick } from './task'

const noop = () => {}
const NAVIGATION_KEYS = new Set([
  'ArrowLeft',
  'ArrowRight',
  'ArrowUp',
  'ArrowDown',
  'Home',
  'End',
  'PageUp',
  'PageDown'
])

export class MuyaRustInputController {
  constructor(container, bridge, renderer, options = {}) {
    if (!container?.addEventListener) {
      throw new TypeError('Rust input controller requires a DOM container.')
    }
    if (!bridge?.dispatch || !bridge?.setSelection) {
      throw new TypeError('Rust input controller requires a MuyaRustBridge.')
    }
    if (!renderer?.logical) throw new TypeError('Rust input controller requires a DOM renderer.')

    this.container = container
    this.bridge = bridge
    this.renderer = renderer
    this.onError = options.onError || noop
    this.onFileDrop = typeof options.onFileDrop === 'function' ? options.onFileDrop : null
    this.onUriDrop = typeof options.onUriDrop === 'function' ? options.onUriDrop : null
    this.onImageClick = typeof options.onImageClick === 'function' ? options.onImageClick : null
    this.autoCheck = Boolean(options.autoCheck)
    this.composition = null
    this.attached = false
    this._tail = Promise.resolve()
    this._inputSelection = null
    this._beforeInput = (event) => this.handleBeforeInput(event)
    this._click = (event) => this.handleClick(event)
    this._copy = (event) => this.handleCopy(event)
    this._cut = (event) => this.handleCut(event)
    this._paste = (event) => this.handlePaste(event)
    this._dragOver = (event) => this.handleDragOver(event)
    this._drop = (event) => this.handleDrop(event)
    this._compositionStart = () => this.handleCompositionStart()
    this._compositionEnd = (event) => this.handleCompositionEnd(event)
    this._keyDown = (event) => this.handleKeyDown(event)
  }

  attach() {
    if (this.attached) return this
    this.attached = true
    this.container.addEventListener('beforeinput', this._beforeInput)
    this.container.addEventListener('click', this._click)
    this.container.addEventListener('copy', this._copy)
    this.container.addEventListener('cut', this._cut)
    this.container.addEventListener('paste', this._paste)
    this.container.addEventListener('dragover', this._dragOver)
    this.container.addEventListener('drop', this._drop)
    this.container.addEventListener('compositionstart', this._compositionStart)
    this.container.addEventListener('compositionend', this._compositionEnd)
    this.container.addEventListener('keydown', this._keyDown)
    return this
  }

  detach() {
    if (!this.attached) return this
    this.attached = false
    this.container.removeEventListener('beforeinput', this._beforeInput)
    this.container.removeEventListener('click', this._click)
    this.container.removeEventListener('copy', this._copy)
    this.container.removeEventListener('cut', this._cut)
    this.container.removeEventListener('paste', this._paste)
    this.container.removeEventListener('dragover', this._dragOver)
    this.container.removeEventListener('drop', this._drop)
    this.container.removeEventListener('compositionstart', this._compositionStart)
    this.container.removeEventListener('compositionend', this._compositionEnd)
    this.container.removeEventListener('keydown', this._keyDown)
    this._inputSelection = null
    return this
  }

  idle() {
    return this._tail
  }

  handleBeforeInput(event) {
    if (event.inputType === 'insertCompositionText') {
      if (!this.composition) startComposition(this)
      event.preventDefault()
      replaceComposition(this, event.data || '')
      return
    }

    if (event.inputType === 'deleteContentForward') {
      const selection = this.readSelection()
      if (selection) handleDeleteForward(this, event, selection)
      return
    }
    if (DELETE_UNIT_INPUTS.has(event.inputType)) {
      const selection = this.readSelection()
      if (selection) handleDeleteUnit(this, event, selection)
      return
    }

    const command = commandForBeforeInput(event)
    if (!command) return
    const observedSelection = this.readSelection()
    if (!observedSelection) return
    event.preventDefault()

    this.schedule(async () => {
      // A burst of beforeinput events can be delivered before the DOM selection
      // has caught up with the asynchronous Rust/WASM patch cycle. The first
      // command uses the browser caret; every following command continues from
      // the logical selection returned by the previous Rust update.
      const selection = this._inputSelection || observedSelection
      await this.bridge.setSelection(selection)
      await this.bridge.dispatch(command)
      this._inputSelection = this.bridge.selection || selection
    })
  }

  handleClick(event) {
    this._inputSelection = null
    return handleTaskClick(this, event) || handleImageClick(this, event)
  }

  handleCopy(event) {
    return handleCopy(this, event)
  }

  handleCut(event) {
    return handleCut(this, event)
  }

  handlePaste(event) {
    const observedSelection = this.readSelection()
    if (!observedSelection) return
    const markdown = markdownFromClipboard(event, this.container.ownerDocument)
    if (markdown === null) return
    event.preventDefault()
    event.stopPropagation?.()
    this.schedule(async () => {
      const selection = this._inputSelection || observedSelection
      await this.bridge.setSelection(selection)
      await this.bridge.dispatch(editorCommands.pasteMarkdown(markdown))
      this._inputSelection = this.bridge.selection || selection
    })
  }

  handleDragOver(event) {
    return handleDragOver(this, event)
  }

  handleDrop(event) {
    this._inputSelection = null
    return handleDrop(this, event)
  }

  handleCompositionStart() {
    this._inputSelection = null
    startComposition(this)
  }

  handleCompositionEnd(event) {
    finishComposition(this, event)
    this._inputSelection = this.bridge.selection || null
  }

  handleKeyDown(event) {
    if (NAVIGATION_KEYS.has(event.key)) {
      this._inputSelection = null
    }
    if (event.key === 'Escape' && cancelComposition(this)) {
      event.preventDefault()
      return
    }
    if (
      event.key === 'Enter' &&
      !event.isComposing &&
      !this.composition &&
      !event.metaKey &&
      !event.ctrlKey &&
      !event.altKey
    ) {
      const selection = this.readSelection()
      if (!selection) return
      event.preventDefault()
      this.schedule(async () => {
        const nextSelection = this._inputSelection || selection
        await this.bridge.setSelection(nextSelection)
        await this.bridge.dispatch(editorCommands.insertParagraph())
        this._inputSelection = this.bridge.selection || nextSelection
      })
      return
    }
    if (event.key !== 'Tab') return

    const selection = this.readSelection()
    if (!selection) return
    const command = commandForTableKey(event, this.renderer, selection)
    if (!command) return
    event.preventDefault()
    this.schedule(async () => {
      await this.bridge.setSelection(selection)
      await this.bridge.dispatch(command)
      this._inputSelection = this.bridge.selection || selection
    })
  }

  readSelection() {
    try {
      return readDomSelection(this.renderer)
    } catch (error) {
      this.onError(error)
      return null
    }
  }

  schedule(task) {
    const result = this._tail.then(task)
    this._tail = result.catch((error) => this.onError(error))
    result.catch(noop)
    return result
  }
}

export const createRustInputController = (container, bridge, renderer, options) =>
  new MuyaRustInputController(container, bridge, renderer, options).attach()
