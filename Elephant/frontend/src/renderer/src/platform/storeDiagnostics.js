import bus from '@/bus'
import { useEditorStore } from '@/store/editor'
import { useLayoutStore } from '@/store/layout'
import { useProjectStore } from '@/store/project'
import { useVaultStore } from 'elephant-front/stores/vaultStore'
import { isDiagnosticVerbose, pushDiagnosticLog } from './rendererDiagnostics'

const EXCALIDRAW_ASSET_RE = /!\[[^\]]*\]\([^)]*(?:^|\/)\.assets\/excalidraw-[^)]+\.png[^)]*\)/i
const ROOT_ASSET_REFERENCE_RE = /(!\[[^\]]*\]\()(\.assets\/[^)\s]+)([^)]*)(\))/g
const PROGRAMMATIC_MARKDOWN_PROTECTION_MS = 5_000
const tabMarkdownSnapshots = new Map()
const protectedProgrammaticMarkdown = new Map()

const summarizeProject = (store) => ({
  root: store.projectTree?.pathname || null,
  rootName: store.projectTree?.name || null,
  pendingTreeEvents: store.pendingTreeEvents?.length || 0,
  activeItem: store.activeItem?.pathname || null
})

const summarizeEditor = (store) => ({
  currentFileId: store.currentFile?.id || null,
  currentFilePath: store.currentFile?.pathname || null,
  currentMarkdownLength: store.currentFile?.markdown?.length || 0,
  tabCount: store.tabs?.length || 0,
  tabIds: store.tabs?.map((tab) => tab.id).slice(0, 8) || []
})

const summarizeLayout = (store) => ({
  showSideBar: store.showSideBar,
  showTabBar: store.showTabBar,
  leftColumn: store.leftColumn,
  rightColumn: store.rightColumn
})

const normalizeSlashPath = (value = '') => String(value || '').replace(/\\/g, '/')
const trimForStaleComparison = (value = '') => String(value || '').trim()
const encodeMarkdownAssetPath = (value = '') => normalizeSlashPath(value)
  .split('/')
  .map((segment) => encodeURIComponent(segment).replace(/[!'()*]/g, (char) => `%${char.charCodeAt(0).toString(16).toUpperCase()}`))
  .join('/')

const shouldIgnoreProgrammaticStaleEcho = (change = {}) => {
  const protection = protectedProgrammaticMarkdown.get(change.id)
  if (!protection) return false
  if (Date.now() > protection.until) {
    protectedProgrammaticMarkdown.delete(change.id)
    return false
  }
  if (typeof change.markdown !== 'string') return false
  const incoming = trimForStaleComparison(change.markdown)
  const protectedMarkdown = String(protection.markdown || '')
  if (!incoming || incoming.length >= protectedMarkdown.length) return false
  return protectedMarkdown.includes(incoming) && EXCALIDRAW_ASSET_RE.test(protectedMarkdown) && !EXCALIDRAW_ASSET_RE.test(incoming)
}

const installEditorContentGuard = (editorStore) => {
  if (editorStore.__ELEPHANT_PROGRAMMATIC_MARKDOWN_GUARD__) return
  const original = editorStore.LISTEN_FOR_CONTENT_CHANGE?.bind(editorStore)
  if (typeof original !== 'function') return
  editorStore.__ELEPHANT_PROGRAMMATIC_MARKDOWN_GUARD__ = true
  editorStore.LISTEN_FOR_CONTENT_CHANGE = (change = {}) => {
    if (shouldIgnoreProgrammaticStaleEcho(change)) {
      const protection = protectedProgrammaticMarkdown.get(change.id)
      pushDiagnosticLog('warn', 'editor-state:ignored-stale-programmatic-markdown-echo', {
        id: change.id,
        incomingLength: typeof change.markdown === 'string' ? change.markdown.length : 0,
        protectedLength: protection?.markdown?.length || 0,
        reason: 'excalidraw-image-was-just-inserted'
      })
      return
    }
    return original(change)
  }
}

const getVaultRootPath = (vaultStore, projectStore) => {
  return vaultStore?.activeVault?.path ||
    vaultStore?.vaults?.find?.((vault) => vault.id === vaultStore.activeVaultId)?.path ||
    projectStore?.projectTree?.pathname ||
    ''
}

const noteRelativeRootAssetPath = (assetSource, tab, vaultStore, projectStore) => {
  const vaultRoot = getVaultRootPath(vaultStore, projectStore)
  const notePath = tab?.pathname || ''
  const pathApi = globalThis.window?.path
  if (!assetSource || !vaultRoot || !notePath || !pathApi?.join || !pathApi?.dirname || !pathApi?.relative) {
    return assetSource
  }
  const noteDirectory = pathApi.dirname(notePath)
  const assetPath = pathApi.join(vaultRoot, assetSource)
  const relativePath = normalizeSlashPath(pathApi.relative(noteDirectory, assetPath))
  if (!relativePath || pathApi.isAbsolute?.(relativePath)) return assetSource
  return encodeMarkdownAssetPath(relativePath)
}

const rewriteRootAssetMarkdownReferencesIfNeeded = (tab, vaultStore, projectStore) => {
  if (!tab?.id || typeof tab.markdown !== 'string' || !tab.markdown.includes('](.assets/')) return false
  let rewrittenCount = 0
  const nextMarkdown = tab.markdown.replace(ROOT_ASSET_REFERENCE_RE, (full, prefix, source, suffix, close) => {
    const replacement = noteRelativeRootAssetPath(source, tab, vaultStore, projectStore)
    if (replacement !== source) rewrittenCount += 1
    return `${prefix}${replacement}${suffix}${close}`
  })
  if (!rewrittenCount || nextMarkdown === tab.markdown) return false
  tab.markdown = nextMarkdown
  tab.isSaved = false
  protectedProgrammaticMarkdown.set(tab.id, {
    markdown: nextMarkdown,
    until: Date.now() + PROGRAMMATIC_MARKDOWN_PROTECTION_MS
  })
  bus.emit('file-changed', {
    id: tab.id,
    markdown: nextMarkdown,
    cursor: tab.cursor || null,
    muyaIndexCursor: tab.muyaIndexCursor || null,
    renderCursor: false,
    history: tab.history,
    blocks: tab.blocks
  })
  pushDiagnosticLog('info', 'editor-state:rewrote-root-vault-assets-relative-to-note', {
    id: tab.id,
    pathname: tab.pathname || null,
    vaultRoot: getVaultRootPath(vaultStore, projectStore) || null,
    rewrittenCount,
    markdownLength: nextMarkdown.length
  })
  return true
}

const protectProgrammaticMarkdownIfNeeded = (tab, vaultStore, projectStore) => {
  if (!tab?.id || typeof tab.markdown !== 'string') return
  rewriteRootAssetMarkdownReferencesIfNeeded(tab, vaultStore, projectStore)
  const previousMarkdown = tabMarkdownSnapshots.get(tab.id) || ''
  const nextMarkdown = tab.markdown
  tabMarkdownSnapshots.set(tab.id, nextMarkdown)
  const insertedExcalidrawAsset = EXCALIDRAW_ASSET_RE.test(nextMarkdown) &&
    nextMarkdown.length > previousMarkdown.length &&
    !EXCALIDRAW_ASSET_RE.test(previousMarkdown)
  if (!insertedExcalidrawAsset) return

  protectedProgrammaticMarkdown.set(tab.id, {
    markdown: nextMarkdown,
    until: Date.now() + PROGRAMMATIC_MARKDOWN_PROTECTION_MS
  })
  bus.emit('file-changed', {
    id: tab.id,
    markdown: nextMarkdown,
    cursor: tab.cursor || null,
    muyaIndexCursor: tab.muyaIndexCursor || null,
    renderCursor: false,
    history: tab.history,
    blocks: tab.blocks
  })
  pushDiagnosticLog('info', 'editor-state:programmatic-excalidraw-markdown-applied-to-muya', {
    id: tab.id,
    pathname: tab.pathname || null,
    markdownLength: nextMarkdown.length,
    protectedMs: PROGRAMMATIC_MARKDOWN_PROTECTION_MS
  })
}

export const installStoreDiagnostics = () => {
  const projectStore = useProjectStore()
  const editorStore = useEditorStore()
  const layoutStore = useLayoutStore()
  const vaultStore = useVaultStore()

  installEditorContentGuard(editorStore)

  pushDiagnosticLog('info', 'store-diagnostics:installed', {
    project: summarizeProject(projectStore),
    editor: summarizeEditor(editorStore),
    layout: summarizeLayout(layoutStore)
  })

  const initialId = editorStore.currentFile?.id
  const initialTab = initialId ? editorStore.tabs.find((tab) => tab.id === initialId) : null
  if (initialTab) protectProgrammaticMarkdownIfNeeded(initialTab, vaultStore, projectStore)

  projectStore.$subscribe((mutation) => {
    if (!isDiagnosticVerbose()) return
    pushDiagnosticLog('info', 'pinia:project', {
      type: mutation.type,
      project: summarizeProject(projectStore)
    })
  })

  vaultStore.$subscribe(() => {
    const currentId = editorStore.currentFile?.id
    const activeTab = currentId ? editorStore.tabs.find((tab) => tab.id === currentId) : null
    if (activeTab) protectProgrammaticMarkdownIfNeeded(activeTab, vaultStore, projectStore)
  })

  editorStore.$subscribe((mutation) => {
    const currentId = editorStore.currentFile?.id
    const activeTab = currentId ? editorStore.tabs.find((tab) => tab.id === currentId) : null
    if (activeTab) protectProgrammaticMarkdownIfNeeded(activeTab, vaultStore, projectStore)
    if (activeTab && activeTab !== editorStore.currentFile) {
      editorStore.currentFile = activeTab
      pushDiagnosticLog('info', 'editor-state:current-file-synced-from-tab', {
        id: currentId,
        pathname: activeTab.pathname || null,
        markdownLength: typeof activeTab.markdown === 'string' ? activeTab.markdown.length : 0
      })
    }
    pushDiagnosticLog('info', 'pinia:editor', {
      type: mutation.type,
      editor: summarizeEditor(editorStore)
    })
  })

  layoutStore.$subscribe((mutation) => {
    if (!isDiagnosticVerbose()) return
    pushDiagnosticLog('info', 'pinia:layout', {
      type: mutation.type,
      layout: summarizeLayout(layoutStore)
    })
  })
}
