import { createSearchLibrary, createStatus } from './searchLibrary'

export class ElephantSearchService {
  constructor(options = {}) {
    this._library = createSearchLibrary(options)
  }

  setEmbeddingProvider(embeddingProvider = null) {
    return this._library.setEmbeddingProvider(embeddingProvider)
  }

  initForVault(vaultRoot, windowId = null) {
    return this._library.initForVault(vaultRoot, windowId)
  }

  search(payload, windowId = null) {
    return this._library.search(payload, windowId)
  }

  indexFile(absolutePath, windowId = null) {
    return this._library.indexFile(absolutePath, windowId)
  }

  deleteFile(absolutePath, windowId = null) {
    return this._library.deleteFile(absolutePath, windowId)
  }

  rebuildIndex(windowId = null) {
    return this._library.rebuildIndex(windowId)
  }

  clearIndex(windowId = null) {
    return this._library.clearIndex(windowId)
  }

  getStatus(windowId = null) {
    return this._library.getStatus(windowId)
  }

  inspectIndex(windowId = null) {
    return this._library.inspectIndex(windowId)
  }

  disable() {
    return this._library.disable()
  }

  enable() {
    return this._library.enable()
  }

  registerWindowVault(windowId, vaultRoot) {
    return this._library.registerWindowVault(windowId, vaultRoot)
  }
}

export { createStatus }

