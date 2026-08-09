import { NODE_ATTRIBUTE } from '../domRenderer'

const SELECTION_BOUNDARY_CODES = new Set([
  'ELEPHANT_RUST_SELECTION_OUTSIDE_DOCUMENT',
  'ELEPHANT_RUST_SELECTION_UNAVAILABLE'
])

const selectionBoundaryError = (message, code) => {
  const error = new TypeError(message)
  error.code = code
  return error
}

export const isSelectionBoundaryError = (error) => SELECTION_BOUNDARY_CODES.has(error?.code)

const rustElement = (node) => {
  const element = node?.nodeType === 1 ? node : node?.parentElement
  return element?.closest?.(`[${NODE_ATTRIBUTE}]`) || null
}

const pointFromDom = (renderer, node, offset) => {
  const element = rustElement(node)
  if (!element) {
    throw selectionBoundaryError(
      'Browser selection is outside the Elephant Rust document.',
      'ELEPHANT_RUST_SELECTION_OUTSIDE_DOCUMENT'
    )
  }

  const id = Number(element.getAttribute(NODE_ATTRIBUTE))
  const logical = renderer.logical.node(id)
  if (!['text', 'code_span'].includes(logical?.kind?.value?.type)) {
    throw new TypeError(
      `Browser selection endpoint ${String(id)} is not editable Rust inline content.`
    )
  }

  const text = element.firstChild
  if (!text || text.nodeType !== 3) {
    throw new TypeError(`Elephant Rust editable DOM for node ${String(id)} is missing.`)
  }

  let utf16Offset
  if (node === text) {
    utf16Offset = offset
  } else if (node === element) {
    utf16Offset = offset === 0 ? 0 : text.data.length
  } else {
    throw new TypeError(`Browser selection endpoint for node ${String(id)} is unsupported.`)
  }

  if (!Number.isSafeInteger(utf16Offset) || utf16Offset < 0 || utf16Offset > text.data.length) {
    throw new RangeError(`Browser selection offset ${String(utf16Offset)} is invalid.`)
  }

  return { node: id, offset_utf16: utf16Offset }
}

export const readDomSelection = (renderer) => {
  const selection = renderer.ownerDocument.defaultView?.getSelection?.()
  if (!selection || !selection.anchorNode || !selection.focusNode) {
    throw selectionBoundaryError(
      'Browser selection is unavailable.',
      'ELEPHANT_RUST_SELECTION_UNAVAILABLE'
    )
  }

  return {
    anchor: pointFromDom(renderer, selection.anchorNode, selection.anchorOffset),
    focus: pointFromDom(renderer, selection.focusNode, selection.focusOffset)
  }
}
