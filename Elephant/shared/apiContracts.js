export const ELEPHANTNOTE_API_VERSION = '2026-07-14-core'

const isPlainObject = (value) =>
  value !== null && typeof value === 'object' && !Array.isArray(value)

const optionalString = (value) => value === undefined || typeof value === 'string'
const requiredString = (value) => typeof value === 'string' && value.trim().length > 0
const textString = (value) => typeof value === 'string'
const optionalBoolean = (value) => value === undefined || typeof value === 'boolean'
const optionalNumber = (value) => value === undefined || Number.isFinite(Number(value))
const optionalObject = (value) => value === undefined || isPlainObject(value)
const optionalEnum = (allowed) => (value) => value === undefined || allowed.includes(value)
const requiredEnum = (allowed) => (value) => allowed.includes(value)

const isSafeLeafFilename = (value) => {
  if (value === undefined) return true
  if (typeof value !== 'string') return false
  const filename = value.trim()
  if (!filename) return true
  if (filename === '.' || filename === '..') return false
  if (filename.includes('\0')) return false
  if (/[\\/]/.test(filename)) return false
  if (filename.split(/[\\/]/).some((part) => part === '..')) return false
  return true
}

const optionalSafeFilename = (value) => isSafeLeafFilename(value)

const assertObject = (payload, action) => {
  if (!isPlainObject(payload)) {
    const error = new Error(`Invalid payload for ${action}: expected an object.`)
    error.code = 'ELEPHANTNOTE_INVALID_API_PAYLOAD'
    throw error
  }
}

const assertField = (payload, field, validator, action) => {
  if (!validator(payload[field])) {
    const error = new Error(`Invalid payload for ${action}: field "${field}" is invalid.`)
    error.code = 'ELEPHANTNOTE_INVALID_API_PAYLOAD'
    throw error
  }
}

export const schema = Object.freeze({
  empty: (payload, action) => {
    assertObject(payload, action)
    return payload
  },
  object:
    (fields = {}) =>
    (payload, action) => {
      assertObject(payload, action)
      for (const [field, validator] of Object.entries(fields)) {
        assertField(payload, field, validator, action)
      }
      return payload
    },
  strictObject:
    (fields = {}) =>
    (payload, action) => {
      assertObject(payload, action)
      for (const field of Object.keys(payload)) {
        if (!Object.prototype.hasOwnProperty.call(fields, field)) {
          const error = new Error(
            `Invalid payload for ${action}: field "${field}" is not supported.`
          )
          error.code = 'ELEPHANTNOTE_INVALID_API_PAYLOAD'
          throw error
        }
      }
      for (const [field, validator] of Object.entries(fields)) {
        assertField(payload, field, validator, action)
      }
      return payload
    },
  optionalString,
  requiredString,
  textString,
  optionalBoolean,
  optionalNumber,
  optionalObject,
  optionalEnum,
  requiredEnum,
  optionalSafeFilename
})

const action = (key, name, payload = schema.empty) => ({ key, name, payload })

export const ELEPHANTNOTE_API_DOMAINS = Object.freeze({
  system: Object.freeze([
    action('API_DESCRIBE', 'api.describe')
  ]),
  vaults: Object.freeze([
    action('VAULTS_GET', 'vaults.get'),
    action('VAULTS_SELECT', 'vaults.select'),
    action('VAULTS_SET_ACTIVE', 'vaults.setActive', schema.object({ vaultId: requiredString })),
    action(
      'VAULTS_SET_ICON',
      'vaults.setIcon',
      schema.object({ vaultId: requiredString, icon: optionalString })
    ),
    action(
      'VAULTS_SET_NAME',
      'vaults.setName',
      schema.object({ vaultId: requiredString, name: requiredString })
    ),
    action('VAULTS_REMOVE', 'vaults.remove', schema.object({ vaultId: requiredString }))
  ]),
  documents: Object.freeze([
    action(
      'DIRECTORY_LIST',
      'directory.list',
      schema.object({
        relativePath: optionalString,
        offset: optionalNumber,
        limit: optionalNumber,
        includePreview: optionalBoolean
      })
    ),
    action(
      'NOTES_CREATE',
      'notes.create',
      schema.object({
        relativePath: optionalString,
        filename: optionalSafeFilename,
        title: optionalString
      })
    ),
    action('NOTES_READ', 'notes.read', schema.object({ relativePath: requiredString })),
    action(
      'NOTES_WRITE',
      'notes.write',
      schema.object({ relativePath: requiredString, markdown: textString })
    ),
    action('FOLDERS_CREATE', 'folders.create', schema.object({ relativePath: optionalString })),
    action(
      'SIDEBAR_ATTACH',
      'sidebar.attach',
      schema.object({
        relativePath: requiredString,
        title: optionalString,
        type: optionalEnum(['note', 'folder'])
      })
    ),
    action('SIDEBAR_DETACH', 'sidebar.detach', schema.object({ relativePath: requiredString })),
    action(
      'ENTRIES_RENAME',
      'entries.rename',
      schema.object({ relativePath: requiredString, title: requiredString })
    ),
    action(
      'ENTRIES_MOVE',
      'entries.move',
      schema.object({ relativePath: requiredString, targetDirectoryPath: optionalString })
    ),
    action('ENTRIES_DELETE', 'entries.delete', schema.object({ relativePath: requiredString }))
  ]),
  search: Object.freeze([
    action(
      'SEARCH_QUERY',
      'search.query',
      schema.object({
        query: requiredString,
        mode: optionalEnum(['exact', 'text']),
        limit: optionalNumber
      })
    ),
    action('SEARCH_STATUS', 'search.status')
  ]),
  coreFeatures: Object.freeze([
    action('FEATURES_GET', 'features.get'),
    action(
      'FEATURES_SET',
      'features.set',
      schema.object({ key: requiredString, enabled: optionalBoolean })
    ),
    action('ATOMIC_CATALOG_GET', 'atomic.catalog.get')
  ])
})

export const listApiContracts = () => Object.values(ELEPHANTNOTE_API_DOMAINS).flat()

export const ELEPHANTNOTE_API_ACTIONS = Object.freeze(
  Object.fromEntries(listApiContracts().map(({ key, name }) => [key, name]))
)

export const API_PAYLOAD_SCHEMAS = Object.freeze(
  Object.fromEntries(listApiContracts().map(({ name, payload }) => [name, payload]))
)

export const validateApiPayload = (actionName, payload = {}) => {
  const validator = API_PAYLOAD_SCHEMAS[actionName]
  if (!validator) return payload
  return validator(payload, actionName)
}
