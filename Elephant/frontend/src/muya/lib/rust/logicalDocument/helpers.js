export const clone = (value) => JSON.parse(JSON.stringify(value))

export const assertNodeId = (value, label = 'node id') => {
  if (!Number.isSafeInteger(value) || value < 1) {
    throw new TypeError(`Invalid ${label}: ${String(value)}`)
  }
  return value
}

export const normalizeRevision = (value) => {
  const revision = Number(value)
  if (!Number.isSafeInteger(revision) || revision < 0) {
    throw new TypeError(`Invalid Muya Rust revision: ${String(value)}`)
  }
  return revision
}

export const utf16Boundary = (value, offset) => {
  if (!Number.isSafeInteger(offset) || offset < 0 || offset > value.length) return false
  if (offset === 0 || offset === value.length) return true
  const previous = value.charCodeAt(offset - 1)
  const current = value.charCodeAt(offset)
  const previousIsHigh = previous >= 0xd800 && previous <= 0xdbff
  const currentIsLow = current >= 0xdc00 && current <= 0xdfff
  return !(previousIsHigh && currentIsLow)
}

export const textValue = (node) => {
  if (node?.kind?.layer !== 'inline') {
    throw new TypeError(`Node ${String(node?.id)} is not editable inline content.`)
  }
  if (node.kind?.value?.type === 'text') return node.kind.value.value
  if (node.kind?.value?.type === 'code_span') return node.kind.value.code
  throw new TypeError(`Node ${String(node?.id)} is not editable inline content.`)
}

export const requiredNode = (document, id) => {
  assertNodeId(id)
  const node = document.nodes.get(id)
  if (!node) throw new Error(`Muya Rust node ${id} was not found.`)
  return node
}

export const assertInsertIndex = (parent, index) => {
  if (!Number.isSafeInteger(index) || index < 0 || index > parent.children.length) {
    throw new RangeError(`Invalid child index ${String(index)} for Muya Rust node ${parent.id}.`)
  }
}
