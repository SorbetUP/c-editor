import { describe, expect, it } from 'vitest'

const installMinimalPathGuard = (target) => {
  target.path = target.path || {}
  const normalize = target.path.normalize || ((value = '') => String(value || '').split('\\').join('/'))
  const join = target.path.join || ((...parts) => normalize(parts.filter(Boolean).join('/')))
  if (typeof target.path.normalize !== 'function') target.path.normalize = normalize
  if (typeof target.path.join !== 'function') target.path.join = join
  if (typeof target.path.resolve !== 'function') target.path.resolve = (...parts) => join(...parts)
  if (typeof target.path.basename !== 'function') {
    target.path.basename = (value = '') => normalize(value).split('/').filter(Boolean).at(-1) || ''
  }
  if (typeof target.path.dirname !== 'function') {
    target.path.dirname = (value = '') => {
      const normalized = normalize(value)
      const parts = normalized.split('/').filter(Boolean)
      if (parts.length <= 1) return normalized.startsWith('/') ? '/' : '.'
      return `${normalized.startsWith('/') ? '/' : ''}${parts.slice(0, -1).join('/')}`
    }
  }
  if (typeof target.path.isAbsolute !== 'function') {
    target.path.isAbsolute = (value = '') => normalize(value).startsWith('/')
  }
  if (typeof target.path.relative !== 'function') {
    target.path.relative = (from = '', to = '') => {
      const fromParts = normalize(from).split('/').filter(Boolean)
      const toParts = normalize(to).split('/').filter(Boolean)
      while (fromParts.length && toParts.length && fromParts[0] === toParts[0]) {
        fromParts.shift()
        toParts.shift()
      }
      return [...fromParts.map(() => '..'), ...toParts].join('/') || ''
    }
  }
  return target.path
}

describe('renderer path resolve parity guard', () => {
  it('adds resolve when browser path shim is incomplete', () => {
    const target = { path: { join: (...parts) => parts.join('/') } }
    const path = installMinimalPathGuard(target)
    expect(typeof path.resolve).toBe('function')
    expect(path.resolve('/tmp', 'elephantnote')).toBe('/tmp/elephantnote')
  })

  it('preserves existing valid resolve implementation', () => {
    const target = { path: { resolve: () => 'custom' } }
    const path = installMinimalPathGuard(target)
    expect(path.resolve('/tmp', 'elephantnote')).toBe('custom')
  })

  it('supports relative path computation used by saved note refresh', () => {
    const target = { path: {} }
    const path = installMinimalPathGuard(target)
    expect(path.relative('/vault', '/vault/folder/note.md')).toBe('folder/note.md')
  })
})
