const getBridge = () => globalThis.window?.elephantnote
const normalizePayload = (payload = {}) => (payload && typeof payload === 'object' ? payload : {})

const directoryListPayload = (payload = '') => {
  if (typeof payload === 'string') return payload
  const normalizedPayload = normalizePayload(payload)
  const keys = Object.keys(normalizedPayload)
  if (
    keys.length === 1 &&
    Object.prototype.hasOwnProperty.call(normalizedPayload, 'relativePath')
  ) {
    return normalizedPayload.relativePath || ''
  }
  return normalizedPayload
}

export const COMPATIBILITY_CALLS = Object.freeze({
  'vaults.get': () => getBridge()?.getVaults?.(),
  'vaults.select': () => getBridge()?.selectVault?.(),
  'vaults.setActive': ({ vaultId }) => getBridge()?.setActiveVault?.(vaultId),
  'vaults.setIcon': (payload) => getBridge()?.setVaultIcon?.(payload),
  'vaults.setName': (payload) => getBridge()?.setVaultName?.(payload),
  'vaults.remove': (payload) => getBridge()?.removeVault?.(payload),
  'directory.list': (payload = '') => getBridge()?.listDirectory?.(directoryListPayload(payload)),
  'notes.create': (payload = {}) => {
    const normalizedPayload = typeof payload === 'string' ? { relativePath: payload } : payload
    return getBridge()?.createNote?.(normalizedPayload)
  },
  'folders.create': ({ relativePath = '' }) => getBridge()?.createFolder?.({ relativePath }),
  'sidebar.attach': (payload) => getBridge()?.attachSidebarEntry?.(payload),
  'sidebar.detach': ({ relativePath }) => getBridge()?.detachSidebarEntry?.({ relativePath }),
  'entries.rename': (payload) => getBridge()?.renameEntry?.(payload),
  'entries.move': (payload) => getBridge()?.moveEntry?.(payload),
  'entries.delete': ({ relativePath }) => getBridge()?.deleteEntry?.({ relativePath }),
  'search.query': (payload) => getBridge()?.search?.query?.(payload),
  'search.status': () => getBridge()?.search?.status?.(),
  'features.get': () => getBridge()?.features?.get?.(),
  'features.set': ({ key, enabled }) => getBridge()?.features?.set?.(key, enabled),
  'atomic.catalog.get': () => getBridge()?.atomic?.getCatalog?.()
})
