import { defineStore } from 'pinia'
import log from '@/platform/runtimeLogShim'
import { useVaultStore } from './vaultStore'
import { elephantnoteClient } from '../services/elephantnoteClient'
import {
  getDocumentTitle,
  parseFrontmatter,
  parseMarkdownTags
} from 'common/elephantnote/markdownDocument'

const DEFAULT_STATUS = Object.freeze({
  status: 'not_initialized',
  vaultPath: '',
  indexedDocuments: 0,
  totalDocuments: 0,
  message: '',
  error: ''
})

const EMPTY_INSPECTION = Object.freeze({
  indexPath: '',
  documents: [],
  folders: [],
  semanticLinks: [],
  graph: null,
  generatedAt: ''
})

const documentSearchCache = new WeakMap()

const clampQueryLimit = (value) => {
  const parsed = Number(value)
  if (!Number.isFinite(parsed)) return 20
  return Math.max(1, Math.min(50, Math.trunc(parsed)))
}

const loadSearchPreference = (key, fallback) => {
  const value = window.localStorage.getItem(`elephantnote:search:${key}`)
  return value === null ? fallback : value
}

const loadBooleanSearchPreference = (key, fallback = false) => {
  const value = loadSearchPreference(key, String(Boolean(fallback)))
  return value === true || value === 'true'
}

const withTimeout = (promise, timeoutMs, message) => {
  let timeoutId
  const timeout = new Promise((_resolve, reject) => {
    timeoutId = window.setTimeout(() => reject(new Error(message)), timeoutMs)
  })
  return Promise.race([promise, timeout]).finally(() => window.clearTimeout(timeoutId))
}

const normalizeRelativePath = (relativePath = '') => String(relativePath || '')
  .replaceAll(String.fromCharCode(92), '/')
  .split('/')
  .filter((part) => part && part !== '.')
  .join('/')

const basenameTitle = (relativePath = '') => {
  const name = normalizeRelativePath(relativePath).split('/').pop() || ''
  return name.replace(/\.md$/i, '') || 'Untitled'
}

const getDocumentPath = (document) => normalizeRelativePath(document?.relativePath || document?.path || '')

const searchLog = (event, details = {}) => {
  console.info(`[search] ${event}`, details)
}

const normalizeDocumentForIndex = (relativePath = '', markdown = '', metadata = {}) => {
  const path = normalizeRelativePath(relativePath)
  const content = String(markdown || '')
  const frontmatter = parseFrontmatter(content)
  const title = metadata.title || getDocumentTitle(content, basenameTitle(path))
  const tags = Array.isArray(metadata.tags) ? metadata.tags : parseMarkdownTags(content)
  const body = frontmatter.body || content
  const excerpt = body.replace(/^#\s+.*$/m, '').trim().slice(0, 500)
  const updatedAt = metadata.updatedAt || frontmatter.fields.updatedAt || new Date().toISOString()

  return {
    relativePath: path,
    path,
    title,
    tags,
    content,
    body,
    excerpt,
    updatedAt
  }
}

const normalizeBackendResult = (result = {}) => ({
  ...result,
  relativePath: normalizeRelativePath(result.relativePath || result.path || ''),
  path: normalizeRelativePath(result.path || result.relativePath || '')
})

const getDocumentSearchMeta = (document) => {
  const cached = documentSearchCache.get(document)
  if (cached) return cached

  const path = String(getDocumentPath(document)).toLowerCase()
  const title = String(document.title || '').toLowerCase()
  const tags = Array.isArray(document.tags) ? document.tags.join(' ').toLowerCase() : ''
  const body = String(document.body || document.content || document.excerpt || '').toLowerCase()
  const meta = {
    path,
    title,
    tags,
    body,
    sortTitle: String(document.title || '')
  }
  documentSearchCache.set(document, meta)
  return meta
}

const scoreDocument = (document, query = '') => {
  const needle = String(query || '').trim().toLowerCase()
  if (!needle) return 0

  const meta = getDocumentSearchMeta(document)
  return (meta.title.includes(needle) ? 10 : 0) +
    (meta.tags.includes(needle) ? 6 : 0) +
    (meta.path.includes(needle) ? 4 : 0) +
    (meta.body.includes(needle) ? 2 : 0)
}

const compareRankedSearchResult = (a, b) => {
  const scoreDiff = b.score - a.score
  if (scoreDiff) return scoreDiff
  return getDocumentSearchMeta(a.document).sortTitle.localeCompare(getDocumentSearchMeta(b.document).sortTitle)
}

const addRankedSearchResult = (ranked, item, limit) => {
  if (ranked.length < limit) {
    ranked.push(item)
    ranked.sort(compareRankedSearchResult)
    return
  }
  const last = ranked[ranked.length - 1]
  if (compareRankedSearchResult(item, last) >= 0) return
  ranked[ranked.length - 1] = item
  ranked.sort(compareRankedSearchResult)
}

const toSearchResult = (document, score = 0) => ({
  relativePath: getDocumentPath(document),
  title: document.title || basenameTitle(getDocumentPath(document)),
  excerpt: document.excerpt || document.body || '',
  tags: document.tags || [],
  score
})

const normalizeConceptCandidate = (candidate = {}) => ({
  ...candidate,
  id: String(candidate.id || ''),
  title: String(candidate.title || candidate.id || 'Concept'),
  score: Number(candidate.score || 0),
  evidenceChunks: Array.isArray(candidate.evidenceChunks) ? candidate.evidenceChunks : []
})

const emptyInspection = () => ({ ...EMPTY_INSPECTION })

export const useSearchStore = defineStore('elephantnoteSearch', {
  state: () => ({
    isOpen: false,
    vaultPath: '',
    query: '',
    mode: 'exact',
    results: [],
    conceptResults: [],
    conceptRoute: null,
    status: { ...DEFAULT_STATUS },
    indexInspection: emptyInspection(),
    busy: false,
    error: '',
    queryLimit: clampQueryLimit(loadSearchPreference('queryLimit', 20)),
    defaultMode: loadSearchPreference('defaultMode', 'exact'),
    visualizationMode: loadSearchPreference('visualizationMode', 'list'),
    graphDensity: Number(loadSearchPreference('graphDensity', 1)) || 1,
    showVisualizationLabels: loadBooleanSearchPreference('showVisualizationLabels'),
    showFolderClusters: loadBooleanSearchPreference('showFolderClusters'),
    autoRefreshInspection: loadBooleanSearchPreference('autoRefreshInspection'),
    polling: false,
    lastStatusRefreshAt: 0,
    lastSearchRequestId: 0
  }),

  actions: {
    async ensureActiveVault() {
      const vaultStore = useVaultStore()
      const vaultPath = vaultStore.activeVault?.path || this.vaultPath
      if (!vaultPath) {
        searchLog('ensureActiveVault:no-vault')
        return this.status
      }
      this.vaultPath = vaultPath
      if (this.status.vaultPath !== vaultPath && this.status.status !== 'disabled') {
        searchLog('ensureActiveVault:init:start', { vaultPath, previousVaultPath: this.status.vaultPath || '' })
        this.indexInspection = emptyInspection()
        this.results = []
        this.conceptResults = []
        this.conceptRoute = null
        this.status = await elephantnoteClient.search.initVault(vaultPath)
        this.lastStatusRefreshAt = Date.now()
        searchLog('ensureActiveVault:init:done', { vaultPath, status: this.status?.status || '', indexedDocuments: this.status?.indexedDocuments || this.status?.notesIndexed || 0 })
      }
      return this.status
    },

    open() {
      const vaultStore = useVaultStore()
      this.vaultPath = vaultStore.activeVault?.path || ''
      this.isOpen = true
      this.error = ''
      this.mode = this.defaultMode
      searchLog('open', { vaultPath: this.vaultPath, mode: this.mode })
      this.ensureActiveVault().then(() => this.refreshStatus({ throttleMs: 2000 }))
    },

    close() {
      this.isOpen = false
      this.error = ''
      searchLog('close')
    },

    setMode(mode) {
      if (!['smart', 'exact', 'semantic'].includes(mode)) return
      this.mode = mode
      searchLog('mode:set', { mode, query: this.query })
      this.search()
    },

    setQuery(value) {
      const previous = this.query
      this.query = String(value || '')
      searchLog('query:set', { previousLength: previous.length, nextLength: this.query.length, query: this.query })
    },

    updateNoteIndex(relativePath, markdown = '', metadata = {}) {
      const document = normalizeDocumentForIndex(relativePath, markdown, metadata)
      if (!document.relativePath) return false
      const documents = Array.isArray(this.indexInspection.documents)
        ? this.indexInspection.documents
        : []
      const nextDocuments = documents.filter((item) => getDocumentPath(item) !== document.relativePath)

      this.indexInspection = {
        ...this.indexInspection,
        documents: [document, ...nextDocuments],
        generatedAt: new Date().toISOString()
      }
      this.status = {
        ...this.status,
        indexedDocuments: this.indexInspection.documents.length,
        totalDocuments: Math.max(this.status.totalDocuments || 0, this.indexInspection.documents.length)
      }
      searchLog('index:update-note', { relativePath: document.relativePath, length: String(markdown || '').length, documents: this.indexInspection.documents.length })
      if (this.query.trim()) {
        this.results = this.localSearch(this.query, this.queryLimit)
        searchLog('index:update-note:local-results', { query: this.query, results: this.results.length })
      }
      return true
    },

    removeNoteIndex(relativePath) {
      const path = normalizeRelativePath(relativePath)
      if (!path) return false
      this.indexInspection = {
        ...this.indexInspection,
        documents: (this.indexInspection.documents || []).filter((item) => getDocumentPath(item) !== path),
        generatedAt: new Date().toISOString()
      }
      this.results = this.results.filter((item) => normalizeRelativePath(item.relativePath) !== path)
      searchLog('index:remove-note', { relativePath: path, documents: this.indexInspection.documents.length, results: this.results.length })
      return true
    },

    localSearch(query = this.query, limit = this.queryLimit) {
      const documents = Array.isArray(this.indexInspection.documents) ? this.indexInspection.documents : []
      const maxResults = clampQueryLimit(limit)
      const ranked = []
      const startedAt = performance.now()
      for (const document of documents) {
        const score = scoreDocument(document, query)
        if (score > 0) {
          addRankedSearchResult(ranked, { document, score }, maxResults)
        }
      }
      const out = ranked.map((item) => toSearchResult(item.document, item.score))
      searchLog('localSearch:done', { query, documents: documents.length, results: out.length, limit: maxResults, elapsedMs: Math.round(performance.now() - startedAt) })
      return out
    },

    async refreshStatus({ throttleMs = 0 } = {}) {
      const now = Date.now()
      if (throttleMs > 0 && now - this.lastStatusRefreshAt < throttleMs) {
        searchLog('status:skip-throttled', { throttleMs, ageMs: now - this.lastStatusRefreshAt })
        return this.status
      }
      try {
        await this.ensureActiveVault()
        searchLog('status:start', { vaultPath: this.vaultPath })
        this.status = await elephantnoteClient.search.status()
        this.lastStatusRefreshAt = Date.now()
        searchLog('status:done', { status: this.status?.status || '', indexedDocuments: this.status?.indexedDocuments || this.status?.notesIndexed || 0, message: this.status?.message || '' })
      } catch (error) {
        this.status = {
          ...DEFAULT_STATUS,
          status: 'error',
          error: error?.message || 'Unable to fetch search status.',
          message: error?.message || 'Unable to fetch search status.'
        }
        this.lastStatusRefreshAt = Date.now()
        searchLog('status:failed', { error: error?.message || String(error) })
      }
      return this.status
    },

    async inspect() {
      try {
        await this.ensureActiveVault()
        const startedAt = performance.now()
        searchLog('inspect:start', { vaultPath: this.vaultPath || '' })
        const inspection = await elephantnoteClient.search.inspect()
        this.indexInspection = {
          ...emptyInspection(),
          ...(inspection || {}),
          documents: Array.isArray(inspection?.documents)
            ? inspection.documents.map((document) => ({
              ...document,
              relativePath: getDocumentPath(document),
              path: getDocumentPath(document)
            }))
            : [],
          folders: Array.isArray(inspection?.folders) ? inspection.folders : [],
          semanticLinks: Array.isArray(inspection?.semanticLinks) ? inspection.semanticLinks : []
        }
        searchLog('inspect:done', {
          documents: Array.isArray(this.indexInspection?.documents) ? this.indexInspection.documents.length : 0,
          semanticLinks: Array.isArray(this.indexInspection?.semanticLinks) ? this.indexInspection.semanticLinks.length : 0,
          hasGraph: !!this.indexInspection?.graph,
          elapsedMs: Math.round(performance.now() - startedAt)
        })
      } catch (error) {
        log.error('[search] inspect failed', error)
        searchLog('inspect:failed', { error: error?.message || String(error) })
        this.indexInspection = {
          ...emptyInspection(),
          generatedAt: new Date().toISOString()
        }
      }
      return this.indexInspection
    },

    setQueryLimit(value) {
      this.queryLimit = clampQueryLimit(value)
      window.localStorage.setItem('elephantnote:search:queryLimit', String(this.queryLimit))
      searchLog('queryLimit:set', { queryLimit: this.queryLimit })
    },

    setDefaultMode(mode) {
      if (!['smart', 'exact', 'semantic'].includes(mode)) return
      this.defaultMode = mode
      this.mode = mode
      window.localStorage.setItem('elephantnote:search:defaultMode', mode)
      searchLog('defaultMode:set', { mode })
    },

    setVisualizationMode(mode) {
      if (!['space', 'graph', 'list'].includes(mode)) return
      this.visualizationMode = mode
      window.localStorage.setItem('elephantnote:search:visualizationMode', mode)
      searchLog('visualizationMode:set', { mode })
    },

    setGraphDensity(value) {
      const parsed = Number(value)
      const density = Number.isFinite(parsed) ? Math.trunc(parsed) : 4
      this.graphDensity = Math.max(1, Math.min(8, density))
      window.localStorage.setItem('elephantnote:search:graphDensity', String(this.graphDensity))
      searchLog('graphDensity:set', { graphDensity: this.graphDensity })
    },

    setBooleanOption(key, value) {
      if (!['showVisualizationLabels', 'showFolderClusters', 'autoRefreshInspection'].includes(key)) return
      this[key] = Boolean(value)
      window.localStorage.setItem(`elephantnote:search:${key}`, String(this[key]))
      searchLog('option:set', { key, value: this[key] })
    },

    async hydrateLocalFallback() {
      const hasDocuments = Array.isArray(this.indexInspection.documents) && this.indexInspection.documents.length > 0
      searchLog('fallback:hydrate', { hasDocuments, documents: this.indexInspection.documents?.length || 0 })
      if (hasDocuments) return
      await this.inspect()
    },

    async searchConcepts(query = this.query, requestId = this.lastSearchRequestId) {
      const normalizedQuery = String(query || '').trim()
      if (!normalizedQuery || !this.vaultPath) {
        this.conceptResults = []
        this.conceptRoute = null
        searchLog('concepts:skip', { requestId, query: normalizedQuery, vaultPath: this.vaultPath || '' })
        return null
      }
      try {
        const startedAt = performance.now()
        searchLog('concepts:start', { requestId, query: normalizedQuery })
        const route = await withTimeout(
          elephantnoteClient.search.concepts({
            query: normalizedQuery,
            limit: 5,
            evidenceLimit: 4
          }),
          8000,
          'Concept routing timed out.'
        )
        if (requestId !== this.lastSearchRequestId || normalizedQuery !== this.query.trim()) {
          searchLog('concepts:stale-ignore', { requestId, activeRequestId: this.lastSearchRequestId, query: normalizedQuery, activeQuery: this.query.trim() })
          return route || null
        }
        this.conceptRoute = route || null
        this.conceptResults = Array.isArray(route?.candidates)
          ? route.candidates.map(normalizeConceptCandidate)
          : []
        searchLog('concepts:done', { requestId, query: normalizedQuery, candidates: this.conceptResults.length, elapsedMs: Math.round(performance.now() - startedAt) })
        return this.conceptRoute
      } catch (error) {
        log.warn('[search] concept routing unavailable', error)
        if (requestId === this.lastSearchRequestId) {
          this.conceptRoute = null
          this.conceptResults = []
        }
        searchLog('concepts:failed', { requestId, query: normalizedQuery, error: error?.message || String(error) })
        return null
      }
    },

    async search() {
      const query = this.query.trim()
      const requestId = this.lastSearchRequestId + 1
      this.lastSearchRequestId = requestId
      const startedAt = performance.now()
      searchLog('request:start', { requestId, query, mode: this.mode, limit: this.queryLimit, vaultPath: this.vaultPath || '' })
      if (!this.vaultPath) {
        await this.ensureActiveVault()
      }
      if (!this.vaultPath) {
        this.results = []
        this.conceptResults = []
        this.conceptRoute = null
        searchLog('request:no-vault', { requestId, query })
        return []
      }
      if (!query) {
        this.results = []
        this.conceptResults = []
        this.conceptRoute = null
        await this.refreshStatus({ throttleMs: 2000 })
        searchLog('request:empty', { requestId })
        return []
      }

      this.busy = true
      this.error = ''
      const conceptPromise = this.searchConcepts(query, requestId)
      try {
        searchLog('backend:start', { requestId, query, mode: this.mode })
        const results = await withTimeout(
          elephantnoteClient.search.query({
            query,
            mode: this.mode,
            limit: clampQueryLimit(this.queryLimit)
          }),
          15000,
          'Local search timed out. Try Exact mode or rebuild the semantic index.'
        )
        const backendResults = Array.isArray(results)
          ? results.map(normalizeBackendResult).filter((result) => result.relativePath)
          : []
        searchLog('backend:done', { requestId, query, results: backendResults.length, elapsedMs: Math.round(performance.now() - startedAt) })
        if (requestId !== this.lastSearchRequestId || query !== this.query.trim()) {
          searchLog('request:stale-ignore', { requestId, activeRequestId: this.lastSearchRequestId, query, activeQuery: this.query.trim(), backendResults: backendResults.length })
          return this.results
        }
        if (backendResults.length) {
          this.results = backendResults
          searchLog('request:results:set-backend', { requestId, query, results: this.results.length })
          return this.results
        }

        await this.hydrateLocalFallback()
        const localResults = this.localSearch(query, this.queryLimit)
        if (requestId !== this.lastSearchRequestId || query !== this.query.trim()) {
          searchLog('request:stale-ignore-local', { requestId, activeRequestId: this.lastSearchRequestId, query, activeQuery: this.query.trim(), localResults: localResults.length })
          return this.results
        }
        this.results = localResults
        searchLog('request:results:set-local', { requestId, query, results: this.results.length })
        return this.results
      } catch (error) {
        searchLog('backend:failed', { requestId, query, error: error?.message || String(error) })
        await this.hydrateLocalFallback()
        const fallbackResults = this.localSearch(query, this.queryLimit)
        if (requestId !== this.lastSearchRequestId || query !== this.query.trim()) {
          searchLog('request:stale-ignore-error', { requestId, activeRequestId: this.lastSearchRequestId, query, activeQuery: this.query.trim(), fallbackResults: fallbackResults.length })
          return this.results
        }
        if (fallbackResults.length) {
          this.error = ''
          this.results = fallbackResults
          searchLog('request:results:set-fallback', { requestId, query, results: this.results.length })
          return this.results
        }
        this.error = error?.message || 'Search failed.'
        this.results = []
        searchLog('request:no-results', { requestId, query, error: this.error })
        return []
      } finally {
        conceptPromise.catch(() => null)
        if (requestId === this.lastSearchRequestId) this.busy = false
        void this.refreshStatus({ throttleMs: 2000 })
        searchLog('request:finish', { requestId, activeRequestId: this.lastSearchRequestId, query, activeQuery: this.query.trim(), results: this.results.length, busy: this.busy, elapsedMs: Math.round(performance.now() - startedAt) })
      }
    },

    async rebuild() {
      await this.ensureActiveVault()
      this.busy = true
      try {
        searchLog('rebuild:start', { vaultPath: this.vaultPath })
        this.status = await elephantnoteClient.search.rebuild()
        this.lastStatusRefreshAt = Date.now()
        searchLog('rebuild:done', { status: this.status?.status || '', provider: this.status?.provider || '', documents: this.status?.documents || this.status?.notesIndexed || 0 })
        this.pollIndexBuild()
        return this.status
      } finally {
        this.busy = false
      }
    },

    async pollIndexBuild() {
      if (this.polling) return
      this.polling = true
      searchLog('poll:start')
      try {
        for (let attempt = 0; attempt < 240; attempt += 1) {
          await new Promise((resolve) => window.setTimeout(resolve, 1000))
          await this.refreshStatus()
          searchLog('poll:tick', { attempt, status: this.status?.status || '' })
          if (this.status.status !== 'indexing') break
        }
      } finally {
        this.polling = false
        searchLog('poll:done', { status: this.status?.status || '' })
      }
    },

    async clear() {
      this.busy = true
      try {
        searchLog('clear:start')
        this.status = await elephantnoteClient.search.clear()
        this.lastStatusRefreshAt = Date.now()
        this.results = []
        this.conceptResults = []
        this.conceptRoute = null
        this.indexInspection = emptyInspection()
        await this.refreshStatus()
        searchLog('clear:done')
      } finally {
        this.busy = false
      }
    },

    async disable() {
      this.status = await elephantnoteClient.search.disable()
      this.lastStatusRefreshAt = Date.now()
      searchLog('disable:done', { status: this.status?.status || '' })
    },

    async enable() {
      this.status = await elephantnoteClient.search.enable()
      this.lastStatusRefreshAt = Date.now()
      const vaultStore = useVaultStore()
      const vaultPath = vaultStore.activeVault?.path || this.vaultPath
      if (vaultPath) {
        this.vaultPath = vaultPath
        this.status = await elephantnoteClient.search.initVault(vaultPath)
        this.lastStatusRefreshAt = Date.now()
      }
      await this.refreshStatus()
      searchLog('enable:done', { vaultPath, status: this.status?.status || '' })
    },

    openResult(result) {
      const relativePath = normalizeRelativePath(result?.relativePath || result?.path || '')
      searchLog('openResult:start', { relativePath, title: result?.title || '' })
      if (!relativePath) return
      const vaultStore = useVaultStore()
      const vaultPath = this.vaultPath || vaultStore.activeVault?.path || ''
      if (!vaultPath) return
      this.vaultPath = vaultPath

      const existingEntry = [
        ...(vaultStore.entries || []),
        ...(vaultStore.rootEntries || []),
        ...(vaultStore.openedNotes || [])
      ].find((entry) => normalizeRelativePath(entry?.path) === relativePath)

      const noteEntry = existingEntry || {
        path: relativePath,
        title: result?.title || basenameTitle(relativePath),
        kind: 'note',
        type: 'note',
        updatedAt: result?.updatedAt || new Date().toISOString()
      }

      vaultStore.activeWorkspaceView = 'notes'
      if (typeof vaultStore.openNote === 'function') {
        vaultStore.openNote(noteEntry)
      } else {
        window.tauri.ipcRenderer.send('mt::open-file', window.path.join(vaultPath, relativePath), {})
      }
      searchLog('openResult:done', { relativePath })
      this.close()
    }
  }
})
