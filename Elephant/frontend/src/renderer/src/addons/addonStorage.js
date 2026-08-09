const STORAGE_KEY_PATTERN = /^[a-zA-Z0-9][a-zA-Z0-9._:-]{0,127}$/

const memoryStore = new Map()

const memoryBackend = {
  get length() {
    return memoryStore.size
  },
  getItem: (key) => {
    return memoryStore.has(key) ? memoryStore.get(key) : null
  },
  setItem: (key, value) => memoryStore.set(key, String(value)),
  removeItem: (key) => memoryStore.delete(key),
  key: (index) => [...memoryStore.keys()][index] || null
}

const getDefaultBackend = () => {
  if (typeof window !== 'undefined' && window.localStorage) {
    return window.localStorage
  }
  return memoryBackend
}

const normalizeStorageKey = (key) => {
  if (typeof key !== 'string' || !STORAGE_KEY_PATTERN.test(key.trim())) {
    throw new TypeError('Addon storage key must be a safe non-empty string')
  }
  return key.trim()
}

const createPrefix = (addonId) => `elephantnote:addons:${addonId}:`

const parseStoredValue = (rawValue, fallback) => {
  if (rawValue === null || rawValue === undefined) return fallback
  try {
    return JSON.parse(rawValue)
  } catch {
    return fallback
  }
}

const listBackendKeys = (backend) => {
  const keys = []
  const length = Number(backend.length) || 0
  for (let index = 0; index < length; index += 1) {
    const key = backend.key(index)
    if (typeof key === 'string') keys.push(key)
  }
  return keys
}

export const createAddonStorage = (addonId, backend = getDefaultBackend()) => {
  const prefix = createPrefix(addonId)
  const toBackendKey = (key) => `${prefix}${normalizeStorageKey(key)}`

  return Object.freeze({
    async get(key, fallback = undefined) {
      return parseStoredValue(backend.getItem(toBackendKey(key)), fallback)
    },

    async set(key, value) {
      backend.setItem(toBackendKey(key), JSON.stringify(value))
    },

    async remove(key) {
      backend.removeItem(toBackendKey(key))
    },

    async clear() {
      for (const key of listBackendKeys(backend)) {
        if (key.startsWith(prefix)) {
          backend.removeItem(key)
        }
      }
    },

    async entries() {
      const result = {}
      for (const key of listBackendKeys(backend)) {
        if (!key.startsWith(prefix)) continue
        const scopedKey = key.slice(prefix.length)
        result[scopedKey] = parseStoredValue(backend.getItem(key), undefined)
      }
      return result
    }
  })
}
