export const restoreDomSelection = (renderer, selection) => {
  if (!selection) return
  const anchor = selectionPoint(renderer, selection.anchor)
  const focus = selectionPoint(renderer, selection.focus)
  const browserSelection = renderer.ownerDocument.defaultView?.getSelection?.()
  if (!browserSelection) return

  if (typeof browserSelection.setBaseAndExtent === 'function') {
    browserSelection.removeAllRanges()
    browserSelection.setBaseAndExtent(anchor.node, anchor.offset, focus.node, focus.offset)
    return
  }

  const range = renderer.ownerDocument.createRange()
  range.setStart(anchor.node, anchor.offset)
  range.setEnd(focus.node, focus.offset)
  browserSelection.removeAllRanges()
  browserSelection.addRange(range)
}

const selectionPoint = (renderer, point) => {
  const node = renderer.logical.node(point?.node)
  const element = renderer.requiredElement(point?.node)
  if (!['text', 'code_span'].includes(node?.kind?.value?.type)) {
    throw new TypeError(
      `Elephant Rust selection node ${String(point?.node)} is not editable inline content.`
    )
  }
  const text = element.firstChild
  if (!text || text.nodeType !== 3) {
    throw new TypeError(`Elephant Rust editable DOM for node ${point.node} is missing.`)
  }
  const offset = Number(point.offset_utf16)
  if (!Number.isSafeInteger(offset) || offset < 0 || offset > text.data.length) {
    throw new RangeError(`Elephant Rust selection offset ${String(point.offset_utf16)} is invalid.`)
  }
  return { node: text, offset }
}
