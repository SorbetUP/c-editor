import { getCurrentWindow } from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { confirm as confirmDialog, open as openDialog } from '@tauri-apps/plugin-dialog'
import { openUrl, openPath, revealItemInDir } from '@tauri-apps/plugin-opener'
import { writeText as clipboardWriteText, readText as clipboardReadText } from '@tauri-apps/plugin-clipboard-manager'

const normalizeSlashes = (value) => String(value || '').replace(/\\/g, '/')

const isAbsolute = (pathname) => {
  const value = normalizeSlashes(pathname)
  return (
    value.startsWith('/') ||
    /^([a-zA-Z]:\/)/.test(value) ||
    value.startsWith('//')
  )
}

const splitSegments = (pathname) => {
  const value = normalizeSlashes(pathname)
  const prefix = value.startsWith('//') ? '//' : value.match(/^([a-zA-Z]:\/)/)?.[1] || '/'
  const body = value.slice(prefix === '//' ? 2 : prefix.length).replace(/\/+/g, '/')
  const segments = body.split('/').filter((segment) => segment && segment !== '.')
  const resolved = []
  for (const segment of segments) {
    if (segment === '..') {
      if (resolved.length && resolved[resolved.length - 1] !== '..') {
        resolved.pop()
      } else if (!isAbsolute(prefix)) {
        resolved.push(segment)
      }
      continue
    }
    resolved.push(segment)
  }
  return { prefix, segments: resolved }
}

export const createPathFacade = () => {
  const join = (...parts) => normalize(joinedPath(parts))
  const normalize = (pathname) => {
    const value = normalizeSlashes(pathname)
    if (!value) return '.'
    const { prefix, segments } = splitSegments(value)
    const body = segments.join('/')
    if (!body) return prefix === '/' ? '/' : prefix || '.'
    return prefix === '/' || prefix === '//' || /^[a-zA-Z]:\/$/.test(prefix)
      ? `${prefix.replace(/\/$/, '')}/${body}`.replace(/\/+/g, '/')
      : body
  }
  const dirname = (pathname) => {
    const value = normalizeSlashes(pathname)
    if (!value) return '.'
    const normalized = normalize(value)
    if (normalized === '/' || /^[a-zA-Z]:\/$/.test(normalized)) return normalized
    const index = normalized.lastIndexOf('/')
    if (index <= 0) return isAbsolute(normalized) ? '/' : '.'
    return normalized.slice(0, index)
  }
  const basename = (pathname, ext = '') => {
    const value = normalizeSlashes(pathname)
    if (!value) return ''
    const normalized = value.replace(/\/+$/, '')
    const index = normalized.lastIndexOf('/')
    const base = index >= 0 ? normalized.slice(index + 1) : normalized
    return ext && base.endsWith(ext) ? base.slice(0, -ext.length) : base
  }
  const extname = (pathname) => {
    const base = basename(pathname)
    const index = base.lastIndexOf('.')
    return index > 0 ? base.slice(index) : ''
  }
  const resolve = (...parts) => normalize(joinedPath(parts, true))
  const relative = (from, to) => {
    const fromParts = splitSegments(resolve(from)).segments
    const toParts = splitSegments(resolve(to)).segments
    while (fromParts.length && toParts.length && fromParts[0] === toParts[0]) {
      fromParts.shift()
      toParts.shift()
    }
    return [...fromParts.map(() => '..'), ...toParts].join('/') || ''
  }
  const joinWithAbsolute = (parts, resolveMode = false) => {
    const filtered = parts.filter((part) => part !== undefined && part !== null && part !== '')
    if (!filtered.length) return resolveMode ? '/' : '.'
    const resolved = filtered.map((part) => normalizeSlashes(part))
    const merged = resolved.join('/')
    return normalize(merged)
  }
  function joinedPath(parts, resolveMode = false) {
    return joinWithAbsolute(parts, resolveMode)
  }
  return {
    sep: '/',
    delimiter: ':',
    normalize,
    join,
    resolve,
    dirname,
    basename,
    extname,
    isAbsolute,
    relative
  }
}

const createEventBus = (target = globalThis) => {
  const listeners = new Map()
  const add = (eventName, handler, once = false) => {
    const wrapped = (event) => {
      if (once) remove(eventName, handler)
      const detail = Array.isArray(event?.detail) ? event.detail : [event?.detail]
      handler?.({ error: event?.detail?.error || null }, ...detail)
    }
    const eventListeners = listeners.get(eventName) || new Map()
    eventListeners.set(handler, wrapped)
    listeners.set(eventName, eventListeners)
    target.addEventListener(eventName, wrapped)
    return () => remove(eventName, handler)
  }
  const remove = (eventName, handler) => {
    const eventListeners = listeners.get(eventName)
    const wrapped = eventListeners?.get(handler)
    if (!wrapped) return
    target.removeEventListener(eventName, wrapped)
    eventListeners.delete(handler)
    if (!eventListeners.size) listeners.delete(eventName)
  }
  const removeAll = (eventName) => {
    const eventListeners = listeners.get(eventName)
    if (!eventListeners) return
    for (const wrapped of eventListeners.values()) {
      target.removeEventListener(eventName, wrapped)
    }
    listeners.delete(eventName)
  }
  const send = (eventName, ...args) => {
    target.dispatchEvent(new CustomEvent(eventName, { detail: args }))
  }
  return {
    send,
    on: (eventName, handler) => add(eventName, handler, false),
    once: (eventName, handler) => add(eventName, handler, true),
    removeListener: remove,
    removeAllListeners: removeAll,
    invoke: async(channel, payload) => {
      const handler = listeners.get(`invoke:${channel}`)?.values()?.next()?.value
      if (handler) return handler({ detail: payload }, payload)
      if (channel === 'update-buffer-state') return true
      throw new Error(`No invoke handler registered for "${channel}"`)
    }
  }
}

const createShellFallback = () => ({
  openExternal: async(url) => {
    window.open(url, '_blank', 'noopener,noreferrer')
  },
  openPath: async(pathname) => {
    window.open(`file://${pathname}`, '_blank', 'noopener,noreferrer')
  },
  showItemInFolder: async(pathname) => {
    window.open(`file://${pathname}`, '_blank', 'noopener,noreferrer')
  }
})

const createClipboardFallback = () => ({
  writeText: async(text) => navigator.clipboard?.writeText?.(text),
  readText: async() => navigator.clipboard?.readText?.()
})

const createWebFrameFallback = () => ({
  setZoomFactor: (zoomFactor) => {
    if (typeof document !== 'undefined' && document.body) {
      document.body.style.zoom = `${zoomFactor * 100}%`
    }
  }
})

const createWebUtilsFallback = () => ({
  getPathForFile: (file) => file?.path || file?.webkitRelativePath || file?.name || ''
})

const createCommandExistsFallback = () => ({
  exists: () => false
})

const STORAGE_PREFIX = 'elephantnote:tauri:'
const KEYBINDINGS_STORAGE_KEY = `${STORAGE_PREFIX}user-keybindings`
const SPELLCHECKER_STORAGE_KEY = `${STORAGE_PREFIX}spellchecker`
const USER_PREFS_STORAGE_KEY = `${STORAGE_PREFIX}preferences`
const USER_DATA_STORAGE_KEY = `${STORAGE_PREFIX}user-data`
const PANDOC_EXTENSIONS = new Set(['html', 'docx', 'odt', 'latex', 'tex', 'ltx', 'rst', 'rest', 'org', 'wiki', 'dokuwiki', 'textile', 'opml', 'epub'])
const MARKDOWN_EXTENSIONS = new Set(['md', 'markdown', 'mdown', 'mkdn', 'mkd', 'mdwn', 'mdtxt', 'mdtext', 'mdx', 'text', 'txt'])

const readStoredJson = (target, key, fallback) => {
  try {
    const raw =
      target?.localStorage?.getItem(key) ??
      target?.__TAURI_BRIDGE_STORAGE__?.get?.(key) ??
      null
    if (raw == null) return fallback
    return JSON.parse(raw)
  } catch {
    return fallback
  }
}

const writeStoredJson = (target, key, value) => {
  try {
    const raw = JSON.stringify(value)
    if (!target.__TAURI_BRIDGE_STORAGE__) {
      target.__TAURI_BRIDGE_STORAGE__ = new Map()
    }
    target.__TAURI_BRIDGE_STORAGE__.set(key, raw)
    if (target?.localStorage) {
      try {
        target.localStorage.setItem(key, raw)
      } catch {
        // Keep the in-memory fallback when web storage is unavailable.
      }
    }
    return true
  } catch {
    return false
  }
}

const createKeybindingsMap = (entries) => new Map(Object.entries(entries))

const createMacosDefaultKeybindings = () => createKeybindingsMap({
  'file.save': 'CmdOrCtrl+S',
  'file.save-as': 'CmdOrCtrl+Shift+S',
  'edit.undo': 'CmdOrCtrl+Z',
  'edit.redo': 'CmdOrCtrl+Shift+Z',
  'edit.find': 'CmdOrCtrl+F',
  'file.quick-open': 'CmdOrCtrl+P',
  'view.toggle-sidebar': 'CmdOrCtrl+J',
  'app.preferences': 'CmdOrCtrl+,'
})

const createWindowsDefaultKeybindings = () => createKeybindingsMap({
  'file.save': 'CmdOrCtrl+S',
  'file.save-as': 'CmdOrCtrl+Shift+S',
  'edit.undo': 'CmdOrCtrl+Z',
  'edit.redo': 'CmdOrCtrl+Y',
  'edit.find': 'CmdOrCtrl+F',
  'file.quick-open': 'CmdOrCtrl+P',
  'view.toggle-sidebar': 'CmdOrCtrl+J',
  'app.preferences': 'CmdOrCtrl+,'
})

const createLinuxDefaultKeybindings = () => createWindowsDefaultKeybindings()

const getDefaultKeybindings = (target) => {
  const platform = String(target?.navigator?.platform || target?.navigator?.userAgent || '').toLowerCase()
  if (platform.includes('mac')) return createMacosDefaultKeybindings()
  if (platform.includes('linux')) return createLinuxDefaultKeybindings()
  return createWindowsDefaultKeybindings()
}

const getCurrentLanguage = (target) => {
  const stored = target?.localStorage?.getItem(`${STORAGE_PREFIX}language`)
  if (stored) return stored
  const browserLocale = String(
    target?.navigator?.language || target?.navigator?.languages?.[0] || 'en'
  )
  const normalized = browserLocale.toLowerCase()
  if (normalized.startsWith('zh-hk')) return 'zh-TW'
  if (normalized.startsWith('zh-tw')) return 'zh-TW'
  if (normalized.startsWith('zh')) return 'zh-CN'
  if (normalized.startsWith('en')) return 'en'
  if (normalized.startsWith('ja')) return 'ja'
  if (normalized.startsWith('ko')) return 'ko'
  if (normalized.startsWith('fr')) return 'fr'
  if (normalized.startsWith('de')) return 'de'
  if (normalized.startsWith('es')) return 'es'
  if (normalized.startsWith('pt')) return 'pt'
  if (normalized.startsWith('ru')) return 'ru'
  return 'en'
}

const getSpellcheckerState = (target) => {
  const state = readStoredJson(target, SPELLCHECKER_STORAGE_KEY, null)
  if (state && typeof state === 'object') return state
  return { enabled: false, language: 'en-US', words: [] }
}

const setSpellcheckerState = (target, nextState) => {
  const state = {
    enabled: !!nextState?.enabled,
    language: nextState?.language || 'en-US',
    words: Array.isArray(nextState?.words) ? nextState.words : []
  }
  writeStoredJson(target, SPELLCHECKER_STORAGE_KEY, state)
  return state
}

const getStoredPreferences = (target) => readStoredJson(target, USER_PREFS_STORAGE_KEY, {})
const setStoredPreferences = (target, nextPrefs) => {
  const current = getStoredPreferences(target)
  const merged = { ...current, ...(nextPrefs || {}) }
  writeStoredJson(target, USER_PREFS_STORAGE_KEY, merged)
  if (typeof merged.language === 'string' && merged.language) {
    try {
      target.dispatchEvent?.(new CustomEvent('language-changed', { detail: [merged.language] }))
    } catch {}
  }
  return merged
}
const getStoredUserData = (target) => readStoredJson(target, USER_DATA_STORAGE_KEY, {})
const setStoredUserData = (target, nextData) => {
  const current = getStoredUserData(target)
  const merged = { ...current, ...(nextData || {}) }
  writeStoredJson(target, USER_DATA_STORAGE_KEY, merged)
  return merged
}

const getFileExtension = (pathname) => {
  const value = String(pathname || '')
  const dot = value.lastIndexOf('.')
  if (dot < 0) return ''
  return value.slice(dot + 1).toLowerCase()
}

const createTauriFileUtilsFacade = (tauriFs) => {
  const metadataCache = new Map()
  const remember = (pathname, info) => {
    metadataCache.set(normalizeSlashes(pathname), info)
    return info
  }
  const getCached = (pathname) => metadataCache.get(normalizeSlashes(pathname))
  const probe = async(pathname) => {
    if (!tauriFs?.stat) return null
    try {
      const info = await tauriFs.stat(pathname)
      return remember(pathname, {
        isFile: !!info?.isFile,
        isDirectory: !!info?.isDirectory,
        info
      })
    } catch {
      return remember(pathname, { isFile: false, isDirectory: false })
    }
  }
  const readDir = async(pathname) => {
    if (!tauriFs?.readDir) return []
    try {
      return await tauriFs.readDir(pathname)
    } catch {
      return []
    }
  }
  return {
    isFile: (pathname) => !!getCached(pathname)?.isFile,
    isDirectory: (pathname) => !!getCached(pathname)?.isDirectory,
    emptyDir: async(pathname) => {
      if (!tauriFs?.readDir || !tauriFs?.remove) return
      const entries = await readDir(pathname)
      await Promise.all(entries.map(async(entry) => {
        const entryPath = entry?.path || entry?.name
        if (!entryPath) return
        await tauriFs.remove(entryPath, { recursive: !!entry?.isDirectory })
      }))
    },
    copy: async(src, dest) => {
      if (tauriFs?.copyFile) return tauriFs.copyFile(src, dest)
      const data = await tauriFs?.readFile?.(src)
      if (data && tauriFs?.writeFile) return tauriFs.writeFile(dest, data)
    },
    ensureDir: async(pathname) => {
      if (tauriFs?.mkdir) {
        try {
          return await tauriFs.mkdir(pathname, { recursive: true })
        } catch {
          return tauriFs.mkdir(pathname).catch(() => {})
        }
      }
    },
    outputFile: async(pathname, data) => {
      if (typeof data === 'string' && tauriFs?.writeTextFile) return tauriFs.writeTextFile(pathname, data)
      if (tauriFs?.writeFile) return tauriFs.writeFile(pathname, data)
    },
    move: async(src, dest) => {
      if (tauriFs?.rename) return tauriFs.rename(src, dest)
    },
    stat: async(pathname) => {
      const info = await probe(pathname)
      return info?.info || info
    },
    writeFile: async(pathname, data) => {
      if (typeof data === 'string' && tauriFs?.writeTextFile) return tauriFs.writeTextFile(pathname, data)
      if (tauriFs?.writeFile) return tauriFs.writeFile(pathname, data)
    },
    readFile: async(pathname) => {
      if (tauriFs?.readFile) return tauriFs.readFile(pathname)
      if (tauriFs?.readTextFile) return tauriFs.readTextFile(pathname)
    },
    ensureDirSync: () => {},
    pathExistsSync: (pathname) => metadataCache.has(normalizeSlashes(pathname)),
    isChildOfDirectory: (dir, child) => {
      const normalizedDir = normalizeSlashes(dir).replace(/\/+$/, '')
      const normalizedChild = normalizeSlashes(child)
      return normalizedChild === normalizedDir || normalizedChild.startsWith(`${normalizedDir}/`)
    },
    hasMarkdownExtension: (filename) => /\.md$/i.test(String(filename || '')),
    MARKDOWN_INCLUSIONS: ['.md', '.markdown', '.mdown', '.mkdn', '.mkd', '.mdtxt'],
    isSamePathSync: (pathA, pathB) => normalizeSlashes(pathA) === normalizeSlashes(pathB),
    isImageFile: (filepath) => /\.(png|jpe?g|gif|webp|svg|bmp|ico)$/i.test(String(filepath || ''))
  }
}

const createFileUtilsFallback = () => ({
  isFile: () => false,
  isDirectory: () => false,
  emptyDir: async() => {
    throw new Error('fileUtils.emptyDir is not available in the current runtime')
  },
  copy: async() => {
    throw new Error('fileUtils.copy is not available in the current runtime')
  },
  ensureDir: async() => {
    throw new Error('fileUtils.ensureDir is not available in the current runtime')
  },
  outputFile: async() => {
    throw new Error('fileUtils.outputFile is not available in the current runtime')
  },
  move: async() => {
    throw new Error('fileUtils.move is not available in the current runtime')
  },
  stat: async() => {
    throw new Error('fileUtils.stat is not available in the current runtime')
  },
  writeFile: async() => {
    throw new Error('fileUtils.writeFile is not available in the current runtime')
  },
  readFile: async() => {
    throw new Error('fileUtils.readFile is not available in the current runtime')
  },
  ensureDirSync: () => {
    throw new Error('fileUtils.ensureDirSync is not available in the current runtime')
  },
  pathExistsSync: () => false,
  isChildOfDirectory: () => false,
  hasMarkdownExtension: (filename) => /\.md$/i.test(String(filename || '')),
  MARKDOWN_INCLUSIONS: ['.md', '.markdown', '.mdown', '.mkdn', '.mkd', '.mdtxt'],
  isSamePathSync: (pathA, pathB) => normalizeSlashes(pathA) === normalizeSlashes(pathB),
  isImageFile: (filepath) => /\.(png|jpe?g|gif|webp|svg|bmp|ico)$/i.test(String(filepath || ''))
})

const createTauriElectronFacade = (target, tauri) => {
  const coreApi = tauri?.core
  const fsApi = tauri?.fs
  const eventBus = createEventBus(target)
  const imageExtensions = ['jpeg', 'jpg', 'png', 'gif', 'svg', 'webp']
  const currentWindow = () => getCurrentWindow()
  const openEditorWindow = () => {
    const label = `editor-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
    return new WebviewWindow(label, {
      url: target.location?.href || '/',
      title: target.document?.title || 'Elephant'
    })
  }
  const decodeText = async(data) => {
    if (typeof data === 'string') return data
    if (data instanceof Uint8Array) {
      return new TextDecoder().decode(data)
    }
    if (data instanceof ArrayBuffer) {
      return new TextDecoder().decode(new Uint8Array(data))
    }
    return String(data ?? '')
  }
  const openMarkdownDocuments = async(filePaths) => {
    const paths = Array.isArray(filePaths) ? filePaths : [filePaths]
    console.info('[tauri:runtime-bridge] openMarkdownDocuments:start', {
      count: paths.filter(Boolean).length,
      paths: paths.filter(Boolean)
    })
    for (const [index, pathname] of paths.filter(Boolean).entries()) {
      const markdown = await decodeText(await target.fileUtils?.readFile?.(pathname))
      const filename = target.path?.basename?.(pathname) || pathname.split('/').pop() || pathname
      console.info('[tauri:runtime-bridge] openMarkdownDocuments:dispatch', {
        index,
        pathname,
        filename,
        markdownLength: markdown.length,
        selected: index === 0
      })
      eventBus.send('mt::open-new-tab', {
        pathname,
        filename,
        markdown,
        isMixedLineEndings: false
      }, {}, index === 0)
    }
    console.info('[tauri:runtime-bridge] openMarkdownDocuments:done', { count: paths.filter(Boolean).length })
  }
  const importPandocDocument = async(pathname) => {
    if (!pathname) return false
    try {
      const result = await target.tauri.shell.exec('pandoc', ['-s', pathname, '-t', 'gfm'])
      if (!result?.success) return false
      const filename = target.path?.basename?.(pathname) || pathname.split('/').pop() || pathname
      eventBus.send('mt::open-new-tab', {
        pathname,
        filename,
        markdown: result.stdout || '',
        isMixedLineEndings: false
      }, {}, true)
      return true
    } catch {
      return false
    }
  }
  const openFolderPath = async(pathname) => {
    if (!pathname) return
    eventBus.send('mt::open-directory', pathname)
  }
  const searchImagePath = async(pathname, src) => {
    if (!fsApi?.readDir || !pathname || !src) return []
    const normalizedSrc = String(src).replace(/\\/g, '/')
    const fullPath = normalizedSrc.startsWith('/')
      ? normalizedSrc
      : `${String(pathname).replace(/\\/g, '/').replace(/\/+$/, '')}/${normalizedSrc}`
    const lastSlash = fullPath.lastIndexOf('/')
    const directory = lastSlash >= 0 ? fullPath.slice(0, lastSlash) : fullPath
    const searchKey = lastSlash >= 0 ? fullPath.slice(lastSlash + 1) : ''
    try {
      const entries = await fsApi.readDir(directory)
      const matcher = searchKey.toLowerCase()
      return entries
        .map((entry) => ({
          file: entry?.name || entry?.path?.split('/').pop() || '',
          path: entry?.path || '',
          type: entry?.isDirectory ? 'directory' : (imageExtensions.some((ext) => String(entry?.name || '').toLowerCase().endsWith(`.${ext}`)) ? 'image' : '')
        }))
        .filter((entry) => entry.file && (entry.type === 'directory' || entry.type === 'image'))
        .filter((entry) => !matcher || entry.file.toLowerCase().includes(matcher))
    } catch {
      return []
    }
  }
  const dispatchNativeCommand = async(channel, args) => {
    switch (channel) {
      case 'mt::cmd-close-window':
      case 'mt::app-try-quit':
        await currentWindow().close()
        return true
      case 'mt::cmd-new-editor-window':
        openEditorWindow()
        return true
      case 'mt::window-toggle-always-on-top': {
        const win = currentWindow()
        await win.setAlwaysOnTop(!(await win.isAlwaysOnTop()))
        return true
      }
      case 'mt::cmd-open-file': {
        const paths = await openDialog({
          multiple: true,
          directory: false,
          filters: [{ name: 'Markdown document', extensions: ['md', 'markdown', 'mdown', 'mkdn', 'mkd', 'mdtxt'] }]
        })
        if (Array.isArray(paths) && paths.length) {
          await openMarkdownDocuments(paths)
        } else if (typeof paths === 'string' && paths) {
          await openMarkdownDocuments([paths])
        }
        return true
      }
      case 'mt::cmd-open-folder': {
        const folder = await openDialog({
          multiple: false,
          directory: true,
          createDirectory: true
        })
        if (typeof folder === 'string' && folder) {
          await openFolderPath(folder)
        } else if (Array.isArray(folder) && folder[0]) {
          await openFolderPath(folder[0])
        }
        return true
      }
      case 'mt::ask-for-open-project-in-sidebar': {
        const folder = await openDialog({
          multiple: false,
          directory: true,
          createDirectory: true
        })
        if (typeof folder === 'string' && folder) {
          await openFolderPath(folder)
        } else if (Array.isArray(folder) && folder[0]) {
          await openFolderPath(folder[0])
        }
        return true
      }
      case 'mt::ask-for-user-preference': {
        eventBus.send('mt::user-preference', getStoredPreferences(target))
        return true
      }
      case 'mt::ask-for-user-data': {
        eventBus.send('mt::user-preference', getStoredUserData(target))
        return true
      }
      case 'mt::set-user-preference': {
        const [settings] = args
        const next = setStoredPreferences(target, settings)
        eventBus.send('mt::user-preference', next)
        return true
      }
      case 'mt::set-user-data': {
        const [settings] = args
        const next = setStoredUserData(target, settings)
        eventBus.send('mt::user-preference', next)
        return true
      }
      case 'mt::cmd-toggle-autosave': {
        const next = setStoredPreferences(target, {
          autoSave: !getStoredPreferences(target).autoSave
        })
        eventBus.send('mt::user-preference', next)
        return true
      }
      case 'mt::cmd-import-file': {
        const selected = await openDialog({
          multiple: false,
          directory: false,
          filters: [{ name: 'Pandoc input', extensions: Array.from(PANDOC_EXTENSIONS) }]
        })
        const pathname = Array.isArray(selected) ? selected[0] : selected
        if (!pathname) return true
        const imported = await importPandocDocument(pathname)
        if (!imported) {
          eventBus.send('mt::pandoc-not-exists', {
            title: 'Import warning',
            type: 'warning',
            message: 'Pandoc is required to import this file.'
          })
        }
        return true
      }
      case 'mt::ask-for-modify-image-folder-path': {
        const [imagePath] = args
        const selected = imagePath || await openDialog({
          multiple: false,
          directory: true,
          createDirectory: true
        })
        const value = Array.isArray(selected) ? selected[0] : selected
        if (value) {
          eventBus.send('mt::user-preference', setStoredUserData(target, { imageFolderPath: value }))
        }
        return true
      }
      case 'mt::get-current-language': {
        eventBus.send('mt::current-language', getCurrentLanguage(target))
        return true
      }
      case 'mt::request-keybindings': {
        eventBus.send('mt::keybindings-response', Object.fromEntries(getDefaultKeybindings(target)))
        return true
      }
      case 'mt::keybinding-debug-dump-keyboard-info': {
        console.info('[tauri] keyboard info', {
          platform: target?.navigator?.platform || target?.navigator?.userAgent || 'unknown'
        })
        return true
      }
      case 'mt::open-file':
      case 'mt::open-file-by-window-id': {
        const filePath = args[0]
        if (filePath) {
          await openMarkdownDocuments([filePath])
          return true
        }
        return false
      }
      case 'mt::open-setting-window':
        eventBus.send('settings::change-tab', args[0] || null)
        if (!target.__tauriSettingWindowOpened) {
          target.__tauriSettingWindowOpened = true
          new WebviewWindow('settings', {
            url: `${target.location?.origin || ''}${target.location?.pathname || ''}#/settings`,
            title: 'Settings'
          })
        }
        return true
      case 'mt::select-default-directory-to-open': {
        const folder = await openDialog({
          multiple: false,
          directory: true,
          createDirectory: true
        })
        if (typeof folder === 'string' && folder) {
          target.localStorage?.setItem(`${STORAGE_PREFIX}defaultDirectoryToOpen`, folder)
          eventBus.send('mt::user-preference', { defaultDirectoryToOpen: folder })
        }
        return true
      }
      case 'mt::ask-for-image-auto-path': {
        const [payload] = args
        const files = await searchImagePath(payload?.pathname, payload?.src)
        eventBus.send(`mt::response-of-image-path-${payload?.id}`, files)
        return true
      }
      case 'mt::make-screenshot':
        eventBus.send('mt::show-notification', {
          title: 'Screenshot',
          type: 'warning',
          message: 'Screenshot capture is not available in the Tauri bridge.'
        })
        return true
      case 'mt::check-for-update':
        eventBus.send('mt::UPDATE_NOT_AVAILABLE', 'Update checks are not configured in the Tauri bridge.')
        return true
      case 'mt::window::drop': {
        const [fileList = []] = args
        for (const file of fileList) {
          const ext = getFileExtension(file)
          if (MARKDOWN_EXTENSIONS.has(ext)) {
            await openMarkdownDocuments([file])
            continue
          }
          if (PANDOC_EXTENSIONS.has(ext)) {
            const imported = await importPandocDocument(file)
            if (!imported) {
              eventBus.send('mt::pandoc-not-exists', {
                title: 'Import warning',
                type: 'warning',
                message: 'Pandoc is required to import this file.'
              })
            }
          }
        }
        return true
      }
      default:
        return false
    }
  }

  const invoke = async(channel, payload) => {
    switch (channel) {
      case 'mt::get-current-language':
        return getCurrentLanguage(target)
      case 'mt::keybinding-get-keyboard-info':
        return {
          layout: target?.navigator?.language || 'en-US',
          keymap: {}
        }
      case 'mt::keybinding-get-pref-keybindings': {
        const defaultKeybindings = getDefaultKeybindings(target)
        const savedUserKeybindings = readStoredJson(target, KEYBINDINGS_STORAGE_KEY, {})
        const userKeybindings = new Map()
        for (const [key, value] of Object.entries(savedUserKeybindings || {})) {
          userKeybindings.set(key, value)
        }
        return {
          defaultKeybindings,
          userKeybindings
        }
      }
      case 'mt::keybinding-save-user-keybindings': {
        const plain = payload instanceof Map
          ? Object.fromEntries(payload)
          : (payload && typeof payload === 'object' ? { ...payload } : {})
        const ok = writeStoredJson(target, KEYBINDINGS_STORAGE_KEY, plain)
        return ok
      }
      case 'mt::spellchecker-set-enabled': {
        const state = getSpellcheckerState(target)
        setSpellcheckerState(target, { ...state, enabled: !!payload })
        return true
      }
      case 'mt::spellchecker-switch-language': {
        const state = getSpellcheckerState(target)
        setSpellcheckerState(target, { ...state, enabled: true, language: String(payload || '') })
        return null
      }
      case 'mt::spellchecker-get-available-dictionaries':
        return ['en-US']
      case 'mt::spellchecker-get-custom-dictionary-words':
        return getSpellcheckerState(target).words
      case 'mt::spellchecker-remove-word': {
        const state = getSpellcheckerState(target)
        const next = state.words.filter((word) => word !== payload)
        setSpellcheckerState(target, { ...state, words: next })
        return true
      }
      case 'update-buffer-state':
        writeStoredJson(target, `${STORAGE_PREFIX}buffer-state`, payload)
        return true
      case 'mt::ask-for-image-path':
        return openDialog({
          multiple: false,
          directory: false,
          filters: [{ name: 'Images', extensions: ['jpeg', 'jpg', 'png', 'gif', 'svg', 'webp'] }]
        }).then((value) => (Array.isArray(value) ? value[0] || '' : value || ''))
      case 'mt::ask-for-excalidraw-source-path': {
        const useBlank = await confirmDialog('Start a blank drawing?', {
          title: 'Open Excalidraw'
        })
        if (useBlank) {
          return { canceled: false, sourcePath: '', mode: 'blank' }
        }
        const filePath = await openDialog({
          multiple: false,
          directory: false,
          filters: [
            { name: 'Images and Excalidraw', extensions: ['jpeg', 'jpg', 'png', 'gif', 'svg', 'webp', 'excalidraw'] },
            { name: 'Excalidraw', extensions: ['excalidraw'] },
            { name: 'Images', extensions: ['jpeg', 'jpg', 'png', 'gif', 'svg', 'webp'] }
          ]
        })
        if (Array.isArray(filePath) ? !filePath[0] : !filePath) {
          return { canceled: true, sourcePath: '', mode: 'cancel' }
        }
        return { canceled: false, sourcePath: Array.isArray(filePath) ? filePath[0] : filePath, mode: 'file' }
      }
      case 'mt::fs-trash-item': {
        if (fsApi?.remove) {
          await fsApi.remove(payload, { recursive: true })
          return true
        }
        return false
      }
    }
    if (!coreApi?.invoke) {
      throw new Error('Tauri core API is unavailable')
    }
    return coreApi.invoke(channel, payload)
  }

  return {
    ipcRenderer: {
      send: (channel, ...args) => {
        void dispatchNativeCommand(channel, args).then((handled) => {
          if (!handled) {
            eventBus.send(channel, ...args)
          }
        })
      },
      on: eventBus.on,
      once: eventBus.once,
      removeListener: eventBus.removeListener,
      removeAllListeners: eventBus.removeAllListeners,
      invoke
    },
    shell: {
      openExternal: async(url) => openUrl(url),
      openPath: async(pathname) => openPath(pathname),
      showItemInFolder: async(pathname) => revealItemInDir(pathname),
      exec: async(command, args = [], options = {}) => {
        if (!coreApi?.invoke) {
          throw new Error('Tauri core API is unavailable')
        }
        return coreApi.invoke('shell_exec', {
          command,
          args,
          cwd: options.cwd || null,
          env: options.env || null
        })
      }
    },
    clipboard: {
      writeText: async(text) => clipboardWriteText(text),
      readText: async() => clipboardReadText()
    },
    webFrame: createWebFrameFallback(),
    webUtils: {
      getPathForFile: (file) => file?.path || file?.webkitRelativePath || file?.name || ''
    },
    process: {
      platform: target.navigator?.userAgentData?.platform || target.navigator?.platform || 'tauri',
      env: { MARKTEXT_VERSION_STRING: target.__MARKTEXT_VERSION_STRING__ || '' }
    },
    fs: fsApi
  }
}

export const installRuntimeBridge = (target = globalThis) => {
  const hasTauri = !!target?.__TAURI__
  const markRuntime = (mode) => {
    target.__MARKTEXT_RUNTIME__ = mode
    return mode
  }
  if (!target.path) target.path = createPathFacade()
  if (!target.commandExists) target.commandExists = createCommandExistsFallback()
  if (!target.i18nUtils) target.i18nUtils = {}
  if (!target.elephantnote) target.elephantnote = {}

  if (hasTauri) {
    const tauri = createTauriElectronFacade(target, target.__TAURI__)
    if (!target.fileUtils) {
      target.fileUtils = createTauriFileUtilsFacade(target.__TAURI__?.fs)
    }
    target.tauri = tauri
    return { mode: markRuntime('tauri'), installed: true }
  }

  if (!target.fileUtils) target.fileUtils = createFileUtilsFallback()
  const ipcRenderer = createEventBus(target)
  const shell = createShellFallback()
  const clipboard = createClipboardFallback()
  const webFrame = createWebFrameFallback()
  const webUtils = createWebUtilsFallback()

  const fallback = {
    ipcRenderer,
    shell,
    clipboard,
    webFrame,
    webUtils,
    process: {
      platform: target.navigator?.userAgentData?.platform || target.navigator?.platform || 'tauri',
      env: { MARKTEXT_VERSION_STRING: target.__MARKTEXT_VERSION_STRING__ || '' }
    }
  }
  target.tauri = fallback

  return { mode: markRuntime('tauri-compatible'), installed: true }
}

export const createCompatibilitySurface = installRuntimeBridge
