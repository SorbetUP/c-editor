import { createNodeElement } from './elements'
import { NODE_ATTRIBUTE, childElements } from './helpers'

export const applyDomPatch = (renderer, patch) => {
  switch (patch.type) {
    case 'replace_text':
      return syncText(renderer, patch.node)
    case 'insert_node': {
      const element = createNodeElement(renderer, renderer.logical.node(patch.node.id))
      return insertElement(renderer, patch.parent, patch.index, element)
    }
    case 'insert_subtree': {
      const element = renderer.renderSubtree(patch.subtree.root)
      return insertElement(renderer, patch.parent, patch.index, element)
    }
    case 'move_node':
      return insertElement(renderer, patch.new_parent, patch.new_index, renderer.requiredElement(patch.node))
    case 'remove_node':
      return removeElement(renderer, patch.node)
    case 'set_block_kind':
      return replaceElementShell(renderer, patch.node)
    default:
      throw new TypeError(`Unknown Muya Rust DOM patch: ${String(patch.type)}`)
  }
}

const syncText = (renderer, id) => {
  const node = renderer.logical.node(id)
  const element = renderer.requiredElement(id)
  const kind = node?.kind?.value || {}
  const value = kind.type === 'text' ? kind.value : kind.type === 'code_span' ? kind.code : null
  if (typeof value !== 'string') {
    throw new TypeError(`Muya Rust node ${id} is not editable inline content.`)
  }
  const text = element.firstChild
  if (!text || text.nodeType !== 3) {
    element.replaceChildren(renderer.ownerDocument.createTextNode(value))
  } else {
    text.data = value
  }
}

const insertElement = (renderer, parentId, index, element) => {
  const parent = renderer.requiredElement(parentId)
  const children = childElements(parent).filter((child) => child !== element)
  parent.insertBefore(element, children[index] || null)
}

const removeElement = (renderer, id) => {
  const element = renderer.requiredElement(id)
  const descendants = [element, ...element.querySelectorAll(`[${NODE_ATTRIBUTE}]`)]
  for (const descendant of descendants) {
    renderer.elements.delete(Number(descendant.getAttribute(NODE_ATTRIBUTE)))
  }
  element.remove()
}

const replaceElementShell = (renderer, id) => {
  const node = renderer.logical.node(id)
  const previous = renderer.requiredElement(id)
  const replacement = createNodeElement(renderer, node)
  for (const child of childElements(previous)) replacement.appendChild(child)
  previous.replaceWith(replacement)
  renderer.elements.set(id, replacement)
}
