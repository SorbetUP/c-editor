import { assertNodeId, clone, normalizeRevision } from './helpers'

export const createSnapshotState = (snapshot) => {
  const document = snapshot?.document
  if (!document || !Array.isArray(document.nodes)) {
    throw new TypeError('Muya Rust snapshot has no logical document tree.')
  }

  const state = {
    root: assertNodeId(document.root, 'document root'),
    revision: normalizeRevision(snapshot.revision),
    nodes: new Map()
  }
  for (const rawNode of document.nodes) {
    const node = clone(rawNode)
    assertNodeId(node.id)
    if (state.nodes.has(node.id)) throw new TypeError(`Duplicate Muya Rust node id ${node.id}.`)
    if (!Array.isArray(node.children)) throw new TypeError(`Node ${node.id} has no children array.`)
    node.children.forEach((child) => assertNodeId(child, `child of ${node.id}`))
    if (node.parent !== null && node.parent !== undefined) {
      assertNodeId(node.parent, `parent of ${node.id}`)
    } else {
      node.parent = null
    }
    state.nodes.set(node.id, node)
  }
  validateLogicalDocument(state)
  return state
}

export const cloneLogicalState = (document) => ({
  root: document.root,
  revision: document.revision,
  nodes: new Map(Array.from(document.nodes, ([id, node]) => [id, clone(node)]))
})

export const validateLogicalDocument = (document) => {
  assertNodeId(document.root, 'document root')
  const root = document.nodes.get(document.root)
  if (!root) throw new Error(`Muya Rust root node ${document.root} is missing.`)
  if (root.parent !== null) throw new Error('Muya Rust root node must not have a parent.')

  const visited = new Set()
  const active = new Set()
  const visit = (id) => {
    if (active.has(id)) throw new Error(`Muya Rust logical tree contains a cycle at node ${id}.`)
    if (visited.has(id)) throw new Error(`Muya Rust node ${id} appears more than once.`)
    const node = document.nodes.get(id)
    if (!node) throw new Error(`Muya Rust child node ${id} is missing.`)
    active.add(id)
    visited.add(id)
    for (const childId of node.children) {
      const child = document.nodes.get(childId)
      if (!child) throw new Error(`Muya Rust child node ${childId} is missing.`)
      if (child.parent !== id) {
        throw new Error(`Muya Rust node ${childId} does not point back to parent ${id}.`)
      }
      visit(childId)
    }
    active.delete(id)
  }
  visit(document.root)
  if (visited.size !== document.nodes.size) {
    throw new Error('Muya Rust logical tree contains unreachable nodes.')
  }
  return true
}

export const toProtocolDocument = (document) => {
  const nodes = []
  const append = (id) => {
    const node = document.nodes.get(id)
    if (!node) throw new Error(`Missing Muya Rust node ${id}.`)
    nodes.push(clone(node))
    node.children.forEach(append)
  }
  append(document.root)
  return { root: document.root, nodes }
}
