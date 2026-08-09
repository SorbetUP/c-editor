const normalizeVaultPath = (payload = '') => {
  if (typeof payload === 'string') return payload.trim()
  return String(payload?.vaultPath || payload?.path || '').trim()
}

const unavailable = (capability) => {
  throw new Error(`${capability} requires the optional Search addon.`)
}

const emptyInspection = () => ({
  indexPath: '',
  documents: [],
  folders: [],
  semanticLinks: [],
  graph: null,
  generatedAt: new Date().toISOString()
})

export const installTauriSearchLifecycleBridge = ({ target = globalThis, client = null } = {}) => {
  const bridgeSearch = target?.elephantnote?.search
  const clientSearch = client?.search
  if (!target?.__TAURI__ || !bridgeSearch || !clientSearch) return false

  const readStatus = typeof clientSearch.status === 'function'
    ? clientSearch.status.bind(clientSearch)
    : typeof bridgeSearch.status === 'function'
      ? bridgeSearch.status.bind(bridgeSearch)
      : async() => ({ enabled: true, runtime: 'tauri-rust' })
  let activeVaultPath = ''

  const initVault = async(payload = '') => {
    activeVaultPath = normalizeVaultPath(payload) || activeVaultPath
    const status = await readStatus()
    return {
      ...status,
      status: status?.status || 'ready',
      vaultPath: status?.vaultPath || status?.activeVault?.path || activeVaultPath,
      indexedDocuments: Number(status?.indexedDocuments || 0),
      totalDocuments: Number(status?.totalDocuments || status?.indexedDocuments || 0)
    }
  }

  const lifecycle = {
    initVault,
    inspect: async() => {
      const info = target?.console?.info || console.info
      info.call(target?.console || console, '[tauri-search] inspect:unavailable', { reason: 'optional-search-addon-missing' })
      return emptyInspection()
    },
    rebuild: () => initVault(activeVaultPath),
    clear: () => unavailable('Clearing the semantic index'),
    disable: () => unavailable('Disabling semantic search'),
    enable: () => unavailable('Enabling semantic search')
  }

  // These methods are runtime compatibility only. The domain-client factory
  // remains minimal, so Search/Knowledge implementation ownership stays with
  // optional addons while the Tauri shell cannot crash on a missing method.
  Object.assign(bridgeSearch, lifecycle)
  Object.assign(clientSearch, lifecycle)

  return true
}
