import { BrowserWindow, ipcMain } from 'electron'
import log from 'electron-log'
import { createDefaultModelSelection, ATOMIC_MODEL_CATALOG } from 'common/elephantnote/atomicWorkspace'
import { createTextEmbedding } from 'common/elephantnote/atomicAiEngine'
import { createConceptProfile } from 'common/elephantnote/knowledge/knowledgeIndex'
import { rankConceptsForQuery } from 'common/elephantnote/knowledge/conceptRouter'
import { getConfig } from '../config/elephantConfigStore'
import { modelRuntime } from '../runtime/elephantRuntime'
import { ElephantSearchService } from './ElephantSearchService'
import { SEARCH_MODES } from './searchTypes'

const searchService = new ElephantSearchService()
let atomicIpcImportPromise = null
let embeddingProviderRegistered = false
let warnedMissingEmbeddingModel = false

const INSPECT_CACHE_TTL_MS = 5000
const inspectCacheByWindow = new Map()
const inspectPromiseByWindow = new Map()

const getInspectCacheKey = (windowId) => windowId ?? 'default'

const clearInspectCache = (windowId = null) => {
  if (windowId === null || windowId === undefined) {
    inspectCacheByWindow.clear()
    inspectPromiseByWindow.clear()
    return
  }

  const key = getInspectCacheKey(windowId)
  inspectCacheByWindow.delete(key)
  inspectPromiseByWindow.delete(key)
}

const inspectWithCache = async(windowId) => {
  const key = getInspectCacheKey(windowId)
  const now = Date.now()
  const cached = inspectCacheByWindow.get(key)

  if (cached && now - cached.createdAt < INSPECT_CACHE_TTL_MS) {
    log.info('[search] inspect:cache-hit', { windowId })
    return cached.value
  }

  if (inspectPromiseByWindow.has(key)) {
    log.info('[search] inspect:dedupe', { windowId })
    return inspectPromiseByWindow.get(key)
  }

  const promise = searchService.inspectIndex(windowId)
    .then((value) => {
      inspectCacheByWindow.set(key, {
        createdAt: Date.now(),
        value
      })
      return value
    })
    .finally(() => {
      inspectPromiseByWindow.delete(key)
    })

  inspectPromiseByWindow.set(key, promise)
  return promise
}

const ensureAtomicIpc = () => {
  if (!atomicIpcImportPromise) {
    atomicIpcImportPromise = import('../atomic/atomicIpc').catch((error) => {
      atomicIpcImportPromise = null
      log.warn('Unable to register Atomic IPC:', error)
    })
  }
  return atomicIpcImportPromise
}

const getSelectedEmbeddingModelId = () => {
  const selection = {
    ...createDefaultModelSelection(),
    ...(getConfig().atomicModelSelection || {})
  }
  return String(selection.embedding || '').trim()
}

const getActiveVaultPath = () => {
  const config = getConfig()
  const vault = config.vaults.find((item) => item.id === config.activeVaultId)
  if (!vault?.path) throw new Error('No active ElephantNote vault.')
  return vault.path
}

const resolveEmbeddingCatalogEntry = (modelId = '') => (
  ATOMIC_MODEL_CATALOG.find((item) =>
    item.id === modelId ||
    item.model === modelId ||
    item.uri === modelId ||
    item.pull === modelId
  ) || {
    id: modelId,
    name: modelId,
    provider: 'node-llama-cpp',
    task: 'embedding',
    model: modelId,
    uri: modelId,
    pull: modelId
  }
)

const createNodeLlamaCppEmbeddingProvider = () => ({
  get source() {
    return getSelectedEmbeddingModelId() ? 'node-llama-cpp' : 'deterministic-local'
  },

  async embedText(text = '') {
    const selectedModelId = getSelectedEmbeddingModelId()
    const preview = String(text || '').replace(/\s+/g, ' ').slice(0, 120)

    if (!selectedModelId) {
      if (!warnedMissingEmbeddingModel) {
        warnedMissingEmbeddingModel = true
        log.warn('[embedding] no embedding model selected; using deterministic local fallback')
      }
      return createTextEmbedding(text)
    }

    const startedAt = Date.now()
    const configuredModel = resolveEmbeddingCatalogEntry(selectedModelId)
    let resolvedModel = configuredModel

    try {
      resolvedModel = await modelRuntime.modelLibrary.resolveLocalModel(configuredModel)
    } catch (error) {
      log.warn('[embedding] selected model is not resolved locally; attempting configured model fallback', {
        selectedModelId,
        error: error instanceof Error ? error.message : String(error || '')
      })
    }

    log.info('[embedding] embed:start', {
      selectedModelId,
      model: resolvedModel?.path || resolvedModel?.modelPath || resolvedModel?.model || configuredModel.model || selectedModelId,
      textLength: String(text || '').length,
      preview
    })

    const vector = await modelRuntime.nodeLlamaCppRuntime.embedText({
      model: {
        ...configuredModel,
        ...resolvedModel,
        task: 'embedding',
        purpose: 'embedding'
      },
      text
    })

    log.info('[embedding] embed:done', {
      selectedModelId,
      dimensions: Array.isArray(vector) ? vector.length : 0,
      latencyMs: Date.now() - startedAt
    })

    return vector
  }
})

const ensureEmbeddingProviderRegistered = () => {
  if (embeddingProviderRegistered) return
  embeddingProviderRegistered = true
  const provider = createNodeLlamaCppEmbeddingProvider()
  searchService.setEmbeddingProvider(provider)
  log.info('[embedding] search embedding provider registered', {
    source: provider.source,
    selectedModelId: getSelectedEmbeddingModelId() || ''
  })
}

export const normalizeSearchMode = (mode) => {
  if (
    mode === SEARCH_MODES.EXACT ||
    mode === SEARCH_MODES.SEMANTIC ||
    mode === SEARCH_MODES.SMART
  ) {
    return mode
  }
  throw new Error('Invalid search mode.')
}

export const clampSearchLimit = (limit) => {
  const parsed = Number(limit)
  if (!Number.isFinite(parsed)) return 20
  return Math.max(1, Math.min(50, Math.trunc(parsed)))
}

export const normalizeSearchQuery = (params = {}) => {
  if (typeof params !== 'object' || params === null) {
    throw new Error('Invalid search payload.')
  }

  if (typeof params.query !== 'string') {
    throw new Error('Query must be a string.')
  }

  return {
    query: params.query,
    mode: normalizeSearchMode(params.mode),
    limit: clampSearchLimit(params.limit)
  }
}

export const normalizeConceptQuery = (params = {}) => {
  if (typeof params !== 'object' || params === null) {
    throw new Error('Invalid concept search payload.')
  }

  if (typeof params.query !== 'string') {
    throw new Error('Query must be a string.')
  }

  return {
    query: params.query,
    limit: Math.max(1, Math.min(12, Number(params.limit) || 5)),
    evidenceLimit: Math.max(1, Math.min(8, Number(params.evidenceLimit) || 5))
  }
}

const createConceptProfilesFromInspection = (inspection = {}) => {
  const documentConcepts = (inspection.documents || []).map((document) => createConceptProfile({
    id: `document:${document.relativePath}`,
    title: document.title || document.relativePath,
    aliases: [document.relativePath, ...(document.tags || [])],
    positiveTerms: [document.title, document.relativePath, ...(document.tags || []), ...(document.sources || []).map((source) => source.title || source.url)].flat(),
    negativeTerms: []
  }))

  const clusterConcepts = (inspection.graph?.clusters || []).map((cluster) => createConceptProfile({
    id: `cluster:${cluster.id || cluster.label}`,
    title: cluster.label || cluster.id || 'Cluster',
    aliases: [...(cluster.tags || []), ...(cluster.paths || [])],
    positiveTerms: [cluster.label, ...(cluster.tags || []), ...(cluster.paths || [])],
    negativeTerms: []
  }))

  const byId = new Map()
  for (const concept of [...documentConcepts, ...clusterConcepts]) {
    if (!concept.id || byId.has(concept.id)) continue
    byId.set(concept.id, concept)
  }
  return [...byId.values()]
}

const routeConcepts = async (payload, windowId) => {
  let inspection = await inspectWithCache(windowId)
  if (!inspection?.chunks?.length && !inspection?.documents?.length) {
    await searchService.rebuildIndex(windowId)
    clearInspectCache(windowId)
    inspection = await inspectWithCache(windowId)
  }

  return rankConceptsForQuery({
    query: payload.query,
    concepts: createConceptProfilesFromInspection(inspection),
    chunks: inspection?.chunks || [],
    limit: payload.limit,
    evidenceLimit: payload.evidenceLimit
  })
}

const getSenderWindowId = (event) => {
  const win = BrowserWindow.fromWebContents(event.sender)
  return win?.id ?? null
}

export const registerSearchIpc = () => {
  log.info('[search] registering IPC handlers')
  ensureAtomicIpc()
  ensureEmbeddingProviderRegistered()

  ipcMain.handle('en:search:init-vault', async(event) => {
    const vaultPath = getActiveVaultPath()
    const windowId = getSenderWindowId(event)
    clearInspectCache(windowId)
    log.info('[search] init-vault', { windowId, vaultPath })
    return searchService.registerWindowVault(windowId, vaultPath)
  })

  ipcMain.handle('en:search:query', async(event, params) => {
    const payload = normalizeSearchQuery(params)
    log.info('[search] query', {
      windowId: getSenderWindowId(event),
      mode: payload.mode,
      limit: payload.limit,
      query: payload.query.slice(0, 80)
    })
    return searchService.search(payload, getSenderWindowId(event))
  })

  ipcMain.handle('en:search:concepts', async(event, params) => {
    const payload = normalizeConceptQuery(params)
    const windowId = getSenderWindowId(event)
    log.info('[search] concepts', {
      windowId,
      limit: payload.limit,
      query: payload.query.slice(0, 80)
    })
    return routeConcepts(payload, windowId)
  })

  ipcMain.handle('en:search:status', async(event) => {
    log.info('[search] status', { windowId: getSenderWindowId(event) })
    return searchService.getStatus(getSenderWindowId(event))
  })

  ipcMain.handle('en:search:inspect', async(event) => {
    const windowId = getSenderWindowId(event)
    log.info('[search] inspect', { windowId })
    return inspectWithCache(windowId)
  })

  ipcMain.handle('en:search:rebuild', async(event) => {
    const windowId = getSenderWindowId(event)
    clearInspectCache(windowId)
    log.info('[search] rebuild', { windowId })
    const status = await searchService.rebuildIndex(windowId)
    clearInspectCache(windowId)
    return status
  })

  ipcMain.handle('en:search:clear', async(event) => {
    const windowId = getSenderWindowId(event)
    clearInspectCache(windowId)
    log.info('[search] clear', { windowId })
    return searchService.clearIndex(windowId)
  })

  ipcMain.handle('en:search:disable', async() => {
    clearInspectCache()
    log.info('[search] disable')
    return searchService.disable()
  })

  ipcMain.handle('en:search:enable', async() => {
    clearInspectCache()
    log.info('[search] enable')
    return searchService.enable()
  })
}

export const getSearchService = () => searchService
