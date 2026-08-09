const PREF_PREFIX = 'elephantnote:pref:'
const DATA_PREFIX = 'elephantnote:data:'
const TRANSIENT_KEYS = new Set(['typewriter', 'focus', 'sourceCode'])

const hasStorage = () => typeof window !== 'undefined' && window.localStorage

const parseStoredValue = (value) => {
  if (typeof value !== 'string') return value
  try {
    return JSON.parse(value)
  } catch {
    return value
  }
}

const readPrefEntry = (prefix, key) => {
  if (!hasStorage()) return undefined
  const raw = window.localStorage.getItem(`${prefix}${key}`)
  return raw === null ? undefined : parseStoredValue(raw)
}

const writePrefEntry = (prefix, key, value) => {
  if (!hasStorage()) return
  window.localStorage.setItem(`${prefix}${key}`, JSON.stringify(value))
}

export const hydratePortablePreferences = (state = {}) => {
  const preferences = {}
  for (const key of Object.keys(state)) {
    if (TRANSIENT_KEYS.has(key)) continue
    const value = readPrefEntry(PREF_PREFIX, key)
    if (typeof value !== 'undefined') {
      preferences[key] = value
    }
  }
  return preferences
}

export const readPortablePreference = (key) => readPrefEntry(PREF_PREFIX, key)

export const persistPortablePreference = (key, value) => {
  if (TRANSIENT_KEYS.has(key)) return
  writePrefEntry(PREF_PREFIX, key, value)
}

export const hydratePortableUserData = (state = {}) => {
  const userData = {}
  for (const key of Object.keys(state)) {
    const value = readPrefEntry(DATA_PREFIX, key)
    if (typeof value !== 'undefined') {
      userData[key] = value
    }
  }
  return userData
}

export const readPortableUserData = (key) => readPrefEntry(DATA_PREFIX, key)

export const persistPortableUserData = (key, value) => {
  writePrefEntry(DATA_PREFIX, key, value)
}

export const isPortableRuntime = () =>
  Boolean(globalThis.__MARKTEXT_RUNTIME__ || globalThis.__TAURI__)
