import { assertInsertIndex, assertNodeId, clone, requiredNode } from './helpers'

export const insertSubtree = (document, parentId, index, subtree) => {
  const parent = requiredNode(document, parentId)
  assertInsertIndex(parent, index)
  if (!subtree || !Array.isArray(subtree.nodes)) {
    throw new TypeError('Inserted Elephant Rust subtree is invalid.')
  }
  const rootId = assertNodeId(subtree.root, 'subtree root')
  const incoming = new Map()
  for (const rawNode of subtree.nodes) {
    const node = clone(rawNode)
    assertNodeId(node.id)
    if (incoming.has(node.id) || document.nodes.has(node.id)) {
      throw new Error(`Elephant Rust subtree node ${node.id} already exists.`)
    }
    incoming.set(node.id, node)
  }
  if (!incoming.has(rootId)) throw new Error(`Elephant Rust subtree root ${rootId} is missing.`)
  for (const node of incoming.values()) {
    if (!Array.isArray(node.children)) throw new Error(`Subtree node ${node.id} has no children.`)
    for (const childId of node.children) {
      const child = incoming.get(childId)
      if (!child || child.parent !== node.id) {
        throw new Error(`Elephant Rust subtree relation ${node.id} -> ${childId} is invalid.`)
      }
    }
    if (node.id !== rootId && !incoming.has(node.parent)) {
      throw new Error(`Elephant Rust subtree node ${node.id} has an external parent.`)
    }
  }
  incoming.get(rootId).parent = parent.id
  for (const [id, node] of incoming) document.nodes.set(id, node)
  parent.children.splice(index, 0, rootId)
}

export const moveNode = (document, nodeId, parentId, index) => {
  const node = requiredNode(document, nodeId)
  if (node.id === document.root) throw new Error('Elephant Rust root node cannot be moved.')
  const oldParent = requiredNode(document, node.parent)
  const oldIndex = oldParent.children.indexOf(node.id)
  if (oldIndex < 0) throw new Error(`Elephant Rust node ${node.id} is absent from its parent.`)
  oldParent.children.splice(oldIndex, 1)

  const parent = requiredNode(document, parentId)
  assertInsertIndex(parent, index)
  let ancestor = parent
  while (ancestor) {
    if (ancestor.id === node.id) throw new Error('Elephant Rust node cannot move into its subtree.')
    ancestor = ancestor.parent === null ? null : document.nodes.get(ancestor.parent)
  }
  parent.children.splice(index, 0, node.id)
  node.parent = parent.id
}

export const removeNode = (document, nodeId) => {
  const node = requiredNode(document, nodeId)
  if (node.id === document.root) throw new Error('Elephant Rust root node cannot be removed.')
  const parent = requiredNode(document, node.parent)
  const index = parent.children.indexOf(node.id)
  if (index < 0) throw new Error(`Elephant Rust node ${node.id} is absent from its parent.`)
  parent.children.splice(index, 1)

  const remove = (id) => {
    const current = requiredNode(document, id)
    current.children.slice().forEach(remove)
    document.nodes.delete(id)
  }
  remove(node.id)
}
