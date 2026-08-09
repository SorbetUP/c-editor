import { createEditorRequest, editorCommands, parseEditorResponse } from './protocol'

export class ElephantRustProtocolError extends Error {
  constructor(payload) {
    super(payload?.message || 'Elephant Rust rejected the editor command.')
    this.name = 'ElephantRustProtocolError'
    this.code = payload?.code || 'unknown'
  }
}

export class ElephantRustPatchError extends Error {
  constructor(cause) {
    super(`Unable to apply Elephant Rust patches: ${cause?.message || String(cause)}`)
    this.name = 'ElephantRustPatchError'
    this.cause = cause
  }
}

const noop = () => {}

export class ElephantRustBridge {
  constructor(engine, options = {}) {
    if (!engine || typeof engine.handle_json !== 'function') {
      throw new TypeError('Elephant Rust engine must expose handle_json(request).')
    }
    if (typeof engine.snapshot_json !== 'function') {
      throw new TypeError('Elephant Rust engine must expose snapshot_json().')
    }

    this.engine = engine
    this.revision = 0
    this.selection = null
    this.desynchronized = false
    this.applyPatches = options.applyPatches || noop
    this.applySnapshot = options.applySnapshot || noop
    this.onSelection = options.onSelection || noop
    this.onError = options.onError || noop
    this._tail = Promise.resolve()
  }

  dispatch(command) {
    return this._enqueue(() => this._dispatch(command))
  }

  setSelection(selection) {
    return this.dispatch(editorCommands.setSelection(selection))
  }

  snapshot() {
    return this._enqueue(() => this._snapshot())
  }

  recover() {
    return this._enqueue(async () => {
      const snapshot = await this._snapshot()
      this.desynchronized = false
      return snapshot
    })
  }

  async _dispatch(command) {
    if (this.desynchronized) {
      throw new ElephantRustPatchError(new Error('Bridge is desynchronized; call recover() first.'))
    }

    let response
    try {
      const request = createEditorRequest(this.revision, command)
      response = parseEditorResponse(await this.engine.handle_json(JSON.stringify(request)))
    } catch (error) {
      this.onError(error)
      throw error
    }

    if (response.type === 'error') {
      const error = new ElephantRustProtocolError(response.payload)
      this.onError(error)
      throw error
    }
    if (response.type === 'snapshot') {
      return this._acceptSnapshot(response.payload)
    }
    return this._acceptUpdate(response.payload)
  }

  async _snapshot() {
    let response
    try {
      response = parseEditorResponse(await this.engine.snapshot_json())
    } catch (error) {
      this.onError(error)
      throw error
    }
    if (response.type === 'error') {
      const error = new ElephantRustProtocolError(response.payload)
      this.onError(error)
      throw error
    }
    if (response.type !== 'snapshot') {
      throw new TypeError(`Expected a Elephant Rust snapshot, received ${response.type}.`)
    }
    return this._acceptSnapshot(response.payload)
  }

  async _acceptUpdate(update) {
    const revision = this._revision(update.revision)
    this.revision = revision
    this.selection = update.selection

    try {
      await this.applyPatches(update.patches || [], update)
      await this.onSelection(update.selection, update)
    } catch (error) {
      this.desynchronized = true
      const patchError = new ElephantRustPatchError(error)
      this.onError(patchError)
      throw patchError
    }
    return update
  }

  async _acceptSnapshot(snapshot) {
    const revision = this._revision(snapshot.revision)
    try {
      await this.applySnapshot(snapshot)
      await this.onSelection(snapshot.selection, snapshot)
    } catch (error) {
      this.desynchronized = true
      const patchError = new ElephantRustPatchError(error)
      this.onError(patchError)
      throw patchError
    }

    this.revision = revision
    this.selection = snapshot.selection
    return snapshot
  }

  _revision(value) {
    const revision = Number(value)
    if (!Number.isSafeInteger(revision) || revision < 0) {
      throw new TypeError(`Elephant Rust returned an invalid revision: ${String(value)}`)
    }
    return revision
  }

  _enqueue(task) {
    const result = this._tail.then(task)
    this._tail = result.catch(noop)
    return result
  }
}

export const createElephantRustBridge = async (factory, markdown, options = {}) => {
  if (typeof factory !== 'function') {
    throw new TypeError('experimentalRustEditor.factory must be a function.')
  }
  const engine = await factory(markdown)
  const bridge = new ElephantRustBridge(engine, options)
  await bridge.snapshot()
  return bridge
}

export { editorCommands }
