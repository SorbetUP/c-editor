export const ELEPHANTNOTE_ASSETS_DIR = '.assets'

const splitPath = (pathname = '') => {
  const normalized = String(pathname || '').replace(/\\/g, '/')
  const slashIndex = normalized.lastIndexOf('/')
  return {
    directory: slashIndex >= 0 ? normalized.slice(0, slashIndex) : '',
    filename: slashIndex >= 0 ? normalized.slice(slashIndex + 1) : normalized
  }
}

export const joinPath = (...parts) => parts
  .filter((part) => part !== undefined && part !== null && String(part).length > 0)
  .join('/')
  .replace(/\/+/g, '/')

export const sanitizeAssetName = (value = '', fallback = 'asset') => {
  const cleaned = String(value || '')
    .replace(/\\/g, '/')
    .split('/')
    .filter(Boolean)
    .pop() || fallback
  const safe = [...cleaned]
    .map((character) => {
      if ('<>:"|?*'.includes(character) || character.charCodeAt(0) < 32) return '-'
      return character
    })
    .join('')
    .replace(/^\.+/, '')
    .trim()
  return safe || fallback
}

export const getVaultAssetRelativePath = (fileName = 'asset') => joinPath(
  ELEPHANTNOTE_ASSETS_DIR,
  sanitizeAssetName(fileName, 'asset')
)

export const isHiddenAssetPath = (pathname = '') => String(pathname || '')
  .replace(/\\/g, '/')
  .split('/')
  .includes(ELEPHANTNOTE_ASSETS_DIR)

export const getPathDirectory = (pathname = '') => splitPath(pathname).directory
