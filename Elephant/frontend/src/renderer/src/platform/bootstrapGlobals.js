import { installGlobalMuyaRuntimeBridge } from '../muya/globalRuntimeBridge.js'

const WINDOWS_SEPARATOR = String.fromCharCode(92)
const normalizeSlashes = (value) => String(value || '').split(WINDOWS_SEPARATOR).join('/')

const collapseForwardSlashes = (value = '') => {
  let next = String(value || '')
  while (next.includes('//')) next = next.replaceAll('//', '/')
  return next
}

const trimTrailingForwardSlashes = (value = '') => {
  let next = String(value || '')
  while (next.length > 1 && next.endsWith('/')) next = next.slice(0, -1)
  return next
}

const isWindowsDrivePath = (value = '') => {
  const text = String(value || '')
  const first = text.charCodeAt(0)
  const isLetter = (first >= 65 && first <= 90) || (first >= 97 && first <= 122)
  return isLetter && text[1] === ':' && text[2] === '/'
}

const resolveSegments = (body = '', absolute = false) => {
  const resolved = []
  for (const segment of String(body || '').split('/')) {
    if (!segment || segment === '.') continue
    if (segment === '..') {
      if (resolved.length && resolved[resolved.length - 1] !== '..') {
        resolved.pop()
      } else if (!absolute) {
        resolved.push('..')
      }
      continue
    }
    resolved.push(segment)
  }
  return resolved
}

const createPathFacade = () => {
  const join = (...parts) => normalize(joinedPath(parts))
  const normalize = (pathname) => {
    const value = normalizeSlashes(pathname)
    if (!value) return '.'
    const isDrive = isWindowsDrivePath(value)
    const prefix = value.startsWith('//') ? '//' : isDrive ? value.slice(0, 3) : value.startsWith('/') ? '/' : ''
    const absolute = !!prefix
    const body = trimTrailingForwardSlashes(collapseForwardSlashes(value.slice(prefix.length)))
    const segments = resolveSegments(body, absolute)
    if (!segments.length) return prefix || '.'
    if (!prefix) return segments.join('/')
    const cleanPrefix = prefix.endsWith('/') ? prefix.slice(0, -1) : prefix
    return collapseForwardSlashes(`${cleanPrefix}/${segments.join('/')}`)
  }
  const dirname = (pathname) => {
    const normalized = normalize(pathname)
    if (normalized === '/' || isWindowsDrivePath(normalized)) return normalized
    const index = normalized.lastIndexOf('/')
    return index > 0 ? normalized.slice(0, index) : normalized.startsWith('/') ? '/' : '.'
  }
  const basename = (pathname, ext = '') => {
    const value = trimTrailingForwardSlashes(normalize(pathname))
    const index = value.lastIndexOf('/')
    const base = index >= 0 ? value.slice(index + 1) : value
    return ext && base.endsWith(ext) ? base.slice(0, -ext.length) : base
  }
  const extname = (pathname) => {
    const base = basename(pathname)
    const index = base.lastIndexOf('.')
    return index > 0 ? base.slice(index) : ''
  }
  const resolve = (...parts) => normalize(joinedPath(parts, true))
  const relative = (from, to) => {
    const fromParts = normalize(from).split('/').filter(Boolean)
    const toParts = normalize(to).split('/').filter(Boolean)
    while (fromParts.length && toParts.length && fromParts[0] === toParts[0]) {
      fromParts.shift()
      toParts.shift()
    }
    return [...fromParts.map(() => '..'), ...toParts].join('/') || ''
  }
  const joinWithAbsolute = (parts, resolveMode = false) => {
    const filtered = parts.filter((part) => part !== undefined && part !== null && part !== '')
    if (!filtered.length) return resolveMode ? '/' : '.'
    return normalize(filtered.map((part) => normalizeSlashes(part)).join('/'))
  }
  function joinedPath(parts, resolveMode = false) {
    return joinWithAbsolute(parts, resolveMode)
  }
  return {
    sep: '/',
    delimiter: ':',
    normalize,
    join,
    resolve,
    dirname,
    basename,
    extname,
    isAbsolute: (pathname) => {
      const value = normalizeSlashes(pathname)
      return value.startsWith('/') || isWindowsDrivePath(value) || value.startsWith('//')
    },
    relative
  }
}

const createFileUtilsFallback = () => ({
  __elephantnoteBootstrapFallback: true,
  isFile: () => false,
  isDirectory: () => false,
  emptyDir: async() => {},
  copy: async() => {},
  ensureDir: async() => {},
  outputFile: async() => {},
  move: async() => {},
  stat: async() => ({}),
  writeFile: async() => {},
  readFile: async() => '',
  ensureDirSync: () => {},
  pathExistsSync: () => false,
  isChildOfDirectory: () => false,
  hasMarkdownExtension: (filename) => String(filename || '').toLowerCase().endsWith('.md'),
  MARKDOWN_INCLUSIONS: ['.md', '.markdown', '.mdown', '.mkdn', '.mkd', '.mdtxt'],
  isSamePathSync: (pathA, pathB) => normalizeSlashes(pathA) === normalizeSlashes(pathB),
  isImageFile: (filepath) => {
    const value = String(filepath || '').toLowerCase()
    return ['.png', '.jpg', '.jpeg', '.gif', '.webp', '.svg', '.bmp', '.ico'].some((ext) => value.endsWith(ext))
  }
})

const installBootstrapGlobals = (target = globalThis) => {
  if (!target.path) target.path = createPathFacade()
  if (target.fileUtils?.__elephantnoteBootstrapFallback && target.__TAURI__) delete target.fileUtils
  if (!target.fileUtils && !target.__TAURI__) target.fileUtils = createFileUtilsFallback()
  if (!target.rgPath) target.rgPath = ''
  if (!target.global) target.global = target
  installGlobalMuyaRuntimeBridge(target)
}

installBootstrapGlobals()

export default installBootstrapGlobals
