import { MuyaRustLogicalDocument } from '../logicalDocument'
import { createNodeElement } from './elements'
import { NODE_ATTRIBUTE } from './helpers'
import { applyDomPatch } from './patches'
import { restoreDomSelection } from './selection'
import { validateDom } from './validation'

export class MuyaRustDomRenderer {
  constructor(container, options = {}) {
    if (!container || typeof container.replaceChildren !== 'function') {
      throw new TypeError('Muya Rust DOM renderer requires an Element container.')
    }
    this.container = container
    this.ownerDocument = container.ownerDocument
    this.logical = new MuyaRustLogicalDocument()
    this.elements = new Map()
    this.onRender = options.onRender || null
  }

  applySnapshot(snapshot) {
    this.logical.loadSnapshot(snapshot)
    this.elements.clear()
    this.elements.set(this.logical.root, this.container)
    const root = this.logical.node(this.logical.root)
    const fragment = this.ownerDocument.createDocumentFragment()
    for (const childId of root.children) fragment.appendChild(this.renderSubtree(childId))
    this.container.replaceChildren(fragment)
    this.validateDom()
    this.onRender?.(this, snapshot)
  }

  applyPatches(patches, update) {
    this.logical.applyPatches(patches, update)
    for (const patch of patches) applyDomPatch(this, patch)
    this.validateDom()
    this.onRender?.(this, update)
  }

  restoreSelection(selection) {
    restoreDomSelection(this, selection)
  }

  element(id) {
    return this.elements.get(Number(id)) || null
  }

  requiredElement(id) {
    const element = this.element(id)
    if (!element) throw new Error(`Muya Rust DOM element ${String(id)} was not found.`)
    return element
  }

  renderSubtree(id) {
    const node = this.logical.node(id)
    if (!node) throw new Error(`Muya Rust logical node ${id} is missing.`)
    const element = createNodeElement(this, node)
    for (const childId of node.children) element.appendChild(this.renderSubtree(childId))
    return element
  }

  validateDom() {
    return validateDom(this)
  }
}

export const createDomPatchAdapter = (container, options = {}) => {
  const renderer = new MuyaRustDomRenderer(container, options)
  return {
    renderer,
    applySnapshot: (snapshot) => renderer.applySnapshot(snapshot),
    applyPatches: (patches, update) => renderer.applyPatches(patches, update),
    onSelection: (selection) => renderer.restoreSelection(selection)
  }
}

export { NODE_ATTRIBUTE }
