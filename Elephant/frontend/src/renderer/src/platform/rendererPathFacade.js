export const ensureRendererPathFacade = (target = globalThis) => {
  const scope = target.window || target
  scope.path = scope.path || {}
  const normalize = scope.path.normalize || ((value = '') => String(value || '').split('\\').join('/'))
  const join = scope.path.join || ((...parts) => normalize(parts.filter(Boolean).join('/')))
  const pathParts = (value = '') => normalize(value).split('/').filter(Boolean)

  if (typeof scope.path.normalize !== 'function') scope.path.normalize = normalize
  if (typeof scope.path.join !== 'function') scope.path.join = join
  if (typeof scope.path.resolve !== 'function') scope.path.resolve = (...parts) => join(...parts)
  if (typeof scope.path.basename !== 'function') {
    scope.path.basename = (value = '') => pathParts(value).at(-1) || ''
  }
  if (typeof scope.path.dirname !== 'function') {
    scope.path.dirname = (value = '') => {
      const normalized = normalize(value)
      const parts = pathParts(normalized)
      if (parts.length <= 1) return normalized.startsWith('/') ? '/' : '.'
      return `${normalized.startsWith('/') ? '/' : ''}${parts.slice(0, -1).join('/')}`
    }
  }
  if (typeof scope.path.isAbsolute !== 'function') {
    scope.path.isAbsolute = (value = '') => normalize(value).startsWith('/')
  }
  if (typeof scope.path.relative !== 'function') {
    scope.path.relative = (from = '', to = '') => {
      const fromParts = pathParts(from)
      const toParts = pathParts(to)
      const sharedLength = Math.min(fromParts.length, toParts.length)
      let commonIndex = 0
      while (commonIndex < sharedLength && fromParts[commonIndex] === toParts[commonIndex]) {
        commonIndex += 1
      }
      const parentSegments = Array.from({ length: fromParts.length - commonIndex }, () => '..')
      return [...parentSegments, ...toParts.slice(commonIndex)].join('/') || ''
    }
  }

  return scope.path
}
