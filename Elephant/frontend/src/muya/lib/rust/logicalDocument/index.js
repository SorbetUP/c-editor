import { applyLogicalPatch } from './applyPatch'
import { normalizeRevision } from './helpers'
import {
  cloneLogicalState,
  createSnapshotState,
  toProtocolDocument,
  validateLogicalDocument
} from './validation'

export class MuyaRustLogicalDocument {
  constructor(snapshot = null) {
    this.root = null
    this.revision = 0
    this.nodes = new Map()
    if (snapshot) this.loadSnapshot(snapshot)
  }

  loadSnapshot(snapshot) {
    this._replaceWith(createSnapshotState(snapshot))
    return this
  }

  applyPatches(patches, update) {
    if (!Array.isArray(patches)) throw new TypeError('Muya Rust patches must be an array.')
    const revision = normalizeRevision(update?.revision)
    const expectedRevision = this.revision + (patches.length > 0 ? 1 : 0)
    if (revision !== expectedRevision) {
      throw new Error(
        `Muya Rust logical revision mismatch: expected ${expectedRevision}, received ${revision}.`
      )
    }

    const next = this.clone()
    for (const patch of patches) applyLogicalPatch(next, patch)
    next.revision = revision
    next.validate()
    this._replaceWith(next)
    return this
  }

  node(id) {
    return this.nodes.get(id) || null
  }

  clone() {
    const next = new MuyaRustLogicalDocument()
    next._replaceWith(cloneLogicalState(this))
    return next
  }

  toProtocolDocument() {
    return toProtocolDocument(this)
  }

  validate() {
    return validateLogicalDocument(this)
  }

  _replaceWith(other) {
    this.root = other.root
    this.revision = other.revision
    this.nodes = other.nodes
  }
}

export const createLogicalPatchAdapter = (options = {}) => {
  const document = new MuyaRustLogicalDocument()
  return {
    document,
    applySnapshot: async (snapshot) => {
      document.loadSnapshot(snapshot)
      await options.onCommit?.(document, snapshot)
    },
    applyPatches: async (patches, update) => {
      document.applyPatches(patches, update)
      await options.onCommit?.(document, update)
    }
  }
}
