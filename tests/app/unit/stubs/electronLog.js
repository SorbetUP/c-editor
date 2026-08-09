const noop = () => {}

const log = {
  error: noop,
  warn: noop,
  info: noop,
  debug: noop,
  verbose: noop,
  silly: noop,
  initialize: noop,
  transports: {
    console: { level: false, writeFn: noop },
    file: { level: false, sync: false, resolvePathFn: noop }
  },
  errorHandler: { startCatching: noop }
}

export default log
