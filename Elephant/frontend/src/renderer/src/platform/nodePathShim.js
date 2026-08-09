const toSlash = (value = '') => String(value || '').replaceAll('\\', '/')
const trimRight = (value = '') => String(value || '').replace(/\/+$/g, '')
const drivePattern = /^[a-zA-Z]:\//

const splitPath = (value = '') => {
  const text = toSlash(value)
  const prefix = text.startsWith('//')
    ? '//'
    : drivePattern.test(text)
      ? text.slice(0, 3)
      : text.startsWith('/')
        ? '/'
        : ''
  const body = text.slice(prefix.length)
  const parts = []
  for (const part of body.split('/')) {
    if (!part || part === '.') continue
    if (part === '..') {
      if (parts.length && parts[parts.length - 1] !== '..') parts.pop()
      else if (!prefix) parts.push(part)
      continue
    }
    parts.push(part)
  }
  return { prefix, parts }
}

export const isAbsolute = (value = '') => {
  const text = toSlash(value)
  return text.startsWith('/') || text.startsWith('//') || drivePattern.test(text)
}

export const normalize = (value = '') => {
  if (!value) return '.'
  const { prefix, parts } = splitPath(value)
  if (!parts.length) return prefix || '.'
  return `${prefix}${prefix && !prefix.endsWith('/') ? '/' : ''}${parts.join('/')}`
}

export const join = (...parts) => normalize(parts.filter(Boolean).map(toSlash).join('/'))

export const resolve = (...parts) => {
  let out = ''
  for (const part of parts.filter(Boolean)) {
    const text = toSlash(part)
    out = isAbsolute(text) ? text : (out ? `${trimRight(out)}/${text}` : text)
  }
  return normalize(out || '/')
}

export const dirname = (value = '') => {
  const text = trimRight(normalize(value))
  if (text === '/' || text === '//' || /^[a-zA-Z]:\/$/.test(text)) return text
  const index = text.lastIndexOf('/')
  if (index < 0) return '.'
  if (index === 0) return '/'
  return text.slice(0, index)
}

export const basename = (value = '', ext = '') => {
  const text = trimRight(toSlash(value))
  const index = text.lastIndexOf('/')
  const base = index >= 0 ? text.slice(index + 1) : text
  return ext && base.endsWith(ext) ? base.slice(0, -ext.length) : base
}

export const extname = (value = '') => {
  const base = basename(value)
  const index = base.lastIndexOf('.')
  return index > 0 ? base.slice(index) : ''
}

export const relative = (from = '', to = '') => {
  const fromSplit = splitPath(resolve(from))
  const toSplit = splitPath(resolve(to))
  if (fromSplit.prefix !== toSplit.prefix) return normalize(to)
  const fromParts = [...fromSplit.parts]
  const toParts = [...toSplit.parts]
  while (fromParts.length && toParts.length && fromParts[0] === toParts[0]) {
    fromParts.shift()
    toParts.shift()
  }
  return [...fromParts.map(() => '..'), ...toParts].join('/') || ''
}

export default {
  sep: '/',
  delimiter: ':',
  normalize,
  join,
  resolve,
  dirname,
  basename,
  extname,
  isAbsolute,
  relative
}
