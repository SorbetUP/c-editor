import { childElements } from './helpers'

export const validateDom = (renderer) => {
  for (const [id, node] of renderer.logical.nodes) {
    const element = renderer.element(id)
    if (!element) throw new Error(`Muya Rust DOM element ${id} is missing.`)

    if (id !== renderer.logical.root) {
      const parent = renderer.element(node.parent)
      if (!parent) throw new Error(`Muya Rust DOM parent ${node.parent} is missing.`)
      if (element.parentElement !== parent) {
        throw new Error(`Muya Rust DOM node ${id} is attached to the wrong parent.`)
      }
    }

    const actualChildren = childElements(element).map((child) =>
      Number(child.getAttribute('data-muya-rust-id'))
    )
    if (actualChildren.length !== node.children.length) {
      throw new Error(`Muya Rust DOM node ${id} has the wrong child count.`)
    }
    node.children.forEach((childId, index) => {
      if (actualChildren[index] !== childId) {
        throw new Error(`Muya Rust DOM child order differs at node ${id}.`)
      }
    })
  }

  if (renderer.elements.size !== renderer.logical.nodes.size) {
    throw new Error('Muya Rust DOM registry contains stale elements.')
  }
  return true
}
