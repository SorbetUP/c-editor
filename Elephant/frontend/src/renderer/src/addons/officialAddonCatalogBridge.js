import { useAddonsStore } from '@/store/addons'

const PATCH_FLAG = '__elephantOfficialCatalogBridgeInstalled'
const isOfficialId = (id = '') => String(id).startsWith('elephant.')

const invoke = (command, payload = {}) => {
  const caller = globalThis?.__TAURI__?.core?.invoke
  if (typeof caller !== 'function') throw new Error(`Tauri command API is unavailable for ${command}`)
  return caller(command, payload)
}

const asCatalogEntries = (value) => Array.isArray(value) ? value : []

const mergeById = (...collections) => {
  const entries = new Map()
  for (const collection of collections) {
    for (const entry of asCatalogEntries(collection)) {
      if (entry?.id) entries.set(entry.id, entry)
    }
  }
  return [...entries.values()]
}

export const installOfficialAddonCatalogBridge = () => {
  const store = useAddonsStore()
  if (store[PATCH_FLAG]) return false
  store[PATCH_FLAG] = true

  const loadCommunityCatalog = store.loadAddonCatalog.bind(store)
  store.loadAddonCatalog = async () => {
    store.catalogLoading = true
    const [communityResult, officialResult] = await Promise.allSettled([
      loadCommunityCatalog(),
      invoke('tauri_official_addons_catalog_list')
    ])

    const community = communityResult.status === 'fulfilled'
      ? asCatalogEntries(communityResult.value)
      : []
    const official = officialResult.status === 'fulfilled'
      ? asCatalogEntries(officialResult.value).map((entry) => ({
        ...entry,
        official: true,
        source: 'official'
      }))
      : []

    store.catalog = mergeById(community, official)
    if (store.catalog.length) {
      store.catalogError = null
    } else {
      const errors = [communityResult, officialResult]
        .filter((result) => result.status === 'rejected')
        .map((result) => result.reason?.message || String(result.reason))
      store.catalogError = errors.join('\n') || 'No addon catalogue is available.'
    }
    store.catalogLoading = false
    return store.catalog
  }

  const installCommunityCatalogAddon = store.installCatalogAddon.bind(store)
  store.installCatalogAddon = async (id) => {
    if (!isOfficialId(id)) return installCommunityCatalogAddon(id)
    if (!store.manager?.external) throw new Error('External addon runtime is not available')
    store.operationInProgress = true
    try {
      const record = await invoke('tauri_official_addons_catalog_install', { addonId: id })
      const existing = store.manager.get(record.manifest.id)
      if (existing) {
        await store.manager.disable(record.manifest.id).catch(() => {})
        store.manager.unregister(record.manifest.id)
      }
      store.manager.external.register({ ...record, source: 'official' })
      await store.loadTrustedState().catch(() => false)
      store.lastError = null
      return record
    } catch (error) {
      store.lastError = error?.message || String(error)
      throw error
    } finally {
      store.operationInProgress = false
      store.refresh()
    }
  }

  return true
}

if (typeof window !== 'undefined') {
  window.addEventListener('elephantnote:addons-ready', () => {
    try {
      installOfficialAddonCatalogBridge()
    } catch (error) {
      console.error('[addons] unable to install official catalogue bridge', error)
    }
  })
}
