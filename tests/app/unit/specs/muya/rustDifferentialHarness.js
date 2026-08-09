import { readFileSync } from 'fs'
import { resolve } from 'path'
import { vi } from 'vitest'

import Muya from '../../../../../Elephant/frontend/src/muya/lib'
import initWasm, { MuyaEditor } from 'muya-rust-wasm-bundle'

export const bundled = process.env.ELEPHANT_EXPERIMENTAL_RUST_EDITOR === '1'
let wasmInitialization = null

export const settle = async () => {
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 0))
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 20))
}

const collectBlocks = (muya, predicate) => {
  const output = []
  const visit = (block) => {
    if (predicate(block)) output.push(block)
    for (const child of block?.children || []) visit(child)
  }
  for (const block of muya.contentState.getBlocks()) visit(block)
  return output
}

const editableBlocks = (muya) =>
  collectBlocks(
    muya,
    (block) =>
      block?.functionType === 'paragraphContent' ||
      block?.functionType === 'cellContent'
  )

const allEditableTextBlocks = (muya) =>
  collectBlocks(muya, (block) => typeof block?.text === 'string')

const visibleText = (value) =>
  String(value || '').replace(/^ {0,3}#{1,6}[\s\u00a0]+/, '')

const applyJsSelection = (muya, block, start, end) => {
  muya.contentState.cursor = {
    start: { key: block.key, offset: start },
    end: { key: block.key, offset: end },
    isEdit: true
  }
  muya.contentState.setCursor()
}

export const setJsSelection = (muya, textIndex, start, end = start) => {
  const block = editableBlocks(muya)[textIndex]
  if (!block) throw new Error(`Muya JS text block ${textIndex} was not found.`)
  applyJsSelection(muya, block, start, end)
}

export const setJsSelectionByText = (muya, value, start, end = start, occurrence = 0) => {
  const matches = editableBlocks(muya).filter((block) => block.text === value)
  const block = matches[occurrence]
  if (!block) {
    throw new Error(
      `Muya JS text block ${JSON.stringify(value)} occurrence ${occurrence} was not found.`
    )
  }
  applyJsSelection(muya, block, start, end)
}

export const setJsSelectionByAnyText = (
  muya,
  value,
  start,
  end = start,
  occurrence = 0
) => {
  const expected = visibleText(value)
  const matches = allEditableTextBlocks(muya).filter(
    (block) => visibleText(block.text) === expected
  )
  const block = matches[occurrence]
  if (!block) {
    throw new Error(
      `Muya JS editable text ${JSON.stringify(value)} occurrence ${occurrence} was not found.`
    )
  }
  applyJsSelection(muya, block, start, end)
}

export const createJsEditor = async (markdown) => {
  document.body.innerHTML = '<div class="muya-differential-host"></div>'
  const muya = new Muya(document.querySelector('.muya-differential-host'), {
    markdown,
    t: (key) => key
  })
  await settle()
  return muya
}

const hasInlineAncestor = (node, nodesById, expectedType) => {
  let parent = node.parent
  while (parent !== null && parent !== undefined) {
    const ancestor = nodesById.get(parent)
    if (!ancestor) return false
    if (ancestor.kind?.layer === 'inline' && ancestor.kind?.value?.type === expectedType) {
      return true
    }
    parent = ancestor.parent
  }
  return false
}

export class RustTraceEditor {
  constructor(markdown) {
    this.engine = new MuyaEditor(markdown)
    this.revision = 0
  }

  request(command) {
    const response = JSON.parse(
      this.engine.handle_json(
        JSON.stringify({
          protocol_version: 1,
          expected_revision: this.revision,
          command
        })
      )
    )
    if (response.type === 'error') {
      throw new Error(`${response.payload.code}: ${response.payload.message}`)
    }
    this.revision = response.payload.revision ?? this.revision
    return response.payload
  }

  snapshot() {
    const response = JSON.parse(this.engine.snapshot_json())
    if (response.type !== 'snapshot') throw new Error(`Expected snapshot, got ${response.type}.`)
    this.revision = response.payload.revision
    return response.payload
  }

  textNodes(snapshot = this.snapshot()) {
    return snapshot.document.nodes.filter(
      (node) => node.kind?.layer === 'inline' && node.kind?.value?.type === 'text'
    )
  }

  codeSpanNodes(snapshot = this.snapshot()) {
    return snapshot.document.nodes.filter(
      (node) => node.kind?.layer === 'inline' && node.kind?.value?.type === 'code_span'
    )
  }

  textNodeByValue(value, occurrence = 0, snapshot = this.snapshot()) {
    const matches = this
      .textNodes(snapshot)
      .filter((candidate) => candidate.kind?.value?.value === value)
    const node = matches[occurrence]
    if (!node) {
      throw new Error(
        `Muya Rust text node ${JSON.stringify(value)} occurrence ${occurrence} was not found.`
      )
    }
    return node
  }

  codeSpanByValue(value, occurrence = 0, snapshot = this.snapshot()) {
    const matches = this
      .codeSpanNodes(snapshot)
      .filter((candidate) => candidate.kind?.value?.code === value)
    const node = matches[occurrence]
    if (!node) {
      throw new Error(
        `Muya Rust code span ${JSON.stringify(value)} occurrence ${occurrence} was not found.`
      )
    }
    return node
  }

  setSelection(textIndex, start, end = start) {
    const node = this.textNodes()[textIndex]
    if (!node) throw new Error(`Muya Rust text node ${textIndex} was not found.`)
    this.setSelectionOnNode(node, start, end)
  }

  setSelectionByText(value, start, end = start, occurrence = 0) {
    this.setSelectionOnNode(this.textNodeByValue(value, occurrence), start, end)
  }

  setSelectionByCode(value, start, end = start, occurrence = 0) {
    this.setSelectionOnNode(this.codeSpanByValue(value, occurrence), start, end)
  }

  setSelectionBetweenText(
    anchorValue,
    anchorOffset,
    focusValue,
    focusOffset,
    anchorOccurrence = 0,
    focusOccurrence = 0
  ) {
    const snapshot = this.snapshot()
    const anchor = this.textNodeByValue(anchorValue, anchorOccurrence, snapshot)
    const focus = this.textNodeByValue(focusValue, focusOccurrence, snapshot)
    this.request({
      type: 'set_selection',
      selection: {
        anchor: { node: anchor.id, offset_utf16: anchorOffset },
        focus: { node: focus.id, offset_utf16: focusOffset }
      }
    })
  }

  setSelectionByTextInMark(value, markType, start, end = start) {
    const snapshot = this.snapshot()
    const nodesById = new Map(snapshot.document.nodes.map((node) => [node.id, node]))
    const node = this.textNodes(snapshot).find(
      (candidate) =>
        candidate.kind?.value?.value === value &&
        hasInlineAncestor(candidate, nodesById, markType)
    )
    if (!node) {
      throw new Error(
        `Muya Rust text ${JSON.stringify(value)} inside ${markType} was not found.`
      )
    }
    this.setSelectionOnNode(node, start, end)
  }

  setSelectionOnNode(node, start, end) {
    this.request({
      type: 'set_selection',
      selection: {
        anchor: { node: node.id, offset_utf16: start },
        focus: { node: node.id, offset_utf16: end }
      }
    })
  }

  markdown() {
    return this.snapshot().markdown
  }
}

export const fakeKeyEvent = (overrides = {}) => ({
  preventDefault: vi.fn(),
  stopPropagation: vi.fn(),
  shiftKey: false,
  ...overrides
})

export const initializeRustWasm = () => {
  if (!wasmInitialization) {
    const wasm = readFileSync(
      resolve('Elephant/frontend/src/muya/lib/rust/generated/muya_wasm_bg.wasm')
    )
    wasmInitialization = initWasm({ module_or_path: wasm })
  }
  return wasmInitialization
}

export const runDifferentialTrace = async (trace) => {
  const jsEditor = await createJsEditor(trace.initial)
  const rustEditor = new RustTraceEditor(trace.initial)
  try {
    const jsResult = await trace.runJs(jsEditor)
    const rustResult = await trace.runRust(rustEditor)
    await settle()
    const jsMarkdown = jsEditor.getMarkdown()
    const rustMarkdown = rustEditor.markdown()
    return { jsEditor, rustEditor, jsMarkdown, rustMarkdown, jsResult, rustResult }
  } catch (error) {
    jsEditor.destroy?.()
    throw error
  }
}
