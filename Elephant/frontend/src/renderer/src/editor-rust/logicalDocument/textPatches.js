import {
  assertInsertIndex,
  assertNodeId,
  clone,
  requiredNode,
  textValue,
  utf16Boundary
} from './helpers'

export const replaceText = (document, patch) => {
  const node = requiredNode(document, patch.node)
  const value = textValue(node)
  const start = patch.range?.start
  const end = patch.range?.end
  if (!utf16Boundary(value, start) || !utf16Boundary(value, end) || start > end) {
    throw new RangeError(`Invalid UTF-16 range ${String(start)}..${String(end)} for node ${node.id}.`)
  }
  const replacement = value.slice(0, start) + String(patch.inserted) + value.slice(end)
  if (node.kind.value.type === 'text') node.kind.value.value = replacement
  else if (node.kind.value.type === 'code_span') node.kind.value.code = replacement
  else throw new TypeError(`Node ${node.id} is not editable inline content.`)
}

export const insertNode = (document, parentId, index, rawNode) => {
  const parent = requiredNode(document, parentId)
  assertInsertIndex(parent, index)
  const node = clone(rawNode)
  assertNodeId(node?.id)
  if (document.nodes.has(node.id)) throw new Error(`Elephant Rust node ${node.id} already exists.`)
  if (!Array.isArray(node.children) || node.children.length !== 0) {
    throw new Error(`Inserted Elephant Rust node ${node.id} must be detached and childless.`)
  }
  node.parent = parent.id
  document.nodes.set(node.id, node)
  parent.children.splice(index, 0, node.id)
}

export const setBlockKind = (document, nodeId, kind) => {
  const node = requiredNode(document, nodeId)
  if (node.kind?.layer !== 'block') {
    throw new TypeError(`Node ${node.id} is not a block node.`)
  }
  node.kind.value = clone(kind)
}
