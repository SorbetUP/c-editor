'use strict'

const fs = require('fs')
const path = require('path')
const os = require('os')
const { clipboard, contextBridge, shell } = require('electron')

const configRoot = process.env.ELEPHANTNOTE_CONFIG_DIR || path.join(os.tmpdir(), 'elephantnote-e2e-config')
const configFile = path.join(configRoot, 'elephantnote.json')
const memory = {
  prefs: {},
  userData: {},
  buffers: {},
  secrets: {},
  enabledAddons: new Map()
}

const readJson = (filename, fallback) => {
  try {
    return JSON.parse(fs.readFileSync(filename, 'utf8'))
  } catch {
    return fallback
  }
}

const normalizeSlashes = (value = '') => String(value || '').replaceAll('\\', '/')
const safeRelativePath = (value = '') => {
  const normalized = normalizeSlashes(value).replace(/^\/+/, '')
  const parts = normalized.split('/').filter((part) => part && part !== '.')
  if (parts.some((part) => part === '..')) throw new Error(`Path traversal is not allowed: ${value}`)
  return parts.join('/')
}

const readConfig = () => readJson(configFile, { vaults: [], activeVaultId: null })
const activeVault = () => {
  const config = readConfig()
  return config.vaults?.find((vault) => vault.id === config.activeVaultId) || config.vaults?.[0] || null
}

const vaultRoot = () => activeVault()?.path || process.env.ELEPHANT_E2E_VAULT_ROOT || ''
const resolveVaultPath = (relativePath = '') => {
  const root = vaultRoot()
  if (!root) throw new Error('No active E2E vault')
  const relative = safeRelativePath(relativePath)
  const resolved = path.resolve(root, relative)
  const canonicalRoot = path.resolve(root)
  if (resolved !== canonicalRoot && !resolved.startsWith(`${canonicalRoot}${path.sep}`)) {
    throw new Error(`Path escaped active vault: ${relativePath}`)
  }
  return resolved
}

const markdownPreview = (content = '') => String(content)
  .replace(/^---[\s\S]*?---\s*/m, '')
  .replace(/^#{1,6}\s+/gm, '')
  .replace(/\s+/g, ' ')
  .trim()
  .slice(0, 400)

const titleFromMarkdown = (content, filename) => {
  const heading = String(content || '').match(/^#\s+(.+)$/m)?.[1]?.trim()
  return heading || String(filename || 'Untitled').replace(/\.md$/i, '')
}

const entryForPath = (root, fullPath) => {
  const stat = fs.statSync(fullPath)
  const relativePath = normalizeSlashes(path.relative(root, fullPath))
  const filename = path.basename(fullPath)
  if (stat.isDirectory()) {
    return {
      id: relativePath || '.',
      path: relativePath,
      relativePath,
      fullPath,
      filename,
      title: filename,
      kind: 'folder',
      type: 'folder',
      updatedAt: stat.mtime.toISOString(),
      children: []
    }
  }
  const content = fs.readFileSync(fullPath, 'utf8')
  return {
    id: relativePath,
    path: relativePath,
    relativePath,
    fullPath,
    filename,
    title: titleFromMarkdown(content, filename),
    kind: 'note',
    type: 'note',
    preview: markdownPreview(content),
    excerpt: markdownPreview(content),
    tags: [],
    updatedAt: stat.mtime.toISOString(),
    size: stat.size
  }
}

const listDirectory = (relativePath = '') => {
  const root = vaultRoot()
  if (!root || !fs.existsSync(root)) return []
  const directory = resolveVaultPath(relativePath)
  if (!fs.existsSync(directory) || !fs.statSync(directory).isDirectory()) return []
  return fs.readdirSync(directory, { withFileTypes: true })
    .filter((entry) => !entry.name.startsWith('.'))
    .filter((entry) => entry.isDirectory() || /\.md$/i.test(entry.name))
    .map((entry) => entryForPath(root, path.join(directory, entry.name)))
    .sort((left, right) => {
      if (left.kind !== right.kind) return left.kind === 'folder' ? -1 : 1
      return left.title.localeCompare(right.title)
    })
}

const allMarkdownEntries = () => {
  const root = vaultRoot()
  if (!root || !fs.existsSync(root)) return []
  const results = []
  const visit = (directory) => {
    for (const item of fs.readdirSync(directory, { withFileTypes: true })) {
      if (item.name.startsWith('.')) continue
      const fullPath = path.join(directory, item.name)
      if (item.isDirectory()) visit(fullPath)
      else if (item.isFile() && /\.md$/i.test(item.name)) results.push(entryForPath(root, fullPath))
    }
  }
  visit(root)
  return results
}

const workspaceForVault = (vault) => {
  if (!vault) return null
  return readJson(path.join(vault.path, '.elephantnote', 'workspace.json'), {
    version: 1,
    vaultName: vault.name || path.basename(vault.path),
    sidebar: []
  })
}

const vaultPayload = () => {
  const config = readConfig()
  const vault = activeVault()
  return {
    vaults: config.vaults || [],
    activeVaultId: vault?.id || null,
    activeVault: vault,
    workspace: workspaceForVault(vault),
    entries: vault ? listDirectory('') : []
  }
}

const searchDocuments = () => allMarkdownEntries().map((entry) => {
  const content = fs.readFileSync(entry.fullPath, 'utf8')
  return {
    ...entry,
    content,
    body: content,
    relativePath: entry.path
  }
})

const searchQuery = (params = {}) => {
  const query = String(params.query || params.q || '').trim().toLowerCase()
  const limit = Math.max(1, Math.min(200, Number(params.limit || params.maxResults || 20)))
  if (!query) return []
  return searchDocuments()
    .map((document) => {
      const haystack = `${document.title}\n${document.path}\n${document.content}`.toLowerCase()
      return { ...document, score: haystack.includes(query) ? 1 : 0 }
    })
    .filter((document) => document.score > 0)
    .slice(0, limit)
}

const readMarkdown = (pathname) => {
  const fullPath = path.isAbsolute(pathname || '') ? pathname : resolveVaultPath(pathname)
  if (!fs.existsSync(fullPath) && normalizeSlashes(pathname) === '.elephantnote/Dashboard.md') {
    writeMarkdown(pathname, '# Dashboard\n')
  }
  return fs.readFileSync(fullPath, 'utf8')
}

const writeMarkdown = (pathname, content = '') => {
  const fullPath = path.isAbsolute(pathname || '') ? pathname : resolveVaultPath(pathname)
  fs.mkdirSync(path.dirname(fullPath), { recursive: true })
  fs.writeFileSync(fullPath, String(content), 'utf8')
  return { ok: true, path: normalizeSlashes(pathname), bytes: Buffer.byteLength(String(content)) }
}

const primitivePreference = (store, key, fallback = null) => Object.prototype.hasOwnProperty.call(store, key) ? store[key] : fallback

const invoke = async (command, payload = {}) => {
  const params = payload || {}
  switch (command) {
    case 'healthcheck': return 'ok'
    case 'tauri_acceptance_enabled': return false
    // Renderer diagnostics are best-effort in the Electron/Tauri compatibility
    // harness. Rejecting them turns harmless startup logging into unhandled
    // promise errors and can prevent addon scenarios from reaching the test.
    case 'tauri_debug_log': return { ok: true }
    // These plugin calls are non-functional in the Electron compatibility
    // harness, but the renderer issues them during desktop startup.
    case 'plugin:event|listen': return 1
    case 'plugin:window-state|restore_state': return true
    case 'plugin:window-state|save_window_state': return true
    case 'tauri_platform_info': return { os: process.platform, family: process.platform === 'win32' ? 'windows' : 'unix', arch: process.arch, mobile: false, desktop: true }
    case 'tauri_vaults_get': return vaultPayload()
    case 'tauri_directory_list': return listDirectory(params.relativePath || params.relative_path || '')
    case 'tauri_notes_create': {
      const directory = params.relativePath || params.relative_path || ''
      const filename = params.filename || 'Untitled.md'
      const relativePath = normalizeSlashes(path.join(directory, filename))
      const fullPath = resolveVaultPath(relativePath)
      if (!fs.existsSync(fullPath)) writeMarkdown(relativePath, '')
      return { path: relativePath, fullPath, title: params.title || path.basename(filename, '.md') }
    }
    case 'tauri_calendar_list': return []
    case 'tauri_sources_list': return []
    case 'tauri_wiki_list': return []
    case 'tauri_wiki_proposals': return []
    case 'tauri_addons_list':
    case 'tauri_addons_list_full':
    case 'tauri_addons_catalog_list':
    case 'tauri_official_addons_catalog_list': return []
    case 'tauri_atomic_features_list': return []
    case 'tauri_atomic_features_get': return null
    case 'tauri_features_get': return {}
    case 'tauri_ai_config_get': return { provider: 'none', search: { enabled: true }, indexing: { autoRebuild: false } }
    case 'tauri_models_list':
    case 'tauri_models_list_local': return []
    case 'tauri_models_active': return null
    case 'tauri_ollama_status': return { available: false, running: false }
    case 'tauri_ollama_list': return { models: [] }
    case 'tauri_sync_status':
    case 'iroh_sync_status': return { status: 'idle', running: false, connected: false, transport: 'iroh' }
    case 'tauri_search_status': return { status: 'ready', enabled: true, vaultPath: vaultRoot(), indexedDocuments: searchDocuments().length, totalDocuments: searchDocuments().length }
    case 'tauri_search_query': return searchQuery(params.params || params)
    case 'tauri_search_inspect': {
      const documents = searchDocuments()
      return { indexPath: '', documents, folders: [], semanticLinks: [], graph: { nodes: [], edges: [], clusters: [] }, generatedAt: new Date().toISOString() }
    }
    case 'tauri_search_rebuild': return { status: 'ready', enabled: true, documents: searchDocuments().length, notesIndexed: searchDocuments().length }
    case 'tauri_search_clear': return { status: 'ready', enabled: true, documents: 0, notesIndexed: 0 }
    case 'tauri_search_enable': return { status: 'ready', enabled: true }
    case 'tauri_search_disable': return { status: 'disabled', enabled: false }
    case 'tauri_fs_read_markdown': {
      const markdown = readMarkdown(params.path)
      return { path: params.path, markdown, content: markdown, encoding: 'utf-8' }
    }
    case 'tauri_notes_read': {
      const content = readMarkdown(params.path || params.relativePath)
      return { path: params.path || params.relativePath, markdown: content, content }
    }
    case 'tauri_fs_write_markdown':
    case 'tauri_notes_write':
    case 'tauri_marktext_write_file': return writeMarkdown(params.path || params.relativePath, params.markdown ?? params.content ?? params.data ?? '')
    case 'tauri_fs_resolve_path': return path.isAbsolute(params.path || '') ? params.path : resolveVaultPath(params.path || '')
    case 'tauri_fs_detect_encoding': return { encoding: 'utf-8', confidence: 1 }
    case 'tauri_prefs_get': return primitivePreference(memory.prefs, params.key)
    case 'tauri_prefs_all': return { ...memory.prefs }
    case 'tauri_prefs_set': memory.prefs[params.key] = params.value; return params.value
    case 'tauri_prefs_set_many': Object.assign(memory.prefs, params.values || params.prefs || {}); return { ...memory.prefs }
    case 'tauri_user_data_get': return primitivePreference(memory.userData, params.key)
    case 'tauri_user_data_all': return { ...memory.userData }
    case 'tauri_user_data_set': memory.userData[params.key] = params.value; return params.value
    case 'tauri_user_data_set_many': Object.assign(memory.userData, params.values || params.data || {}); return { ...memory.userData }
    case 'tauri_buffer_save': memory.buffers[params.key || params.id || 'default'] = params.value ?? params.data; return true
    case 'tauri_buffer_load': return memory.buffers[params.key || params.id || 'default'] ?? null
    case 'tauri_buffer_clear': delete memory.buffers[params.key || params.id || 'default']; return true
    case 'tauri_secret_set': memory.secrets[params.key] = params.value; return true
    case 'tauri_secret_get': return memory.secrets[params.key] ?? null
    case 'tauri_secret_delete': delete memory.secrets[params.key]; return true
    case 'tauri_keybindings_get': return {}
    case 'tauri_keybindings_save': return true
    case 'tauri_recents_list': return []
    case 'tauri_recents_add':
    case 'tauri_recents_clear': return true
    case 'tauri_markdown_parse':
    case 'tauri_markdown_to_text': return { markdown: params.markdown || '', text: String(params.markdown || '').replace(/[#*_`>-]/g, '') }
    case 'tauri_markdown_render_html': return { html: `<p>${String(params.markdown || '').replace(/[&<>]/g, (character) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' })[character])}</p>` }
    case 'tauri_markdown_extract_frontmatter': return { fields: {}, body: params.markdown || '' }
    case 'tauri_markdown_extract_links': return []
    case 'tauri_muya_parse': return { markdown: params.markdown || '', blocks: [] }
    case 'tauri_muya_render_html': return { html: `<p>${String(params.markdown || '')}</p>` }
    case 'tauri_muya_tokens': return []
    case 'tauri_muya_extras': return {}
    case 'tauri_addons_set_enabled':
    case 'tauri_addons_set_enabled_checked': memory.enabledAddons.set(params.addonId, params.enabled === true); return { ok: true }
    case 'tauri_addons_read_entry': throw new Error(`No physical addon is installed in the E2E fixture: ${params.addonId || ''}`)
    default:
      if (command.startsWith('plugin:path|') || command.includes('app_data_dir')) return configRoot
      if (command.startsWith('plugin:window|')) return null
      if (command.startsWith('plugin:dialog|')) return null
      if (command.startsWith('plugin:opener|')) return null
      if (command.startsWith('plugin:clipboard-manager|')) return null
      if (command.startsWith('plugin:fs|')) return null
      console.warn('[e2e-tauri] unhandled invoke', command, params)
      return null
  }
}

let callbackId = 0
const transformCallback = (callback, once = false) => {
  callbackId += 1
  const id = callbackId
  const property = `_${id}`
  Object.defineProperty(globalThis, property, {
    configurable: true,
    value: (value) => {
      if (once) delete globalThis[property]
      callback?.(value)
    }
  })
  return id
}

const tauri = {
  core: {
    invoke,
    convertFileSrc: (pathname) => `file://${normalizeSlashes(pathname)}`
  },
  path: {
    appDataDir: async () => configRoot,
    appConfigDir: async () => configRoot,
    join: async (...parts) => path.join(...parts),
    dirname: async (pathname) => path.dirname(pathname),
    basename: async (pathname, extension) => path.basename(pathname, extension)
  },
  fs: {
    readTextFile: async (pathname) => fs.readFileSync(pathname, 'utf8'),
    writeTextFile: async (pathname, contents) => writeMarkdown(pathname, contents),
    readFile: async (pathname) => new Uint8Array(fs.readFileSync(pathname)),
    writeFile: async (pathname, contents) => { fs.mkdirSync(path.dirname(pathname), { recursive: true }); fs.writeFileSync(pathname, Buffer.from(contents)); return true },
    readDir: async (pathname) => fs.readdirSync(pathname, { withFileTypes: true }).map((entry) => ({ name: entry.name, isFile: entry.isFile(), isDirectory: entry.isDirectory() })),
    stat: async (pathname) => { const stat = fs.statSync(pathname); return { isFile: stat.isFile(), isDirectory: stat.isDirectory(), size: stat.size } },
    mkdir: async (pathname, options) => fs.mkdirSync(pathname, { recursive: options?.recursive !== false }),
    remove: async (pathname, options) => fs.rmSync(pathname, { recursive: options?.recursive === true, force: true }),
    rename: async (from, to) => fs.renameSync(from, to),
    copyFile: async (from, to) => fs.copyFileSync(from, to)
  },
  dialog: {
    open: async () => null,
    save: async () => null,
    confirm: async () => true
  },
  opener: {
    openUrl: async (url) => shell.openExternal(url),
    openPath: async (pathname) => shell.openPath(pathname),
    revealItemInDir: async (pathname) => shell.showItemInFolder(pathname)
  },
  clipboardManager: {
    writeText: async (text) => clipboard.writeText(String(text)),
    readText: async () => clipboard.readText()
  }
}

contextBridge.exposeInMainWorld('__MARKTEXT_RUNTIME__', 'tauri')
contextBridge.exposeInMainWorld('__TAURI__', tauri)
contextBridge.exposeInMainWorld('__TAURI_INTERNALS__', {
  invoke,
  transformCallback,
  convertFileSrc: tauri.core.convertFileSrc,
  metadata: {
    currentWindow: { label: 'main' },
    currentWebview: { label: 'main' }
  }
})
contextBridge.exposeInMainWorld('__ELEPHANT_E2E__', {
  configRoot,
  vaultRoot: vaultRoot(),
  arguments: process.argv.filter((argument) => argument.startsWith('--elephant-e2e-arg='))
})
