import { defineStore } from 'pinia'
import log from '@/platform/runtimeLogShim'
import { elephantnoteClient } from '../services/elephantnoteClient'
import { useSearchStore } from './searchStore'
import {
  createCalendarBuckets,
  createGraphModel,
  createRecentNoteEntries,
  createTagTopics,
  createWorkspaceStats,
  isNoteEntry
} from 'common/elephantnote/workspaceInsights'

import { useNavigationStore } from './navigationStore'

const WORKSPACE_VIEWS = ['notes', 'wiki', 'chat', 'canvas', 'graph', 'calendar', 'models']

const getPinnedNotesStorageKey = (vault) => {
  const scope = vault?.id || vault?.path || 'default'
  return `elephantnote:pinnedNotes:${scope}`
}

const readPinnedNotePaths = (key) => {
  try {
    const raw = window.localStorage.getItem(key)
    const parsed = raw ? JSON.parse(raw) : []
    return Array.isArray(parsed) ? parsed.filter((path) => typeof path === 'string' && path) : []
  } catch {
    return []
  }
}

const writePinnedNotePaths = (key, paths) => {
  window.localStorage.setItem(key, JSON.stringify(paths))
}

const replacePathPrefix = (pathname, oldPath, newPath) => {
  if (!pathname || !oldPath || !newPath) return pathname
  if (pathname === oldPath) return newPath
  if (pathname.startsWith(`${oldPath}/`)) {
    return `${newPath}${pathname.slice(oldPath.length)}`
  }
  return pathname
}

const removePathsByPrefix = (paths, targetPath) => {
  return paths.filter((pathname) => {
    if (!pathname) return false
    return pathname !== targetPath && !pathname.startsWith(`${targetPath}/`)
  })
}

const getRenamedRelativePath = (oldPath, title) => {
  const parts = String(oldPath || '').split('/').filter(Boolean)
  if (parts.length <= 1) return title
  return [...parts.slice(0, -1), title].join('/')
}

const joinRelativePath = (...parts) => parts
  .flatMap((part) => String(part || '').split('/'))
  .filter(Boolean)
  .join('/')

const getMovedRelativePath = (oldPath, targetDirectoryPath = '') => {
  if (!oldPath) return ''
  return joinRelativePath(targetDirectoryPath, oldPath.split('/').pop())
}

const getParentRelativePath = (pathname = '') => {
  const parts = String(pathname || '').split('/').filter(Boolean)
  if (parts.length <= 1) return ''
  return parts.slice(0, -1).join('/')
}

const isMovingIntoSelf = (sourcePath, targetDirectoryPath) => {
  if (!sourcePath || !targetDirectoryPath) return false
  return targetDirectoryPath === sourcePath || targetDirectoryPath.startsWith(`${sourcePath}/`)
}

const entryArray = (value) => Array.isArray(value) ? value : []

const getSidebarItems = (workspace) => entryArray(workspace?.sidebar)

const getWorkspaceResult = (result, fallback = null) => {
  if (result?.workspace && typeof result.workspace === 'object') return result.workspace
  if (result && typeof result === 'object' && Array.isArray(result.sidebar)) return result
  return fallback
}

const isAttachedSidebarEntry = (item) => item?.type === 'note' || item?.type === 'folder'

export const useVaultStore = defineStore('elephantnoteVaults', {
  state: () => ({
    vaults: [],
    activeVaultId: null,
    activeVault: null,
    workspace: null,
    entries: [],
    rootEntries: [],
    currentPath: '',
    activeWorkspaceView: 'notes',
    chatSidebarOpen: false,
    chatSidebarWidth: 460,
    chatHistoryOpen: false,
    chatMode: 'advanced',
    filter: 'all',
    sort: 'updated-newest',
    viewMode: 'grid',
    loading: false,
    error: '',
    wikiRecords: [],
    wikiLoading: false,
    openedNotePath: '',
    openedNotes: [],
    pinnedNotePaths: []
  }),

  getters: {
    hasVault: (state) => !!state.activeVault,
    activeEntries(state) {
      let entries = [...entryArray(state.entries)]
      const pinned = new Set(entryArray(state.pinnedNotePaths))
      entries.sort((a, b) => {
        const aPinned = pinned.has(a.path)
        const bPinned = pinned.has(b.path)
        if (aPinned !== bPinned) return aPinned ? -1 : 1
        if (state.sort === 'updated-oldest') {
          return new Date(a.updatedAt) - new Date(b.updatedAt)
        }
        if (state.sort === 'title') {
          return a.title.localeCompare(b.title)
        }
        return new Date(b.updatedAt) - new Date(a.updatedAt)
      })
      return entries
    },
    activeNoteEntries() {
      return this.activeEntries.filter(isNoteEntry)
    },
    rootSidebarEntries(state) {
      const pinned = new Set(entryArray(state.pinnedNotePaths))
      return [...entryArray(state.rootEntries)].sort((a, b) => {
        const aPinned = pinned.has(a.path)
        const bPinned = pinned.has(b.path)
        if (aPinned !== bPinned) return aPinned ? -1 : 1
        if ((a.kind || a.type) !== (b.kind || b.type)) {
          return (a.kind || a.type) === 'folder' ? -1 : 1
        }
        return String(a.title || '').localeCompare(String(b.title || ''))
      })
    },
    workspaceStats(state) {
      return createWorkspaceStats({
        entries: entryArray(state.rootEntries),
        recentNoteEntries: this.recentNoteEntries
      })
    },
    tagTopics(state) {
      return createTagTopics(entryArray(state.rootEntries))
    },
    calendarBuckets(state) {
      return createCalendarBuckets(entryArray(state.rootEntries))
    },
    graphModel(state) {
      const searchStore = useSearchStore()
      const semanticGraph = searchStore.indexInspection?.graph
      if (semanticGraph?.nodes?.length) return semanticGraph
      return createGraphModel({
        entries: entryArray(state.rootEntries),
        tagTopics: this.tagTopics
      })
    },
    recentNoteEntries(state) {
      return createRecentNoteEntries({
        entries: entryArray(state.entries),
        openedNotes: entryArray(state.openedNotes),
        pinnedNotePaths: entryArray(state.pinnedNotePaths)
      })
    },
    sidebarItems(state) {
      return getSidebarItems(state.workspace)
    },
    sidebarAttachedItems() {
      return this.sidebarItems.filter(isAttachedSidebarEntry)
    },
    wikiProposals(state) {
      return entryArray(state.wikiRecords).filter((record) => record.status !== 'dismissed')
    }
  },

  actions: {
    loadPinnedNotes() {
      this.pinnedNotePaths = readPinnedNotePaths(getPinnedNotesStorageKey(this.activeVault))
    },

    persistPinnedNotes() {
      writePinnedNotePaths(getPinnedNotesStorageKey(this.activeVault), this.pinnedNotePaths)
    },

    isEntryPinned(pathname) {
      return entryArray(this.pinnedNotePaths).includes(pathname)
    },

    isNotePinned(pathname) {
      return this.isEntryPinned(pathname)
    },

    togglePinnedEntry(pathname) {
      if (!pathname) return false
      const pinnedPaths = entryArray(this.pinnedNotePaths)
      const index = pinnedPaths.indexOf(pathname)
      if (index === -1) {
        this.pinnedNotePaths = [pathname, ...pinnedPaths]
      } else {
        this.pinnedNotePaths = [
          ...pinnedPaths.slice(0, index),
          ...pinnedPaths.slice(index + 1)
        ]
      }
      this.persistPinnedNotes()
      return this.isEntryPinned(pathname)
    },

    togglePinnedNote(pathname) {
      return this.togglePinnedEntry(pathname)
    },

    isEntryVisibleInSidebar(pathname) {
      return !!pathname && this.sidebarAttachedItems.some((item) => item.path === pathname)
    },

    isFolderVisibleInSidebar(pathname) {
      return this.isEntryVisibleInSidebar(pathname)
    },

    async attachEntryToSidebar(entry) {
      if (!entry?.path) return false
      if (this.isEntryVisibleInSidebar(entry.path)) return true
      const payload = {
        relativePath: entry.path,
        title: entry.title || entry.filename?.replace(/\.md$/i, '') || entry.path.split('/').pop(),
        type: entry.kind || entry.type
      }
      const result = await elephantnoteClient.sidebar.attach(payload)
      this.workspace = getWorkspaceResult(result, this.workspace)
      return true
    },

    async detachEntryFromSidebar(pathname) {
      if (!pathname) return false
      const result = await elephantnoteClient.sidebar.detach(pathname)
      this.workspace = getWorkspaceResult(result, this.workspace)
      return true
    },

    async toggleEntrySidebarVisibility(entry) {
      if (!entry?.path) return false
      if (this.isEntryVisibleInSidebar(entry.path)) {
        await this.detachEntryFromSidebar(entry.path)
        return false
      }
      await this.attachEntryToSidebar(entry)
      return true
    },

    applyPayload(payload) {
      if (!payload || payload.canceled) return
      const previousActiveVaultId = this.activeVaultId
      const previousOpenedNotePath = this.openedNotePath
      const previousOpenedNotes = this.openedNotes
      const preservesActiveNote = Boolean(
        previousActiveVaultId &&
        previousActiveVaultId === payload.activeVaultId &&
        previousOpenedNotePath
      )
      log.info('[vault] applyPayload', {
        vaultCount: Array.isArray(payload.vaults) ? payload.vaults.length : 0,
        entryCount: Array.isArray(payload.entries) ? payload.entries.length : 0,
        activeVaultId: payload.activeVaultId || null,
        preservesActiveNote,
        openedNotePath: previousOpenedNotePath || null
      })
      this.vaults = entryArray(payload.vaults)
      this.activeVaultId = payload.activeVaultId || null
      this.activeVault = payload.activeVault || null
      this.workspace = payload.workspace || null
      this.entries = entryArray(payload.entries)
      this.rootEntries = entryArray(payload.entries)
      this.openedNotePath = preservesActiveNote ? previousOpenedNotePath : ''
      this.openedNotes = preservesActiveNote ? previousOpenedNotes : []
      this.wikiRecords = []
      this.currentPath = ''
      this.activeWorkspaceView = 'notes'
      this.loadPinnedNotes()
      window.dispatchEvent(new CustomEvent('elephantnote:vault-active-changed', {
        detail: {
          activeVaultId: this.activeVaultId,
          activeVaultPath: this.activeVault?.path || null
        }
      }))
    },

    async load() {
      this.loading = true
      this.error = ''
      try {
        log.info('[vault] load:start')
        this.applyPayload(await elephantnoteClient.vaults.get())
        useNavigationStore().reset({ type: 'all_notes' })
        log.info('[vault] load:done', {
          activeVaultId: this.activeVaultId || null,
          entryCount: Array.isArray(this.entries) ? this.entries.length : 0
        })
      } catch (err) {
        log.error('[vault] load failed', err)
        this.error = err.message || 'Unable to load vaults.'
      } finally {
        this.loading = false
      }
    },

    async chooseVault() {
      this.loading = true
      this.error = ''
      try {
        log.info('[vault] chooseVault:start')
        const payload = await elephantnoteClient.vaults.select()
        if (payload?.canceled) return false
        this.applyPayload(payload)
        this.currentPath = ''
        useNavigationStore().push({
          type: 'workspace',
          id: this.activeVaultId,
          title: this.activeVault?.name
        })
        log.info('[vault] chooseVault:done', {
          activeVaultId: this.activeVaultId || null
        })
        return true
      } catch (err) {
        log.error('[vault] chooseVault failed', err)
        this.error = err.message || 'Unable to choose vault.'
        return false
      } finally {
        this.loading = false
      }
    },

    async setActiveVault(vaultId, options = {}) {
      this.loading = true
      this.error = ''
      try {
        log.info('[vault] setActiveVault:start', { vaultId })
        this.applyPayload(await elephantnoteClient.vaults.setActive(vaultId))
        this.currentPath = ''
        if (options.record !== false) {
          useNavigationStore().push({
            type: 'workspace',
            id: this.activeVaultId,
            title: this.activeVault?.name
          })
        }
        log.info('[vault] setActiveVault:done', {
          vaultId,
          activeVaultId: this.activeVaultId || null
        })
      } catch (err) {
        log.error('[vault] setActiveVault failed', err)
        this.error = err.message || 'Unable to switch vault.'
      } finally {
        this.loading = false
      }
    },

    async setVaultIcon(vaultId, icon = '') {
      if (!vaultId) return false
      const normalizedIcon = String(icon || '').trim()
      this.vaults = this.vaults.map((vault) => vault.id === vaultId
        ? { ...vault, icon: normalizedIcon }
        : vault)
      if (this.activeVault?.id === vaultId) {
        this.activeVault = { ...this.activeVault, icon: normalizedIcon }
      }
      const payload = await elephantnoteClient.vaults.setIcon(vaultId, icon)
      this.applyPayload(payload)
      return true
    },

    async setVaultName(vaultId, name = '') {
      if (!vaultId) return false
      const normalizedName = String(name || '').trim()
      if (!normalizedName) return false
      this.vaults = this.vaults.map((vault) => vault.id === vaultId
        ? { ...vault, name: normalizedName }
        : vault)
      if (this.activeVault?.id === vaultId) {
        this.activeVault = { ...this.activeVault, name: normalizedName }
      }
      const payload = await elephantnoteClient.vaults.setName(vaultId, normalizedName)
      this.applyPayload(payload)
      return true
    },

    async removeVault(vaultId) {
      if (!vaultId) return false
      const payload = await elephantnoteClient.vaults.remove(vaultId)
      this.applyPayload(payload)
      return true
    },

    async openDirectory(relativePath = '', options = {}) {
      log.info('[vault] openDirectory:start', {
        relativePath,
        previousOpenedNotePath: this.openedNotePath || null,
        previousCurrentPath: this.currentPath || ''
      })
      this.currentPath = relativePath
      this.openedNotePath = ''
      this.activeWorkspaceView = 'notes'
      const entries = entryArray(await elephantnoteClient.directory.list(relativePath))
      this.entries = entries
      if (!relativePath) {
        this.rootEntries = entries
      }
      if (options.record !== false) {
        useNavigationStore().push(relativePath
          ? { type: 'folder', id: relativePath, path: relativePath }
          : { type: 'all_notes' })
      }
      log.info('[vault] openDirectory:done', {
        relativePath,
        entryCount: this.entries.length,
        openedNotePath: this.openedNotePath || null
      })
    },

    async loadWiki({ regenerate = false } = {}) {
      if (!this.activeVault?.path) return
      this.wikiLoading = true
      try {
        const wiki = regenerate
          ? await elephantnoteClient.wiki.propose()
          : await elephantnoteClient.wiki.list()
        this.wikiRecords = wiki.records || []
      } finally {
        this.wikiLoading = false
      }
    },

    async acceptWikiProposal(id) {
      const result = await elephantnoteClient.wiki.accept(id)
      this.wikiRecords = result.wiki?.records || []
      if (result.note?.path) {
        this.currentPath = 'Wiki'
        this.entries = result.entries || this.entries
        this.openedNotePath = result.note.path
        this.rememberNote({
          kind: 'note',
          path: result.note.path,
          title: result.note.path.split('/').pop()?.replace(/\.md$/i, '') || 'Wiki',
          updatedAt: new Date().toISOString()
        })
        window.tauri.ipcRenderer.send('mt::open-file', result.note.fullPath, {})
      }
    },

    async dismissWikiProposal(id) {
      const wiki = await elephantnoteClient.wiki.dismiss(id)
      this.wikiRecords = wiki.records || []
    },

    async refreshSavedNote(pathname) {
      if (!this.activeVault?.path || !pathname) return

      const relativePath = window.path.relative(this.activeVault.path, pathname)
      if (!relativePath || relativePath.startsWith('..') || window.path.isAbsolute(relativePath)) return

      const parentPath = window.path.dirname(relativePath)
      const directoryPath = parentPath === '.' ? '' : parentPath
      const entries = entryArray(await elephantnoteClient.directory.list(directoryPath))
      const savedEntry = entries.find((entry) => entry.path === relativePath)

      if (directoryPath === this.currentPath) {
        this.entries = entries
      }
      if (savedEntry) {
        this.rememberNote(savedEntry)
      }
    },

    rememberNote(entry) {
      if (!entry?.path) return
      this.openedNotes = [
        {
          path: entry.path,
          title: entry.title || entry.path.split('/').pop()?.replace(/\.md$/i, '') || 'Untitled',
          kind: 'note',
          type: entry.type || 'note',
          updatedAt: entry.updatedAt || new Date().toISOString()
        },
        ...entryArray(this.openedNotes).filter((note) => note.path !== entry.path)
      ].slice(0, 8)
    },

    updateNoteMetadata(pathname, metadata = {}) {
      if (!pathname) return
      const applyMetadata = (entry) => {
        if (!entry || entry.path !== pathname) return entry
        return {
          ...entry,
          title: metadata.title || entry.title,
          tags: Array.isArray(metadata.tags) ? metadata.tags : entry.tags,
          updatedAt: metadata.updatedAt || entry.updatedAt
        }
      }
      this.entries = entryArray(this.entries).map(applyMetadata)
      this.rootEntries = entryArray(this.rootEntries).map(applyMetadata)
      this.openedNotes = entryArray(this.openedNotes).map(applyMetadata)
    },

    openNote(entry, options = {}) {
      log.info('[vault] openNote:start', {
        path: entry?.path || null,
        previousOpenedNotePath: this.openedNotePath || null,
        activeVaultId: this.activeVaultId || null,
        record: options.record !== false
      })
      if (this.openedNotePath === entry.path) {
        log.info('[vault] openNote:skip-already-open', { path: entry.path })
        return
      }
      this.openedNotePath = entry.path
      this.rememberNote(entry)
      window.tauri.ipcRenderer.send('mt::open-file', `${this.activeVault.path}/${entry.path}`, {})
      if (options.record !== false) {
        useNavigationStore().push({
          type: 'note',
          id: entry.path,
          path: entry.path,
          title: entry.title
        })
      }
      log.info('[vault] openNote:done', {
        path: this.openedNotePath,
        currentPath: this.currentPath || null,
        openedNotesCount: this.openedNotes.length
      })
    },

    closeNote() {
      log.info('[vault] closeNote', { previousOpenedNotePath: this.openedNotePath || null })
      this.openedNotePath = ''
    },

    setWorkspaceView(view, options = {}) {
      log.info('[vault] setWorkspaceView:start', {
        view,
        previousView: this.activeWorkspaceView,
        previousOpenedNotePath: this.openedNotePath || null
      })
      this.activeWorkspaceView = WORKSPACE_VIEWS.includes(view)
        ? view
        : 'notes'
      this.openedNotePath = ''
      if (options.record !== false) {
        useNavigationStore().push({ type: view })
      }
      log.info('[vault] setWorkspaceView:done', {
        view: this.activeWorkspaceView,
        openedNotePath: this.openedNotePath || null
      })
    },

    openChatSidebar() {
      this.chatSidebarOpen = true
    },

    closeChatSidebar() {
      this.chatSidebarOpen = false
    },

    toggleChatSidebar() {
      this.chatSidebarOpen = !this.chatSidebarOpen
    },

    toggleChatHistory() {
      this.chatHistoryOpen = !this.chatHistoryOpen
    },

    setChatSidebarWidth(width) {
      const parsed = Number(width)
      this.chatSidebarWidth = Math.min(720, Math.max(360, Number.isFinite(parsed) ? parsed : 460))
    },

    setChatMode(mode) {
      this.chatMode = mode || 'advanced'
    },

    async navigateTo(entry) {
      if (!entry?.type) return
      if (entry.type === 'all_notes') {
        await this.openDirectory('', { record: false })
        return
      }
      if (entry.type === 'folder') {
        await this.openDirectory(entry.path || entry.id || '', { record: false })
        return
      }
      if (entry.type === 'note') {
        const path = entry.path || entry.id
        if (!path) return
        const note = [...entryArray(this.entries), ...entryArray(this.rootEntries), ...entryArray(this.openedNotes)]
          .find((item) => item?.path === path) || {
          path,
          title: entry.title || path.split('/').pop()?.replace(/\.md$/i, '') || 'Untitled',
          kind: 'note',
          type: 'note'
        }
        this.openNote(note, { record: false })
        return
      }
      if (entry.type === 'workspace' && entry.id) {
        await this.setActiveVault(entry.id, { record: false })
        return
      }
      if (WORKSPACE_VIEWS.includes(entry.type)) {
        this.setWorkspaceView(entry.type, { record: false })
      }
    },

    async createNote() {
      const result = await elephantnoteClient.notes.create(this.currentPath)
      const resultEntries = entryArray(result?.entries)
      this.entries = resultEntries
      if (!this.currentPath) {
        this.rootEntries = resultEntries
      }
      this.openedNotePath = result.note.path
      this.rememberNote(result.note)
      window.tauri.ipcRenderer.send('mt::open-file', result.note.fullPath, {})
      useNavigationStore().push({
        type: 'note',
        id: result.note.path,
        path: result.note.path,
        title: result.note.title
      })
    },

    async ensureDashboardNote() {
      if (this.openedNotePath && isDashboardNotePath(this.openedNotePath)) {
        return {
          path: this.openedNotePath,
          title: 'Dashboard'
        }
      }

      const existing = [...entryArray(this.entries), ...entryArray(this.rootEntries), ...entryArray(this.openedNotes)]
        .find((entry) => isDashboardNotePath(entry?.path))
      if (existing) {
        this.openNote(existing, { record: false })
        return existing
      }

      const result = await elephantnoteClient.notes.create(buildDashboardNoteCreatePayload())
      const resultEntries = entryArray(result?.entries)
      this.entries = resultEntries
      if (!this.currentPath) {
        this.rootEntries = resultEntries
      }
      const dashboardNote = {
        path: result.note.path || DASHBOARD_NOTE_RELATIVE_PATH,
        title: result.note.title || 'Dashboard',
        kind: 'note',
        type: 'note',
        updatedAt: new Date().toISOString()
      }
      this.openNote(dashboardNote, { record: false })
      return dashboardNote
    },

    async createFolder() {
      const result = await elephantnoteClient.folders.create(this.currentPath)
      const resultEntries = entryArray(result?.entries)
      this.entries = resultEntries
      if (!this.currentPath) {
        this.rootEntries = resultEntries
      }
    },

    async renameEntry(entry, title) {
      const oldPath = entry?.path
      const result = await elephantnoteClient.entries.rename({
        relativePath: entry.path,
        title
      })
      this.workspace = getWorkspaceResult(result, this.workspace)
      const resultEntries = entryArray(result?.entries)
      this.entries = resultEntries
      if (!this.currentPath) {
        this.rootEntries = resultEntries
      }
      if (oldPath) {
        const nextPath = getRenamedRelativePath(oldPath, title)
        this.pinnedNotePaths = entryArray(this.pinnedNotePaths).map((pathname) => replacePathPrefix(pathname, oldPath, nextPath))
        this.currentPath = replacePathPrefix(this.currentPath, oldPath, nextPath)
        this.openedNotePath = replacePathPrefix(this.openedNotePath, oldPath, nextPath)
        this.persistPinnedNotes()
      }
      if (this.openedNotePath === entry.path) this.closeNote()
    },

    async moveEntry(entry, targetDirectoryPath = '') {
      const oldPath = entry?.path
      if (!oldPath) return false

      const normalizedTargetDirectory = joinRelativePath(targetDirectoryPath)
      if (isMovingIntoSelf(oldPath, normalizedTargetDirectory)) return false
      if (getParentRelativePath(oldPath) === normalizedTargetDirectory) return false

      const nextPath = getMovedRelativePath(oldPath, normalizedTargetDirectory)
      if (!nextPath || nextPath === oldPath) return false

      const oldOpenedNotePath = this.openedNotePath
      const result = await elephantnoteClient.entries.move({
        relativePath: oldPath,
        targetDirectoryPath: normalizedTargetDirectory
      })

      this.workspace = getWorkspaceResult(result, this.workspace)
      this.pinnedNotePaths = entryArray(this.pinnedNotePaths).map((pathname) =>
        replacePathPrefix(pathname, oldPath, nextPath)
      )
      this.currentPath = replacePathPrefix(this.currentPath, oldPath, nextPath)
      this.openedNotePath = replacePathPrefix(this.openedNotePath, oldPath, nextPath)
      this.openedNotes = entryArray(this.openedNotes).map((note) => ({
        ...note,
        path: replacePathPrefix(note.path, oldPath, nextPath),
        title: note.path === oldPath
          ? nextPath.split('/').pop()?.replace(/\.md$/i, '') || note.title
          : note.title
      }))
      this.persistPinnedNotes()

      this.entries = entryArray(await elephantnoteClient.directory.list(this.currentPath))
      this.rootEntries = entryArray(await elephantnoteClient.directory.list(''))

      if (oldOpenedNotePath && oldOpenedNotePath !== this.openedNotePath && this.openedNotePath) {
        window.tauri.ipcRenderer.send('mt::open-file', `${this.activeVault.path}/${this.openedNotePath}`, {})
      }

      return true
    },

    async deleteEntry(entry) {
      const oldPath = entry?.path
      const result = await elephantnoteClient.entries.delete(entry.path)
      this.workspace = getWorkspaceResult(result, this.workspace)
      if (Array.isArray(result?.entries)) {
        this.entries = result.entries
        if (!this.currentPath) this.rootEntries = result.entries
      } else {
        this.entries = entryArray(await elephantnoteClient.directory.list(this.currentPath))
        this.rootEntries = this.currentPath
          ? entryArray(await elephantnoteClient.directory.list(''))
          : this.entries
      }
      if (oldPath) {
        this.pinnedNotePaths = removePathsByPrefix(this.pinnedNotePaths, oldPath)
        this.currentPath = removePathsByPrefix([this.currentPath], oldPath)[0] || ''
        this.openedNotePath = removePathsByPrefix([this.openedNotePath], oldPath)[0] || ''
        this.persistPinnedNotes()
      }
      if (this.openedNotePath === entry.path) this.closeNote()
    },

    notifyAiUnavailable() {
      window.tauri.ipcRenderer.send('mt::show-notification', {
        title: 'AI features are not available yet.',
        type: 'info'
      })
    },

    openSettings() {
      window.tauri.ipcRenderer.send('mt::open-setting-window')
    }
  }
})
