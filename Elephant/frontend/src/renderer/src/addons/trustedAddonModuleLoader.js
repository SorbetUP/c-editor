const MAX_MODULE_DEPTH = 64
const STATIC_IMPORT_PATTERN = /(^|[;\n])(\s*(?:import|export)\s+(?:[^'";]+?\s+from\s+)?)(['"])([^'"]+)\3/gm

const normalizeModulePath = (value, { requireExtension = true } = {}) => {
  const parts = []
  for (const part of String(value || '').replaceAll('\\', '/').split('/')) {
    if (!part || part === '.') continue
    if (part === '..') {
      if (!parts.length) throw new Error(`Trusted addon module escapes its package: ${value}`)
      parts.pop()
      continue
    }
    parts.push(part)
    if (parts.length > MAX_MODULE_DEPTH) throw new Error('Trusted addon module graph is too deep')
  }
  let normalized = parts.join('/')
  if (!normalized) throw new Error(`Trusted addon module path is empty: ${value}`)
  if (!normalized.split('/').at(-1).includes('.')) normalized += '.js'
  if (requireExtension && !normalized.endsWith('.js')) {
    throw new Error(`Trusted addon module must be JavaScript: ${value}`)
  }
  return normalized
}

const resolveModuleSpecifier = (parentPath, specifier) => {
  const value = String(specifier || '')
  if (!value.startsWith('.')) {
    throw new Error(`Trusted addon modules cannot import external dependency: ${value}`)
  }
  const parent = normalizeModulePath(parentPath).split('/')
  parent.pop()
  return normalizeModulePath([...parent, ...value.split('/')].join('/'))
}

const staticDependencies = (source) => {
  const dependencies = []
  for (const match of String(source || '').matchAll(STATIC_IMPORT_PATTERN)) {
    if (!dependencies.includes(match[4])) dependencies.push(match[4])
  }
  return dependencies
}

export const loadTrustedAddonModuleGraph = async ({ addonId, entryPath, readModule }) => {
  if (typeof readModule !== 'function') throw new TypeError('readModule must be a function')
  const urls = []
  const loaded = new Map()
  const loading = new Set()

  const load = async (modulePath, depth = 0) => {
    const normalized = normalizeModulePath(modulePath)
    if (loaded.has(normalized)) return loaded.get(normalized)
    if (depth > MAX_MODULE_DEPTH) throw new Error('Trusted addon module graph exceeds the depth limit')
    if (loading.has(normalized)) {
      throw new Error(`Cyclic trusted addon module graph is not supported: ${normalized}`)
    }

    loading.add(normalized)
    try {
      const payload = await readModule(normalized)
      let source = typeof payload?.source === 'string' ? payload.source : ''
      if (!source) throw new Error(`Trusted addon module is empty: ${normalized}`)
      const dependencyUrls = new Map()
      for (const specifier of staticDependencies(source)) {
        const dependencyPath = resolveModuleSpecifier(normalized, specifier)
        dependencyUrls.set(specifier, await load(dependencyPath, depth + 1))
      }
      source = source.replace(STATIC_IMPORT_PATTERN, (full, boundary, statement, quote, specifier) => {
        const dependencyUrl = dependencyUrls.get(specifier)
        return dependencyUrl ? `${boundary}${statement}${quote}${dependencyUrl}${quote}` : full
      })
      const blob = new Blob([
        source,
        `\n//# sourceURL=elephant-addon://${addonId}/${normalized}`
      ], { type: 'text/javascript' })
      const url = URL.createObjectURL(blob)
      loaded.set(normalized, url)
      urls.push(url)
      return url
    } finally {
      loading.delete(normalized)
    }
  }

  try {
    const entryUrl = await load(normalizeModulePath(entryPath || 'main.js'))
    return { entryUrl, urls }
  } catch (error) {
    for (const url of urls) URL.revokeObjectURL(url)
    throw error
  }
}

export const revokeTrustedAddonModuleGraph = (urls = []) => {
  for (const url of urls) URL.revokeObjectURL(url)
}

export const trustedAddonModulePathForTest = normalizeModulePath
