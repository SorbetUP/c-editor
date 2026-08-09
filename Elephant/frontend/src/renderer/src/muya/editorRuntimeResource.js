const BLOCK_SELECTOR = '[data-elephant-editor-layer="block"][data-elephant-editor-kind]'

const blockDescriptor = (element) => Object.freeze({
  nodeId: Number(element.getAttribute('data-elephant-editor-node')) || null,
  kind: element.getAttribute('data-elephant-editor-kind') || '',
  language: element.getAttribute('data-language') || '',
  element
})

export const createRustEditorRuntimeBinding = ({ runtime, getMarkdown = () => '' } = {}) => {
  if (!runtime?.bridge || typeof runtime.bridge.dispatch !== 'function') {
    throw new TypeError('A live Rust editor runtime is required')
  }

  const listeners = new Set()
  const root = runtime.domContainer || null
  let disposed = false

  const payload = (detail = {}) => Object.freeze({
    engine: 'rust',
    markdown: String(detail.markdown ?? getMarkdown() ?? ''),
    revision: runtime.bridge.revision,
    selection: runtime.bridge.selection,
    root,
    ...detail
  })

  const notify = (detail = {}) => {
    if (disposed) return
    const event = payload(detail)
    for (const listener of [...listeners]) listener(event)
  }

  const resource = Object.freeze({
    apiVersion: 1,
    owner: 'elephant.core.editor',
    engine: 'rust',
    root,
    getMarkdown: () => String(getMarkdown() ?? ''),
    snapshot: () => runtime.bridge.snapshot(),
    dispatch: (command) => runtime.bridge.dispatch(command),
    queryBlocks(options = {}) {
      const kind = typeof options === 'string' ? options : String(options.kind || '')
      const language = typeof options === 'object' ? String(options.language || '') : ''
      if (!root?.querySelectorAll) return []
      return [...root.querySelectorAll(BLOCK_SELECTOR)]
        .filter((element) => !kind || element.getAttribute('data-elephant-editor-kind') === kind)
        .filter((element) => !language || element.getAttribute('data-language') === language)
        .map(blockDescriptor)
    },
    watch(listener, options = {}) {
      if (typeof listener !== 'function') {
        throw new TypeError('Editor runtime listener must be a function')
      }
      listeners.add(listener)
      if (options.immediate !== false) listener(payload({ reason: 'attached' }))
      return () => listeners.delete(listener)
    }
  })

  return Object.freeze({
    resource,
    notify,
    dispose() {
      disposed = true
      listeners.clear()
    }
  })
}
