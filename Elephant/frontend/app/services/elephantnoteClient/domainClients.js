import { ELEPHANTNOTE_API_ACTIONS as API } from 'common/elephantnote/apiActions'

const getBridge = () => globalThis.window?.elephantnote

const callSearchBridge = (method, payload) => {
  const fn = getBridge()?.search?.[method]
  if (typeof fn !== 'function') {
    throw new Error(`Elephant search.${method} is unavailable in this runtime.`)
  }
  return payload === undefined ? fn() : fn(payload)
}

const directoryListPayload = (payload = '') =>
  typeof payload === 'string' ? { relativePath: payload } : payload

const normalizeRelativePath = (value = '') => String(value || '')
  .replace(/\\/g, '/')
  .split('/')
  .filter((part) => part && part !== '.' && part !== '..')
  .join('/')

const parentRelativePath = (value = '') => {
  const parts = normalizeRelativePath(value).split('/').filter(Boolean)
  return parts.length > 1 ? parts.slice(0, -1).join('/') : ''
}

const noteCreatePayload = (payload = '') =>
  typeof payload === 'string'
    ? { relativePath: normalizeRelativePath(payload) }
    : { ...(payload || {}), relativePath: normalizeRelativePath(payload?.relativePath || '') }

const folderCreateRequest = (payload = '') => {
  if (typeof payload !== 'string') {
    const request = { ...(payload || {}) }
    const requestedPath = normalizeRelativePath(request.relativePath || request.path || '')
    return {
      request: { ...request, relativePath: requestedPath || 'New Folder' },
      parentPath: parentRelativePath(requestedPath)
    }
  }
  const parentPath = normalizeRelativePath(payload)
  return {
    request: { relativePath: [parentPath, 'New Folder'].filter(Boolean).join('/') },
    parentPath
  }
}

const normalizeCreatedNote = async (call, request, result) => {
  if (result?.note && Array.isArray(result?.entries)) return result
  if (!result?.path) throw new Error('The note backend did not return the created note path.')
  const entries = await call(API.DIRECTORY_LIST, {
    relativePath: normalizeRelativePath(request.relativePath || '')
  })
  return { note: result, entries: Array.isArray(entries) ? entries : [] }
}

const normalizeCreatedFolder = async (call, parentPath, result) => {
  if (result?.folder && Array.isArray(result?.entries)) return result
  if (!result?.path) throw new Error('The folder backend did not return the created folder path.')
  const entries = await call(API.DIRECTORY_LIST, { relativePath: parentPath })
  return { folder: result, entries: Array.isArray(entries) ? entries : [] }
}

export const createDomainClients = (call, requireAtomicFeatureApi) => ({
  vaults: {
    get: () => call(API.VAULTS_GET),
    select: () => call(API.VAULTS_SELECT),
    createLocal: () => globalThis.window?.elephantnote?.createLocalVault?.() || call(API.VAULTS_SELECT),
    setActive: (vaultId) => call(API.VAULTS_SET_ACTIVE, { vaultId }),
    setIcon: (vaultId, icon) => call(API.VAULTS_SET_ICON, { vaultId, icon }),
    setName: (vaultId, name) => call(API.VAULTS_SET_NAME, { vaultId, name }),
    remove: (vaultId) => call(API.VAULTS_REMOVE, { vaultId })
  },
  directory: {
    list: (payload = '') => call(API.DIRECTORY_LIST, directoryListPayload(payload))
  },
  notes: {
    create: async (payload = '') => {
      const request = noteCreatePayload(payload)
      const result = await call(API.NOTES_CREATE, request)
      return normalizeCreatedNote(call, request, result)
    },
    read: (relativePath) =>
      call(API.NOTES_READ, typeof relativePath === 'string' ? { relativePath } : relativePath),
    write: (payload = {}) => call(API.NOTES_WRITE, payload)
  },
  folders: {
    create: async (payload = '') => {
      const { request, parentPath } = folderCreateRequest(payload)
      const result = await call(API.FOLDERS_CREATE, request)
      return normalizeCreatedFolder(call, parentPath, result)
    }
  },
  sidebar: {
    attach: (payload) => call(API.SIDEBAR_ATTACH, payload),
    detach: (relativePath) => call(API.SIDEBAR_DETACH, { relativePath })
  },
  entries: {
    rename: (payload) => call(API.ENTRIES_RENAME, payload),
    move: (payload) => call(API.ENTRIES_MOVE, payload),
    delete: (relativePath) => call(API.ENTRIES_DELETE, { relativePath })
  },
  search: {
    query: (params) => call(API.SEARCH_QUERY, params),
    concepts: (params = {}) => callSearchBridge('concepts', params),
    status: () => call(API.SEARCH_STATUS)
  },
  features: {
    get: () => call(API.FEATURES_GET),
    set: (key, enabled) => call(API.FEATURES_SET, { key, enabled })
  },
  atomic: {
    getCatalog: () => call(API.ATOMIC_CATALOG_GET)
  },
  atomicFeatures: {
    describeApi: () => requireAtomicFeatureApi().describeApi(),
    callApi: (action, args = {}) => requireAtomicFeatureApi().callApi({ action, arguments: args }),
    providers: () => requireAtomicFeatureApi().providers()
  }
})
