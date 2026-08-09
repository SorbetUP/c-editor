const normalizeSlashes = (value) => String(value || '').replace(/\\/g, '/')

const createPathFacade = () => {
  const normalize = (pathname) => {
    const value = normalizeSlashes(pathname)
    if (!value) return '.'
    const isDrive = /^[a-zA-Z]:\//.test(value)
    const prefix = value.startsWith('//') ? '//' : isDrive ? value.slice(0, 3) : value.startsWith('/') ? '/' : ''
    const body = value.slice(prefix.length).replace(/\/+/g, '/').replace(/\/+$/, '')
    if (!body) return prefix || '.'
    return prefix ? `${prefix.replace(/\/$/, '')}/${body}`.replace(/\/+/g, '/') : body
  }
  const join = (...parts) => normalize(parts.filter((part) => part !== undefined && part !== null && part !== '').join('/'))
  const dirname = (pathname) => {
    const normalized = normalize(pathname)
    if (normalized === '/' || /^[a-zA-Z]:\/$/.test(normalized)) return normalized
    const index = normalized.lastIndexOf('/')
    return index > 0 ? normalized.slice(0, index) : '.'
  }
  const basename = (pathname, ext = '') => {
    const value = normalizeSlashes(pathname).replace(/\/+$/, '')
    const index = value.lastIndexOf('/')
    const base = index >= 0 ? value.slice(index + 1) : value
    return ext && base.endsWith(ext) ? base.slice(0, -ext.length) : base
  }
  const extname = (pathname) => {
    const base = basename(pathname)
    const index = base.lastIndexOf('.')
    return index > 0 ? base.slice(index) : ''
  }
  const resolve = (...parts) => normalize(parts.filter((part) => part !== undefined && part !== null && part !== '').join('/'))
  const relative = (from, to) => {
    const fromParts = normalize(from).split('/').filter(Boolean)
    const toParts = normalize(to).split('/').filter(Boolean)
    while (fromParts.length && toParts.length && fromParts[0] === toParts[0]) {
      fromParts.shift()
      toParts.shift()
    }
    return [...fromParts.map(() => '..'), ...toParts].join('/') || ''
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
      return value.startsWith('/') || /^([a-zA-Z]:\/)/.test(value) || value.startsWith('//')
    },
    relative
  }
}

class EnvPaths {
  /**
   * @param {string} userDataPath The user data path.
   * @returns
   */
  constructor(userDataPath) {
    const currentDate = new Date()
    if (!userDataPath) {
      throw new Error('"userDataPath" is not set.')
    }

    const path = globalThis.path || createPathFacade()

    this._electronUserDataPath = userDataPath // path.join(userDataPath, 'electronUserData')
    this._userDataPath = userDataPath
    this._logPath = path.join(
      this._userDataPath,
      'logs',
      `${currentDate.getFullYear()}${currentDate.getMonth() + 1}`
    )
    this._preferencesPath = userDataPath // path.join(this._userDataPath, 'preferences')
    this._editorBufferStorePath = path.join(this._userDataPath, 'editorStates')

    this._dataCenterPath = userDataPath

    this._preferencesFilePath = path.join(this._preferencesPath, 'preference.json')

    // TODO(sessions): enable this...
    // this._globalStorage = path.join(this._userDataPath, 'globalStorage')
    // this._preferencesPath = path.join(this._userDataPath, 'preferences')
    // this._sessionsPath = path.join(this._userDataPath, 'sessions')
  }

  get electronUserDataPath() {
    // This path is identical to app.getPath('userData') but userDataPath must not necessarily be the same path.
    return this._electronUserDataPath
  }

  get userDataPath() {
    return this._userDataPath
  }

  get logPath() {
    return this._logPath
  }

  get preferencesPath() {
    return this._preferencesPath
  }

  get dataCenterPath() {
    return this._dataCenterPath
  }

  get preferencesFilePath() {
    return this._preferencesFilePath
  }

  get editorBufferStorePath() {
    return this._editorBufferStorePath
  }
}

export default EnvPaths
