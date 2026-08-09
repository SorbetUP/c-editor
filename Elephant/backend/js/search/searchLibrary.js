import fs from 'fs-extra'
import path from 'path'
import log from 'electron-log'
import { SEARCH_MODES, SEARCH_STATUSES } from './searchTypes'
import { isIgnoredPath, isMarkdownFile } from './pathSafety'
import { markdownToSearchText } from './markdownToSearchText'
import {
  chunkAtomicMarkdown,
  createAtomicDocument,
  createAtomicSemanticIndex,
  createTextEmbedding,
  searchAtomicSemanticIndex
} from 'common/elephantnote/atomicAiEngine'
import { createKnowledgeChunkIndex } from 'common/elephantnote/knowledge/knowledgeIndex'
import { buildAutomaticOrganizationPlan } from 'common/elephantnote/knowledge/organizationPlanner'
import { createSemanticGraph } from './graphLibrary'

const normalizeVaultRoot = (vaultRoot) => path.resolve(vaultRoot || '')

const createStatus = ({
  status = SEARCH_STATUSES.NOT_INITIALIZED,
  vaultPath = '',
  indexedDocuments = 0,
  totalDocuments = 0,
  message = '',
  error = ''
} = {}) => ({
  status,
  vaultPath,
  indexedDocuments,
  totalDocuments,
  message,
  error
})

const listMarkdownFiles = async (vaultRoot) => {
  const files = []
  const root = normalizeVaultRoot(vaultRoot)

  const walk = async (directory) => {
    const entries = await fs.readdir(directory, { withFileTypes: true }).catch(() => [])
    for (const entry of entries) {
      const absolutePath = path.join(directory, entry.name)
      const relativePath = path.relative(root, absolutePath).split(path.sep).join('/')
      if (isIgnoredPath(relativePath)) continue
      if (entry.isDirectory()) await walk(absolutePath)
      else if (entry.isFile() && isMarkdownFile(absolutePath)) files.push(absolutePath)
    }
  }

  if (root) await walk(root)
  return files.sort((a, b) => a.localeCompare(b))
}

const titleFromMarkdown = (markdown = '', fallback = '') => {
  const frontmatterTitle = markdown.match(/^---\r?\n[\s\S]*?^\s*title:\s*["']?(.+?)["']?\s*$/m)
  if (frontmatterTitle?.[1]) return frontmatterTitle[1].trim()
  const heading = markdown.match(/^#\s+(.+)$/m)
  return heading?.[1]?.trim() || fallback
}

const exactSearchMarkdownFiles = async ({ vaultRoot, query, limit }) => {
  const loweredQuery = String(query || '').toLowerCase()
  if (!loweredQuery) return []

  const files = await listMarkdownFiles(vaultRoot)
  const matches = []

  for (const absolutePath of files) {
    const relativePath = path.relative(vaultRoot, absolutePath).split(path.sep).join('/')
    const markdown = await fs.readFile(absolutePath, 'utf8').catch(() => '')
    const searchable = markdownToSearchText(markdown)
    const haystack = `${relativePath}\n${searchable}`.toLowerCase()
    const index = haystack.indexOf(loweredQuery)
    if (index === -1) continue

    const title = titleFromMarkdown(
      markdown,
      path.basename(relativePath, path.extname(relativePath))
    )
    const snippetSource = searchable || markdown
    const snippetIndex = snippetSource.toLowerCase().indexOf(loweredQuery)
    const start = Math.max(0, snippetIndex - 70)
    const snippet =
      snippetIndex >= 0
        ? snippetSource
            .slice(start, snippetIndex + loweredQuery.length + 90)
            .replace(/\s+/g, ' ')
            .trim()
        : relativePath

    matches.push({
      id: `exact:${relativePath}`,
      uri: `elephantnote://vault/${encodeURI(relativePath)}`,
      title,
      relativePath,
      score: index === 0 ? 1 : 0.75,
      matchType: 'keyword',
      snippets: snippet ? [{ text: snippet, score: 1 }] : []
    })

    if (matches.length >= limit) break
  }

  return matches
}

const coerceEmbeddingVector = (value) => {
  const raw = Array.isArray(value)
    ? value
    : ArrayBuffer.isView(value)
      ? Array.from(value)
      : Array.isArray(value?.embedding)
        ? value.embedding
        : ArrayBuffer.isView(value?.embedding)
          ? Array.from(value.embedding)
          : null

  if (!raw?.length) return null
  const vector = raw.map((entry) => Number(entry))
  return vector.every(Number.isFinite) ? vector : null
}

const runtimeEmbeddingFallback = (text) => createTextEmbedding(text)

const createRuntimeEmbeddedDocument = async ({ relativePath, markdown, embeddingProvider }) => {
  const document = createAtomicDocument({ relativePath, markdown })
  if (!embeddingProvider?.embedText) {
    return {
      ...document,
      markdown
    }
  }
  const safeEmbedText = async (text, fallbackEmbedding) => {
    try {
      const vector = coerceEmbeddingVector(await embeddingProvider.embedText(text))
      if (vector) return vector
      log.warn('[search] embedText returned an invalid vector, using deterministic fallback', {
        relativePath,
        source: embeddingProvider?.source || ''
      })
      return fallbackEmbedding || runtimeEmbeddingFallback(text)
    } catch (error) {
      log.warn('[search] embedText fallback to deterministic index', {
        relativePath,
        error: error instanceof Error ? error.message : String(error || '')
      })
      return fallbackEmbedding || runtimeEmbeddingFallback(text)
    }
  }
  const chunks = await Promise.all(
    chunkAtomicMarkdown(markdown).map(async (chunk) => ({
      ...chunk,
      embedding: await safeEmbedText(chunk.content, chunk.embedding)
    }))
  )
  return {
    ...document,
    markdown,
    chunks,
    embedding: await safeEmbedText(`${document.title}\n${document.plainText}`, document.embedding)
  }
}

const readAtomicDocuments = async (vaultRoot, embeddingProvider = null) => {
  const files = await listMarkdownFiles(vaultRoot)
  const documents = []
  for (const absolutePath of files) {
    const relativePath = path.relative(vaultRoot, absolutePath).split(path.sep).join('/')
    const markdown = await fs.readFile(absolutePath, 'utf8').catch(() => '')
    documents.push(
      await createRuntimeEmbeddedDocument({ relativePath, markdown, embeddingProvider })
    )
  }
  return documents
}

const createKnowledgeIndexFromAtomicDocuments = (documents = []) => createKnowledgeChunkIndex(
  documents.map((document) => ({
    relativePath: document.relativePath,
    title: document.title,
    markdown: document.markdown || document.plainText || '',
    updatedAt: document.updatedAt || ''
  }))
)

const shouldUseRuntimeQueryEmbedding = (semanticIndex, embeddingProvider) =>
  semanticIndex?.embeddingSource &&
  semanticIndex.embeddingSource !== 'deterministic-local' &&
  Boolean(embeddingProvider?.embedText)

const createRuntimeQueryEmbedding = async ({ query, semanticIndex, embeddingProvider }) => {
  if (!shouldUseRuntimeQueryEmbedding(semanticIndex, embeddingProvider)) return null
  try {
    const vector = coerceEmbeddingVector(await embeddingProvider.embedText(query))
    if (vector) return vector
    log.warn('[search] query embedding returned an invalid vector, using deterministic query embedding', {
      source: embeddingProvider?.source || ''
    })
    return null
  } catch (error) {
    log.warn('[search] query embedding fallback to deterministic index', {
      error: error instanceof Error ? error.message : String(error || '')
    })
    return null
  }
}

const localSearchMarkdownFiles = async ({
  vaultRoot,
  query,
  mode,
  limit,
  semanticIndex,
  embeddingProvider
}) => {
  if (mode === SEARCH_MODES.EXACT) return exactSearchMarkdownFiles({ vaultRoot, query, limit })

  const queryEmbedding = await createRuntimeQueryEmbedding({
    query,
    semanticIndex,
    embeddingProvider
  })
  const semanticMatches = searchAtomicSemanticIndex({
    index: semanticIndex,
    query,
    queryEmbedding,
    limit
  })
  if (mode === SEARCH_MODES.SEMANTIC) return semanticMatches

  const exactMatches = await exactSearchMarkdownFiles({ vaultRoot, query, limit })
  const byPath = new Map()
  for (const match of [...semanticMatches, ...exactMatches]) {
    const current = byPath.get(match.relativePath)
    if (!current || match.score > current.score) {
      byPath.set(match.relativePath, {
        ...match,
        matchType: current ? 'hybrid' : match.matchType
      })
    }
  }
  return [...byPath.values()]
    .sort((a, b) => b.score - a.score || a.relativePath.localeCompare(b.relativePath))
    .slice(0, Math.max(1, Math.min(50, Number(limit) || 20)))
}

export const createSearchLibrary = ({ embeddingProvider = null } = {}) => {
  const activeVaultRootByWindow = new Map()
  const statusByVault = new Map()
  const indexByVault = new Map()
  let enabled = true
  let currentEmbeddingProvider = embeddingProvider

  const getStatus = (vaultRoot) => {
    const root = normalizeVaultRoot(vaultRoot)
    return statusByVault.get(root) || createStatus({ vaultPath: root })
  }

  const setStatus = (vaultRoot, patch) => {
    const root = normalizeVaultRoot(vaultRoot)
    const nextStatus = {
      ...createStatus({ vaultPath: root }),
      ...getStatus(root),
      ...patch,
      vaultPath: root
    }
    statusByVault.set(root, nextStatus)
    return nextStatus
  }

  const buildIndex = async (root, message = 'Atomic embedding search is ready.') => {
    log.info('[search] buildIndex:start', { root })
    setStatus(root, {
      status: SEARCH_STATUSES.INDEXING,
      message: 'Building local embedding index...',
      error: ''
    })
    try {
      const documents = await readAtomicDocuments(root, currentEmbeddingProvider)
      const knowledgeChunkIndex = createKnowledgeIndexFromAtomicDocuments(documents)
      const index = {
        ...createAtomicSemanticIndex(documents),
        knowledgeChunkIndex,
        embeddingSource: currentEmbeddingProvider?.source || 'deterministic-local'
      }
      indexByVault.set(root, index)
      log.info('[search] buildIndex:done', {
        root,
        documents: documents.length,
        chunks: knowledgeChunkIndex.chunks.length,
        embeddingSource: index.embeddingSource
      })
      return setStatus(root, {
        status: SEARCH_STATUSES.READY,
        indexedDocuments: documents.length,
        totalDocuments: documents.length,
        message:
          index.embeddingSource === 'node-llama-cpp'
            ? 'node-llama-cpp embedding search is ready.'
            : message,
        error: ''
      })
    } catch (error) {
      log.error('[search] buildIndex:error', { root, error })
      return setStatus(root, {
        status: SEARCH_STATUSES.ERROR,
        message: 'Embedding index failed.',
        error: error instanceof Error ? error.message : String(error || '')
      })
    }
  }

  const ensureIndex = async (root) => {
    if (indexByVault.has(root)) return indexByVault.get(root)
    await buildIndex(root)
    return indexByVault.get(root) || createAtomicSemanticIndex([])
  }

  return {
    setEmbeddingProvider(embeddingProvider = null) {
      log.info('[search] setEmbeddingProvider', {
        source: embeddingProvider?.source || '',
        enabled: Boolean(embeddingProvider?.embedText)
      })
      currentEmbeddingProvider = embeddingProvider
      indexByVault.clear()
    },

    async initForVault(vaultRoot, windowId = null) {
      const root = normalizeVaultRoot(vaultRoot)
      if (!root) return createStatus()

      if (windowId !== null) activeVaultRootByWindow.set(windowId, root)
      log.info('[search] initForVault', { root, windowId, enabled })

      if (!enabled) {
        const status = createStatus({ status: SEARCH_STATUSES.DISABLED, vaultPath: root })
        statusByVault.set(status.vaultPath, status)
        return status
      }

      return setStatus(root, {
        status: SEARCH_STATUSES.NOT_INITIALIZED,
        message: 'Atomic local search is ready. Build the index from a semantic query or rebuild action.',
        error: ''
      })
    },

    async search({ query, mode = SEARCH_MODES.SMART, limit = 20 } = {}, windowId = null) {
      if (!enabled) return []
      const root = windowId !== null ? activeVaultRootByWindow.get(windowId) : null
      if (!root) return []
      const normalizedQuery = String(query || '').trim()
      if (!normalizedQuery) return []
      log.info('[search] search', { root, windowId, mode, limit, query: normalizedQuery.slice(0, 80) })
      const hasSemanticIndex = indexByVault.has(root)
      if (mode === SEARCH_MODES.EXACT) {
        return exactSearchMarkdownFiles({
          vaultRoot: root,
          query: normalizedQuery,
          limit: Math.max(1, Math.min(50, Number(limit) || 20))
        })
      }
      if (!hasSemanticIndex && mode === SEARCH_MODES.SMART) {
        return exactSearchMarkdownFiles({
          vaultRoot: root,
          query: normalizedQuery,
          limit: Math.max(1, Math.min(50, Number(limit) || 20))
        })
      }
      const semanticIndex = hasSemanticIndex ? indexByVault.get(root) : await ensureIndex(root)
      return localSearchMarkdownFiles({
        vaultRoot: root,
        query: normalizedQuery,
        mode,
        limit: Math.max(1, Math.min(50, Number(limit) || 20)),
        semanticIndex,
        embeddingProvider: currentEmbeddingProvider
      })
    },

    async indexFile(absolutePath, windowId = null) {
      const root = absolutePath
        ? this._resolveVaultRootForPath(absolutePath, windowId)
        : ''
      if (!root) return
      log.info('[search] indexFile', { root, windowId, absolutePath })
      if (!indexByVault.has(root)) {
        log.info('[search] indexFile skipped because no semantic index exists yet', {
          root,
          windowId
        })
        return
      }
      await buildIndex(root, 'Atomic embedding search refreshed.')
    },

    async deleteFile(absolutePath, windowId = null) {
      await this.indexFile(absolutePath, windowId)
    },

    async rebuildIndex(windowId = null) {
      const root = this._resolveRootFromWindow
        ? this._resolveRootFromWindow(windowId)
        : ''
      if (!root) return createStatus()
      log.info('[search] rebuildIndex', { root, windowId })
      return buildIndex(root, 'Embedding index rebuilt with automatic semantic links.')
    },

    async clearIndex(windowId = null) {
      const root = this._resolveRootFromWindow
        ? this._resolveRootFromWindow(windowId)
        : ''
      if (!root) return createStatus()
      log.info('[search] clearIndex', { root, windowId })
      indexByVault.delete(root)
      return setStatus(root, {
        status: SEARCH_STATUSES.READY,
        indexedDocuments: 0,
        totalDocuments: 0,
        message: 'Embedding index cleared. Rebuild search to recreate semantic links.',
        error: ''
      })
    },

    async getStatus(windowId = null) {
      const root = this._resolveRootFromWindow ? this._resolveRootFromWindow(windowId) : ''
      if (!root) return createStatus()
      return getStatus(root)
    },

    async inspectIndex(windowId = null) {
      const root = this._resolveRootFromWindow ? this._resolveRootFromWindow(windowId) : ''
      const status = root ? getStatus(root) : createStatus()
      const index = root && indexByVault.has(root) ? indexByVault.get(root) : null
      const documents = (index?.documents || []).map((document) => ({
        id: document.id,
        relativePath: document.relativePath,
        title: document.title,
        chunkCount: document.chunks.length,
        sourceCount: document.sources.length,
        sources: document.sources,
        tags: document.tags
      }))
      const knowledgeChunks = (index?.knowledgeChunkIndex?.chunks || []).map((chunk) => ({
        id: chunk.id,
        documentPath: chunk.documentPath,
        relativePath: chunk.relativePath,
        chunkIndex: chunk.chunkIndex,
        headingPath: chunk.headingPath,
        textHash: chunk.textHash,
        tokenCount: chunk.tokenCount,
        wordCount: chunk.wordCount,
        lexicalTerms: chunk.lexicalTerms,
        preview: String(chunk.text || '').slice(0, 240)
      }))
      const graph = createSemanticGraph({
        documents: index?.documents || [],
        semanticLinks: index?.semanticLinks || []
      })
      const organization = buildAutomaticOrganizationPlan({ graph })
      return {
        status,
        indexPath: '',
        documents,
        chunks: knowledgeChunks,
        chunkIndex: index?.knowledgeChunkIndex
          ? {
              version: index.knowledgeChunkIndex.version,
              generatedAt: index.knowledgeChunkIndex.generatedAt,
              stats: index.knowledgeChunkIndex.stats
            }
          : null,
        folders: graph.nodes.filter((node) => node.kind === 'folder'),
        semanticLinks: index?.semanticLinks || [],
        graph,
        organization,
        features: {
          embeddings: true,
          embeddingSource: index?.embeddingSource || 'deterministic-local',
          semanticLinks: true,
          semanticClusters: graph.clusters.some((cluster) => cluster.kind === 'semantic'),
          automaticSources: true,
          autoTags: true,
          automaticOrganization: organization.proposals.length > 0,
          chunkLevelKnowledgeIndex: Boolean(index?.knowledgeChunkIndex)
        },
        generatedAt: index?.generatedAt || new Date().toISOString()
      }
    },

    disable() {
      enabled = false
      for (const [root, status] of statusByVault.entries()) {
        statusByVault.set(root, {
          ...status,
          status: SEARCH_STATUSES.DISABLED,
          message: 'Atomic local search disabled.'
        })
      }
      return createStatus({
        status: SEARCH_STATUSES.DISABLED,
        message: 'Atomic local search disabled.'
      })
    },

    enable() {
      enabled = true
      return createStatus({
        status: SEARCH_STATUSES.NOT_INITIALIZED,
        message: 'Atomic local search enabled.'
      })
    },

    async registerWindowVault(windowId, vaultRoot) {
      const root = normalizeVaultRoot(vaultRoot)
      if (!root) return createStatus()
      log.info('[search] registerWindowVault', { root, windowId })
      activeVaultRootByWindow.set(windowId, root)
      return this.initForVault(root, windowId)
    },

    _resolveRootFromWindow(windowId) {
      if (windowId === null || windowId === undefined) return ''
      return activeVaultRootByWindow.get(windowId) || ''
    },

    _resolveVaultRootForPath(absolutePath, windowId = null) {
      if (windowId !== null && activeVaultRootByWindow.has(windowId)) {
        return activeVaultRootByWindow.get(windowId)
      }
      if (!absolutePath) return ''
      for (const root of activeVaultRootByWindow.values()) {
        if (absolutePath.startsWith(root + path.sep)) return root
      }
      return ''
    }
  }
}

export { createStatus }
