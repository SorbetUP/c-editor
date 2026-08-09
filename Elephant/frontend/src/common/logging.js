const DEFAULT_LOG_LEVELS = new Set(['debug', 'info', 'warn', 'error'])

const serializeLogValue = (value) => {
  if (value == null) return String(value)
  if (typeof value === 'string') return value
  if (typeof value === 'number' || typeof value === 'boolean' || typeof value === 'bigint') {
    return String(value)
  }
  if (typeof value === 'function') {
    return `[Function ${value.name || 'anonymous'}]`
  }
  if (value instanceof Error) {
    return value.stack || `${value.name}: ${value.message}`
  }
  if (value instanceof Date) {
    return value.toISOString()
  }
  try {
    return JSON.stringify(value)
  } catch {
    return Object.prototype.toString.call(value)
  }
}

const formatLogArgs = (args = []) => args.map(serializeLogValue).join(' ')

const createConsoleMirror = ({ emit, enabled = true } = {}) => {
  const originalConsole = {
    log: console.log.bind(console),
    info: console.info.bind(console),
    warn: console.warn.bind(console),
    error: console.error.bind(console),
    debug: console.debug.bind(console)
  }

  const mirror = (level, args) => {
    const safeLevel = DEFAULT_LOG_LEVELS.has(level) ? level : 'info'
    if (enabled) {
      try {
        emit?.({
          level: safeLevel,
          message: formatLogArgs(args),
          args: args.map(serializeLogValue),
          timestamp: new Date().toISOString()
        })
      } catch {
        // Never let logging break the app.
      }
    }

    const target = originalConsole[safeLevel] || originalConsole.info
    target(...args)
  }

  return {
    log: (...args) => mirror('info', args),
    info: (...args) => mirror('info', args),
    warn: (...args) => mirror('warn', args),
    error: (...args) => mirror('error', args),
    debug: (...args) => mirror('debug', args),
    originalConsole,
    formatLogArgs,
    serializeLogValue
  }
}

export { createConsoleMirror, formatLogArgs, serializeLogValue }
