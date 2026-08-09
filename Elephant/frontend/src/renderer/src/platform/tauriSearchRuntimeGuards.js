const normalizeStatus = (status = {}, vaultPath = '') => {
  const indexedDocuments = Number(
    status.indexedDocuments ??
    status.notesIndexed ??
    status.indexedNotes ??
    status.documents ??
    0
  ) || 0
  return {
    ...status,
    vaultPath: status.vaultPath || status.activeVault?.path || vaultPath || '',
    indexedDocuments,
    totalDocuments: Number(status.totalDocuments ?? status.totalNotes ?? indexedDocuments) || indexedDocuments
  }
}

export const installTauriSearchRuntimeGuards = (target = globalThis) => {
  const search = target?.elephantnote?.search
  if (!target?.__TAURI__ || !search || target.__ELEPHANT_SEARCH_RUNTIME_GUARDS__) return false
  target.__ELEPHANT_SEARCH_RUNTIME_GUARDS__ = true

  let activeVaultPath = ''
  const rememberVaultPath = (value = '') => {
    const path = String(value || '').trim()
    if (path) activeVaultPath = path
    return activeVaultPath
  }

  const originalInitVault = search.initVault?.bind(search)
  if (typeof originalInitVault === 'function') {
    search.initVault = async(vaultPath = '') => {
      rememberVaultPath(vaultPath)
      console.info('[search] bridge:initVault:start', { vaultPath: activeVaultPath })
      const result = await originalInitVault(vaultPath)
      const normalized = normalizeStatus(result, activeVaultPath)
      rememberVaultPath(normalized.vaultPath)
      console.info('[search] bridge:initVault:done', {
        vaultPath: normalized.vaultPath,
        indexedDocuments: normalized.indexedDocuments,
        status: normalized.status || ''
      })
      return normalized
    }
  }

  const originalStatus = search.status?.bind(search)
  if (typeof originalStatus === 'function') {
    search.status = async() => {
      const result = await originalStatus()
      const normalized = normalizeStatus(result, activeVaultPath)
      rememberVaultPath(normalized.vaultPath)
      console.info('[search] bridge:status:done', {
        vaultPath: normalized.vaultPath,
        indexedDocuments: normalized.indexedDocuments,
        status: normalized.status || '',
        semantic: normalized.semantic === true,
        realEmbedding: normalized.realEmbedding === true
      })
      return normalized
    }
  }

  const originalRebuild = search.rebuild?.bind(search)
  if (typeof originalRebuild === 'function') {
    search.rebuild = async() => {
      console.info('[search] bridge:rebuild:start', { vaultPath: activeVaultPath })
      const result = await originalRebuild()
      const normalized = normalizeStatus(result, activeVaultPath)
      rememberVaultPath(normalized.vaultPath)
      console.info('[search] bridge:rebuild:done', {
        vaultPath: normalized.vaultPath,
        indexedDocuments: normalized.indexedDocuments,
        provider: normalized.provider || '',
        semantic: normalized.semantic === true,
        realEmbedding: normalized.realEmbedding === true
      })
      return normalized
    }
  }

  console.info('[search] runtime-guards:installed')
  return true
}
