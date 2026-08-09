const requireInvoke = (target = globalThis) => {
  const ipcRenderer = target?.tauri?.ipcRenderer
  if (typeof ipcRenderer?.invoke === 'function') return ipcRenderer.invoke.bind(ipcRenderer)

  const core = target?.__TAURI__?.core
  if (typeof core?.invoke === 'function') return core.invoke.bind(core)

  throw new Error('Muya Rust engine requires the Tauri invoke bridge.')
}

const ensureState = (state) => {
  if (!state || typeof state !== 'object' || typeof state.markdown !== 'string') {
    throw new Error('Muya Rust engine returned an invalid editor state.')
  }
  return state
}

const ensureCommand = (command) => {
  if (!command || typeof command !== 'object' || typeof command.type !== 'string') {
    throw new Error('Muya Rust engine requires a typed editor command.')
  }
  return command
}

const ensureJsonState = (jsonState) => {
  if (!jsonState || jsonState.type !== 'muya-json-state' || !Array.isArray(jsonState.blocks)) {
    throw new Error('Muya Rust engine returned an invalid JSON document state.')
  }
  return jsonState
}

const ensureSelection = (selection) => {
  if (!selection || !Number.isInteger(selection.anchor) || !Number.isInteger(selection.focus)) {
    throw new Error('Muya Rust engine requires a valid UTF-16 selection.')
  }
  return { anchor: selection.anchor, focus: selection.focus }
}

const ensureSessionId = (sessionId) => {
  if (sessionId == null) return ''
  const value = String(sessionId)
  if (!/^[A-Za-z0-9:_-]{1,128}$/.test(value)) {
    throw new Error('Muya Rust engine requires a safe editor session id.')
  }
  return value
}

export const createRustMuyaEngineClient = ({
  invoke,
  target = globalThis,
  sessionId = null
} = {}) => {
  const call = invoke || requireInvoke(target)
  const editorId = ensureSessionId(sessionId)
  const usesSession = editorId.length > 0
  let state = null

  const create = async(markdown = '') => {
    const command = usesSession ? 'tauri_muya_session_create' : 'tauri_muya_engine_create'
    const payload = usesSession
      ? { editorId, markdown: String(markdown) }
      : { markdown: String(markdown) }
    state = ensureState(await call(command, payload))
    return state
  }

  const capabilities = async() => call('tauri_muya_engine_capabilities')

  const applyTransaction = async(commandName, command, extraPayload = {}) => {
    if (!state) throw new Error('Muya Rust engine must be initialized before applying commands.')
    const transaction = usesSession
      ? await call('tauri_muya_session_apply', {
        editorId,
        command: ensureCommand(command)
      })
      : await call(commandName, {
        state,
        command: ensureCommand(command),
        ...extraPayload
      })
    state = ensureState(transaction?.state)
    return transaction
  }

  const applySpecialized = async(sessionCommand, command) => {
    if (!usesSession) throw new Error('Complete Muya commands require a Rust-owned session.')
    if (!state) throw new Error('Muya Rust session must be initialized before applying commands.')
    const transaction = await call(sessionCommand, {
      editorId,
      command: ensureCommand(command)
    })
    state = ensureState(transaction?.state)
    return transaction
  }

  const query = async(queryCommand, { requireState = true } = {}) => {
    if (requireState && !state) {
      throw new Error('Muya Rust engine must be initialized before querying editor state.')
    }
    if (usesSession) {
      if (!state) throw new Error('Muya Rust session must be initialized before querying state.')
      return call('tauri_muya_session_query', {
        editorId,
        query: ensureCommand(queryCommand)
      })
    }
    return call('tauri_muya_engine_query', {
      state: state || null,
      query: ensureCommand(queryCommand)
    })
  }

  const apply = async(command) => applyTransaction('tauri_muya_engine_apply', command)
  const applyGrouped = async(command, continueGroup = false) => applyTransaction(
    'tauri_muya_engine_apply_grouped',
    command,
    { continueGroup: Boolean(continueGroup) }
  )
  const applyParity = async(command) => {
    if (!state) throw new Error('Muya Rust engine must be initialized before applying parity commands.')
    const transaction = usesSession
      ? await call('tauri_muya_session_apply_parity', {
        editorId,
        command: ensureCommand(command)
      })
      : await call('tauri_muya_engine_apply_parity', {
        state,
        command: ensureCommand(command)
      })
    state = ensureState(transaction?.state)
    return transaction
  }
  const applyComplete = async(command) => applySpecialized(
    'tauri_muya_session_apply_complete',
    command
  )

  const syncDocument = async(markdown, selection, continueGroup = false) => {
    if (!state) throw new Error('Muya Rust engine must be initialized before synchronizing a document.')
    const payload = {
      markdown: String(markdown),
      selection: ensureSelection(selection),
      continueGroup: Boolean(continueGroup)
    }
    const transaction = usesSession
      ? await call('tauri_muya_session_sync_document', { editorId, ...payload })
      : await call('tauri_muya_engine_sync_document', { state, ...payload })
    state = ensureState(transaction?.state)
    return transaction
  }

  const pasteClipboard = async(html = '', text = '') => {
    if (!state) throw new Error('Muya Rust engine must be initialized before pasting clipboard data.')
    const payload = { html: String(html), text: String(text) }
    const transaction = usesSession
      ? await call('tauri_muya_session_paste_clipboard', { editorId, ...payload })
      : await call('tauri_muya_engine_paste_clipboard', { state, ...payload })
    state = ensureState(transaction?.state)
    return transaction
  }

  const applyBatch = async(commands = []) => {
    if (usesSession) throw new Error('Batch commands are not available on Rust-owned Muya sessions.')
    if (!state) throw new Error('Muya Rust engine must be initialized before applying commands.')
    const transaction = await call('tauri_muya_engine_apply_batch', {
      state,
      commands: commands.map(ensureCommand)
    })
    state = ensureState(transaction?.state)
    return transaction
  }

  const commitComposition = async(selection, text) => {
    if (!state) throw new Error('Muya Rust engine must be initialized before committing composition.')
    const payload = { selection: ensureSelection(selection), text: String(text) }
    const transaction = usesSession
      ? await call('tauri_muya_session_commit_composition', { editorId, ...payload })
      : await call('tauri_muya_engine_commit_composition', { state, ...payload })
    state = ensureState(transaction?.state)
    return transaction
  }

  const reset = async(markdown = '') => create(markdown)
  const close = async() => {
    if (!usesSession) return false
    const closed = await call('tauri_muya_session_close', { editorId })
    state = null
    return Boolean(closed)
  }

  return {
    get state() { return state },
    get markdown() { return state?.markdown || '' },
    get sessionId() { return editorId || null },
    get usesSession() { return usesSession },
    create,
    reset,
    close,
    syncDocument,
    capabilities,
    apply,
    applyGrouped,
    applyParity,
    applyComplete,
    pasteClipboard,
    applyBatch,
    commitComposition,
    query,
    insertText: (text) => apply({ type: 'insertText', text: String(text) }),
    replaceSelection: (text) => apply({ type: 'replaceSelection', text: String(text) }),
    deleteBackward: () => apply({ type: 'deleteBackward' }),
    deleteForward: () => apply({ type: 'deleteForward' }),
    setSelection: (anchor, focus = anchor) => apply({ type: 'setSelection', anchor, focus }),
    toggleInline: (marker) => apply({ type: 'toggleInline', marker }),
    transformBlock: (kind) => apply({ type: 'transformBlock', kind }),
    insertLineBreak: () => apply({ type: 'insertLineBreak' }),
    undo: () => apply({ type: 'undo' }),
    redo: () => apply({ type: 'redo' }),
    applyOperation: (operation) => applyParity({ type: 'applyOperation', operation }),
    keyboardRule: (key, { shiftKey = false } = {}) => applyParity({
      type: 'keyboardRule',
      key: String(key),
      shiftKey: Boolean(shiftKey)
    }),
    tableCommand: (action, index = 0) => applyParity({
      type: 'tableCommand',
      action: String(action),
      index
    }),
    resizeImage: (cursor, width) => applyParity({
      type: 'resizeImage',
      cursor,
      width: String(width)
    }),
    upsertFootnote: (label, text) => applyParity({
      type: 'upsertFootnote',
      label: String(label),
      text: String(text)
    }),
    insertTemplate: (id) => applyParity({ type: 'insertTemplate', id: String(id) }),
    replaceRange: (start, end, text = '') => applyComplete({
      type: 'replaceRange',
      start,
      end,
      text: String(text)
    }),
    completeDeleteBackward: () => applyComplete({ type: 'deleteBackward' }),
    completeDeleteForward: () => applyComplete({ type: 'deleteForward' }),
    insertParagraph: (location, text = '') => applyComplete({
      type: 'insertParagraph',
      location: String(location),
      text: String(text)
    }),
    duplicateBlock: () => applyComplete({ type: 'duplicateBlock' }),
    deleteBlock: () => applyComplete({ type: 'deleteBlock' }),
    moveBlock: (fromStart, fromEnd, target) => applyComplete({
      type: 'moveBlock',
      fromStart,
      fromEnd,
      target
    }),
    indentSelection: ({ outdent = false, width = 2 } = {}) => applyComplete({
      type: 'indentSelection',
      outdent: Boolean(outdent),
      width
    }),
    toggleTask: () => applyComplete({ type: 'toggleTask' }),
    setCodeLanguage: (language = '') => applyComplete({
      type: 'setCodeLanguage',
      language: String(language)
    }),
    insertLink: (url, title = '') => applyComplete({
      type: 'insertLink',
      url: String(url),
      title: String(title)
    }),
    removeLink: () => applyComplete({ type: 'removeLink' }),
    searchReplace: ({
      query: searchQuery,
      replacement = '',
      replaceAll = false,
      caseSensitive = false,
      wholeWord = false
    }) => applyComplete({
      type: 'searchReplace',
      query: String(searchQuery),
      replacement: String(replacement),
      replaceAll: Boolean(replaceAll),
      caseSensitive: Boolean(caseSensitive),
      wholeWord: Boolean(wholeWord)
    }),
    selectAll: () => applyComplete({ type: 'selectAll' }),
    jsonState: async() => ensureJsonState(await query({ type: 'jsonState' })),
    clipboard: () => query({ type: 'clipboard' }),
    imageToolbar: (cursor = null) => query({ type: 'imageToolbar', cursor }),
    footnotePopup: (cursor = null) => query({ type: 'footnotePopup', cursor }),
    slashCommands: (queryText = '') => query({
      type: 'slashCommands',
      query: String(queryText)
    }, { requireState: false }),
    previewDescriptor: (blockType, language = null, text = '') => query({
      type: 'previewDescriptor',
      blockType: String(blockType),
      language: language == null ? null : String(language),
      text: String(text)
    }, { requireState: false })
  }
}

export const isRustMuyaEngineAvailable = (target = globalThis) => (
  typeof target?.tauri?.ipcRenderer?.invoke === 'function' ||
  typeof target?.__TAURI__?.core?.invoke === 'function'
)

