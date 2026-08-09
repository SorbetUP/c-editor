import { ELEPHANTNOTE_API_ACTIONS as apiActions } from 'common/elephantnote/apiActions'

const getCore = (target) => target?.__TAURI__?.core

const invoke = async (target, command, payload = {}) => {
  const core = getCore(target)
  if (typeof core?.invoke !== 'function') {
    throw new Error(`Tauri command API is unavailable for ${command}`)
  }
  return core.invoke(command, payload)
}

const normalizePayload = (payload) =>
  payload && typeof payload === 'object' && !Array.isArray(payload) ? payload : {}

const normalizeDirectoryPayload = (payload = {}) => {
  if (typeof payload === 'string') return { relativePath: payload }
  return normalizePayload(payload)
}

const markdownValue = (payload = {}) => {
  if (typeof payload === 'string') return payload
  const value = normalizePayload(payload)
  return String(value.markdown ?? value.content ?? value.text ?? '')
}

const openVaultDirectory = async (target) => {
  const open = target?.__TAURI__?.dialog?.open
  if (typeof open !== 'function') throw new Error('Tauri dialog API is unavailable')
  const selected = await open({ directory: true, multiple: false })
  if (!selected) return null
  const vaultPath = typeof selected === 'string' ? selected : selected.path
  if (!vaultPath) return null
  return invoke(target, 'tauri_vaults_select_path', { vaultPath })
}

const dispatchApiAction = async (bridge, action, payload = {}) => {
  switch (action) {
    case apiActions.API_DESCRIBE:
      return bridge.api.describe()
    case apiActions.VAULTS_GET:
      return bridge.getVaults()
    case apiActions.VAULTS_SELECT:
      return bridge.selectVault()
    case apiActions.VAULTS_SET_ACTIVE:
      return bridge.setActiveVault(payload.vaultId)
    case apiActions.VAULTS_SET_ICON:
      return bridge.setVaultIcon(payload)
    case apiActions.VAULTS_SET_NAME:
      return bridge.setVaultName(payload)
    case apiActions.VAULTS_REMOVE:
      return bridge.removeVault(payload)
    case apiActions.DIRECTORY_LIST:
      return bridge.listDirectory(payload)
    case apiActions.NOTES_CREATE:
      return bridge.createNote(payload)
    case apiActions.NOTES_READ:
      return bridge.notes.read(payload)
    case apiActions.NOTES_WRITE:
      return bridge.notes.write(payload)
    case apiActions.FOLDERS_CREATE:
      return bridge.createFolder(payload)
    case apiActions.SIDEBAR_ATTACH:
      return bridge.attachSidebarEntry(payload)
    case apiActions.SIDEBAR_DETACH:
      return bridge.detachSidebarEntry(payload)
    case apiActions.ENTRIES_RENAME:
      return bridge.renameEntry(payload)
    case apiActions.ENTRIES_MOVE:
      return bridge.moveEntry(payload)
    case apiActions.ENTRIES_DELETE:
      return bridge.deleteEntry(payload)
    case apiActions.SEARCH_QUERY:
      return bridge.search.query(payload)
    case apiActions.SEARCH_STATUS:
      return bridge.search.status()
    case apiActions.FEATURES_GET:
      return bridge.features.get()
    case apiActions.FEATURES_SET:
      return bridge.features.set(payload.key, payload.enabled)
    case apiActions.ATOMIC_CATALOG_GET:
      return bridge.atomic.getCatalog()
    default:
      throw new Error(`Unsupported Elephant core API action: ${String(action)}`)
  }
}

const createAtomicFeatureApi = (target) => {
  const list = () => invoke(target, 'tauri_atomic_features_list')
  const get = (feature) => invoke(target, 'tauri_atomic_features_get', { feature })
  const toggle = (feature) => invoke(target, 'tauri_atomic_features_toggle', { feature })
  const set = (feature, enabled) => invoke(target, 'tauri_atomic_features_set', { feature, enabled })

  return {
    list,
    get,
    toggle,
    set,
    providers: async() => [],
    describeApi: async() => ({
      runtime: 'tauri',
      owner: 'elephant-core',
      actions: ['list', 'get', 'toggle', 'set']
    }),
    callApi: async(request = {}) => {
      const action = String(request.action || request.method || '')
      const args = Array.isArray(request.arguments) ? request.arguments : []
      if (action === 'list') return list()
      if (action === 'get') return get(args[0] ?? request.feature)
      if (action === 'toggle') return toggle(args[0] ?? request.feature)
      if (action === 'set') {
        return set(args[0] ?? request.feature, args[1] ?? request.enabled)
      }
      throw new Error(`Unsupported atomic feature action: ${action || 'missing'}`)
    }
  }
}

const createBridge = (target) => {
  const atomicFeatures = createAtomicFeatureApi(target)
  const bridge = {
    getVaults: () => invoke(target, 'tauri_vaults_get'),
    selectVault: () => openVaultDirectory(target),
    setActiveVault: (vaultId) => invoke(target, 'tauri_vaults_set_active', { vaultId }),
    setVaultIcon: (payload = {}) => invoke(target, 'tauri_vaults_set_icon', normalizePayload(payload)),
    setVaultName: (payload = {}) => invoke(target, 'tauri_vaults_set_name', normalizePayload(payload)),
    removeVault: (payload = {}) => invoke(target, 'tauri_vaults_remove', normalizePayload(payload)),
    listDirectory: (payload = {}) => invoke(target, 'tauri_directory_list', normalizeDirectoryPayload(payload)),
    createNote: (payload = {}) => invoke(target, 'tauri_notes_create', normalizePayload(payload)),
    createFolder: (payload = {}) => invoke(target, 'tauri_folders_create', normalizePayload(payload)),
    attachSidebarEntry: (payload = {}) => {
      const value = normalizePayload(payload)
      return invoke(target, 'tauri_sidebar_attach', {
        relativePath: value.relativePath,
        title: value.title,
        entryType: value.type ?? value.entryType
      })
    },
    detachSidebarEntry: (payload = {}) => invoke(target, 'tauri_sidebar_detach', normalizePayload(payload)),
    renameEntry: (payload = {}) => invoke(target, 'tauri_entries_rename', normalizePayload(payload)),
    moveEntry: (payload = {}) => invoke(target, 'tauri_entries_move', normalizePayload(payload)),
    deleteEntry: (payload = {}) => invoke(target, 'tauri_entries_delete', normalizePayload(payload)),
    notes: {
      read: (payload = {}) => invoke(target, 'tauri_notes_read', normalizePayload(payload)),
      write: (payload = {}) => {
        const value = normalizePayload(payload)
        return invoke(target, 'tauri_notes_write', {
          relativePath: value.relativePath,
          markdown: markdownValue(value)
        })
      }
    },
    markdown: {
      parse: (payload = {}) => invoke(target, 'tauri_markdown_parse', { markdown: markdownValue(payload) }),
      renderHtml: (payload = {}) => invoke(target, 'tauri_markdown_render_html', { markdown: markdownValue(payload) }),
      toText: (payload = {}) => invoke(target, 'tauri_markdown_to_text', { markdown: markdownValue(payload) }),
      extractFrontmatter: (payload = {}) => invoke(target, 'tauri_markdown_extract_frontmatter', { markdown: markdownValue(payload) }),
      extractLinks: (payload = {}) => invoke(target, 'tauri_markdown_extract_links', { markdown: markdownValue(payload) })
    },
    muya: {
      parse: (payload = {}) => invoke(target, 'tauri_muya_parse', normalizePayload(payload)),
      renderHtml: (payload = {}) => invoke(target, 'tauri_muya_render_html', normalizePayload(payload)),
      tokens: (payload = {}) => invoke(target, 'tauri_muya_tokens', normalizePayload(payload)),
      extras: (payload = {}) => invoke(target, 'tauri_muya_extras', normalizePayload(payload)),
      contract: (payload = {}) => invoke(target, 'tauri_muya_contract', normalizePayload(payload)),
      clipboard: (payload = {}) => invoke(target, 'tauri_muya_clipboard', normalizePayload(payload)),
      copyMarkdown: (payload = {}) => invoke(target, 'tauri_muya_copy_markdown', normalizePayload(payload)),
      copyHtml: (payload = {}) => invoke(target, 'tauri_muya_copy_html', normalizePayload(payload)),
      paste: (payload = {}) => invoke(target, 'tauri_muya_paste', normalizePayload(payload)),
      backspace: (payload = {}) => invoke(target, 'tauri_muya_backspace', normalizePayload(payload)),
      removeNext: (payload = {}) => invoke(target, 'tauri_muya_remove_next', normalizePayload(payload)),
      undo: (payload = {}) => invoke(target, 'tauri_muya_undo', normalizePayload(payload)),
      redo: (payload = {}) => invoke(target, 'tauri_muya_redo', normalizePayload(payload)),
      moveCursor: (payload = {}) => invoke(target, 'tauri_muya_move_cursor', normalizePayload(payload)),
      inputRule: (payload = {}) => invoke(target, 'tauri_muya_input_rule', normalizePayload(payload)),
      tableInsertRow: (payload = {}) => invoke(target, 'tauri_muya_table_insert_row', normalizePayload(payload)),
      tableInsertColumn: (payload = {}) => invoke(target, 'tauri_muya_table_insert_column', normalizePayload(payload)),
      tableContract: (payload = {}) => invoke(target, 'tauri_muya_table_contract', normalizePayload(payload)),
      imageSelection: (payload = {}) => invoke(target, 'tauri_muya_image_selection', normalizePayload(payload)),
      startComposition: (payload = {}) => invoke(target, 'tauri_muya_start_composition', normalizePayload(payload)),
      updateComposition: (payload = {}) => invoke(target, 'tauri_muya_update_composition', normalizePayload(payload)),
      commitComposition: (payload = {}) => invoke(target, 'tauri_muya_commit_composition', normalizePayload(payload)),
      cancelComposition: (payload = {}) => invoke(target, 'tauri_muya_cancel_composition', normalizePayload(payload)),
      editorSnapshot: (payload = {}) => invoke(target, 'tauri_muya_editor_snapshot', normalizePayload(payload))
    },
    attachments: {
      list: () => invoke(target, 'tauri_attachments_list'),
      writeText: (payload = {}) => invoke(target, 'tauri_attachments_write_text', normalizePayload(payload))
    },
    drawings: {
      list: () => invoke(target, 'tauri_drawings_list'),
      create: (payload = {}) => invoke(target, 'tauri_drawings_create', normalizePayload(payload)),
      read: (payload = {}) => invoke(target, 'tauri_drawings_read', normalizePayload(payload)),
      write: (payload = {}) => invoke(target, 'tauri_drawings_write', normalizePayload(payload))
    },
    sources: {
      list: () => invoke(target, 'tauri_sources_list')
    },
    search: {
      query: (payload = {}) => invoke(target, 'tauri_search_query', { params: normalizePayload(payload) }),
      status: () => invoke(target, 'tauri_search_status')
    },
    features: {
      get: () => invoke(target, 'tauri_features_get'),
      set: (key, enabled) => invoke(target, 'tauri_features_set', { key, enabled })
    },
    atomic: {
      getCatalog: () => atomicFeatures.list()
    },
    atomicFeatures,
    clipboard: {
      writeText: async(text) => target.__TAURI__?.clipboardManager?.writeText?.(text),
      readText: async() => target.__TAURI__?.clipboardManager?.readText?.()
    }
  }

  bridge.api = {
    describe: async() => ({
      runtime: 'tauri',
      backend: 'rust',
      bridge: 'elephantnote-tauri-core',
      actions: apiActions
    }),
    call: async(action, payload = {}) => ({
      ok: true,
      data: await dispatchApiAction(bridge, action, normalizePayload(payload))
    })
  }

  return bridge
}

export const installTauriElephantNoteBridge = (target = globalThis) => {
  if (!target?.__TAURI__ || target?.elephantnote?.getVaults) return false
  target.elephantnote = {
    ...(target.elephantnote || {}),
    ...createBridge(target)
  }
  return true
}
