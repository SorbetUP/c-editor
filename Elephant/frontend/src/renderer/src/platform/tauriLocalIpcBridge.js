const LOCAL_IPC_EVENTS = new Set([
  'mt::response-file-save',
  'mt::response-file-save-as',
  'mt::save-tabs',
  'mt::save-and-close-tabs',
  'mt::tab-saved',
  'mt::tab-save-failure',
  'mt::force-close-tabs-by-id'
])

const NOTE_OPEN_EVENTS = new Set([
  'mt::open-file',
  'mt::open-file-by-window-id'
])

const ASSET_EXTENSION_RE = /\.(?:png|jpe?g|gif|webp|svg|avif|bmp|ico|pdf|excalidraw)$/i
const MARKDOWN_EXTENSION_RE = /\.md$/i

const normalizePath = (value = '') => String(value || '').replace(/\\/g, '/').replace(/\/+/g, '/')

const trimTrailingSlash = (value = '') => normalizePath(value).replace(/\/+$/g, '')

const isAbsolutePath = (value = '') => {
  const normalized = normalizePath(value)
  return normalized.startsWith('/') || /^[a-zA-Z]:\//.test(normalized)
}

const isMarkdownPath = (value = '') => MARKDOWN_EXTENSION_RE.test(normalizePath(value))
const isAssetPath = (value = '') => ASSET_EXTENSION_RE.test(normalizePath(value))

const fallbackRelativePath = (vaultRoot = '', filePath = '') => {
  const root = trimTrailingSlash(vaultRoot)
  const file = normalizePath(filePath)
  if (!root || !file || file === root) return ''
  const prefix = `${root}/`
  if (!file.startsWith(prefix)) return ''
  return file.slice(prefix.length)
}

const dispatchLocalIpcEvent = (target, channel, args) => {
  target.dispatchEvent(new CustomEvent(channel, { detail: args }))
}

const getBasename = (target, pathname = '') => (
  target.path?.basename?.(pathname) || normalizePath(pathname).split('/').filter(Boolean).pop() || 'Untitled.md'
)

const getRelativeVaultPath = (target, vaultRoot = '', filePath = '') => {
  if (!vaultRoot || !filePath) return ''

  const relative = target.path?.relative?.(vaultRoot, filePath) || fallbackRelativePath(vaultRoot, filePath)
  if (!relative || relative.startsWith('..') || isAbsolutePath(relative)) return ''
  return normalizePath(relative)
}

const readVaultNote = async(target, relativePath) => {
  const invoke = target.__TAURI__?.core?.invoke
  if (typeof invoke === 'function') {
    const result = await invoke('tauri_notes_read', { relativePath })
    if (result != null) return result
  }
  if (typeof target.elephantnote?.notes?.read === 'function') {
    return target.elephantnote.notes.read({ relativePath })
  }
  if (typeof target.elephantnote?.readNote === 'function') {
    return target.elephantnote.readNote({ relativePath })
  }
  return null
}

const openAssetExternally = async(target, filePath, channel) => {
  console.info('[tauri:local-ipc] open-file ignored non-markdown asset', { channel, filePath: normalizePath(filePath) })
  const opener = target.__TAURI__?.opener?.openUrl || target.tauri?.shell?.openPath || target.tauri?.shell?.openExternal
  if (typeof opener === 'function') {
    try {
      await opener(filePath)
    } catch (error) {
      console.warn('[tauri:local-ipc] external asset open failed', {
        channel,
        filePath: normalizePath(filePath),
        error: error?.message || String(error)
      })
    }
  }
  return true
}

const openVaultNoteWithBackend = async(target, channel, args) => {
  const filePath = args[0]
  if (!filePath || typeof target.elephantnote?.getVaults !== 'function') return false

  if (isAssetPath(filePath) || !isMarkdownPath(filePath)) {
    return openAssetExternally(target, filePath, channel)
  }

  try {
    const payload = await target.elephantnote.getVaults()
    const vaultRoot = payload?.activeVault?.path || ''
    const relativePath = getRelativeVaultPath(target, vaultRoot, filePath)
    if (!relativePath) return false
    if (!isMarkdownPath(relativePath)) {
      return openAssetExternally(target, filePath, channel)
    }

    const note = await readVaultNote(target, relativePath)
    const markdown = typeof note?.markdown === 'string'
      ? note.markdown
      : (typeof note?.content === 'string' ? note.content : null)
    if (markdown === null) return false

    console.info('[tauri:local-ipc] open-file via notes.read', {
      channel,
      relativePath,
      length: markdown.length
    })
    dispatchLocalIpcEvent(target, 'mt::open-new-tab', [
      {
        pathname: note?.fullPath || filePath,
        filename: getBasename(target, note?.fullPath || filePath),
        markdown,
        isMixedLineEndings: false
      },
      {},
      true
    ])
    return true
  } catch (error) {
    console.error('[tauri:local-ipc] open-file notes.read failed', {
      channel,
      filePath,
      error: error?.message || String(error)
    })
    return false
  }
}

export const installTauriLocalIpcBridge = (target = globalThis) => {
  if (!target?.__TAURI__ || target.__TAURI_LOCAL_IPC_BRIDGE_INSTALLED__) return false
  const ipc = target.tauri?.ipcRenderer
  if (!ipc?.send) return false

  const nativeSend = ipc.send.bind(ipc)
  ipc.send = (channel, ...args) => {
    if (NOTE_OPEN_EVENTS.has(channel)) {
      void openVaultNoteWithBackend(target, channel, args).then((handled) => {
        if (!handled) nativeSend(channel, ...args)
      })
      return undefined
    }

    if (LOCAL_IPC_EVENTS.has(channel)) {
      console.info('[tauri:local-ipc] dispatch', { channel })
      dispatchLocalIpcEvent(target, channel, args)
      return undefined
    }
    return nativeSend(channel, ...args)
  }

  target.__TAURI_LOCAL_IPC_BRIDGE_INSTALLED__ = true
  console.info('[tauri:local-ipc] bridge:installed')
  return true
}
