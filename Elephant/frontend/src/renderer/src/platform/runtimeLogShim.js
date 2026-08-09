const levelToConsole = (level) => {
  if (level === false || level === 'silent') return null
  return level || 'info'
}

const makeLogger = () => {
  const state = {
    errorHandler: null,
    transports: {
      console: { level: 'info' },
      file: { level: 'info', sync: false, resolvePathFn: null }
    }
  }

  const emit = (method, args) => {
    const level = levelToConsole(state.transports.console.level)
    if (!level) return
    const fn = console[method] || console.log
    fn.apply(console, args)
  }

  return {
    transports: state.transports,
    errorHandler: {
      startCatching: ({ onError } = {}) => {
        state.errorHandler = onError || null
      }
    },
    initialize: () => {},
    info: (...args) => emit('info', args),
    warn: (...args) => emit('warn', args),
    error: (...args) => {
      emit('error', args)
      state.errorHandler?.(args[0] instanceof Error ? args[0] : new Error(String(args[0] || 'Error')))
    },
    debug: (...args) => emit('debug', args),
    log: (...args) => emit('log', args)
  }
}

export default makeLogger()
