const MAX_LOGS = 1000

const DIAGNOSTIC_VERBOSE_STORAGE_KEY = 'elephantnote:diagnostics:verbose'

let consoleForwardingInstalled = false
let forwardingConsole = false

export const isDiagnosticVerbose = () => {
  const target = globalThis.window || globalThis
  return Boolean(
    target.__ELEPHANT_VERBOSE_DIAGNOSTICS__ ||
    target.localStorage?.getItem?.(DIAGNOSTIC_VERBOSE_STORAGE_KEY) === 'true' ||
    target.location?.search?.includes('elephantDiagnostics=1')
  )
}

const shouldEmitDiagnostic = (level) => level === 'error' || level === 'warn' || isDiagnosticVerbose()

const toErrorObject = (error) => ({
  name: error?.name || 'Error',
  message: error?.message || String(error || ''),
  stack: error?.stack || ''
})

const normalizeDetails = (details) => {
  if (details instanceof Error) return toErrorObject(details)
  if (typeof details === 'string' && details.length > 4000) return `${details.slice(0, 4000)}…`
  return details
}

const safeJson = (value) => {
  try {
    return JSON.stringify(value)
  } catch {
    return String(value)
  }
}

const normalizeConsoleArgs = (args = []) => {
  const [first, ...rest] = args
  const message = typeof first === 'string' ? first : safeJson(normalizeDetails(first))
  const details = rest.length === 0
    ? null
    : rest.map((item) => normalizeDetails(item))
  return { message, details }
}

const isNoisyElementPlusWarning = (entry) => {
  if (entry.level !== 'warn') return false
  const text = `${entry.message || ''} ${safeJson(entry.details || '')}`
  return text.includes('[el-dialog] [API] the title slot is about to be deprecated')
}

const forwardToTauriTerminal = (entry) => {
  if (isNoisyElementPlusWarning(entry) && !isDiagnosticVerbose()) return
  const invoke = globalThis.window?.__TAURI__?.core?.invoke
  if (!invoke) return
  try {
    void invoke('tauri_debug_log', {
      level: entry.level,
      message: entry.message,
      details: entry.details || null
    }).catch(() => {})
  } catch {}
}

const installConsoleForwarding = () => {
  const targetConsole = globalThis.console
  if (consoleForwardingInstalled || !targetConsole) return
  consoleForwardingInstalled = true
  for (const level of ['log', 'info', 'warn', 'error', 'debug']) {
    const original = targetConsole[level]?.bind(targetConsole)
    if (typeof original !== 'function') continue
    targetConsole[level] = (...args) => {
      original(...args)
      if (forwardingConsole) return
      forwardingConsole = true
      try {
        const normalized = normalizeConsoleArgs(args)
        forwardToTauriTerminal({
          time: new Date().toISOString(),
          level: level === 'log' ? 'info' : level,
          message: normalized.message,
          details: normalized.details
        })
      } finally {
        forwardingConsole = false
      }
    }
  }
}

export const pushDiagnosticLog = (level, message, details = null) => {
  const target = globalThis.window || globalThis
  const entry = {
    time: new Date().toISOString(),
    level,
    message,
    details: normalizeDetails(details)
  }
  target.__ELEPHANT_DEBUG_LOGS__ = target.__ELEPHANT_DEBUG_LOGS__ || []
  target.__ELEPHANT_DEBUG_LOGS__.push(entry)
  if (target.__ELEPHANT_DEBUG_LOGS__.length > MAX_LOGS) {
    target.__ELEPHANT_DEBUG_LOGS__.splice(0, target.__ELEPHANT_DEBUG_LOGS__.length - MAX_LOGS)
  }
  if (shouldEmitDiagnostic(level)) {
    const method = level === 'error' ? 'error' : level === 'warn' ? 'warn' : 'log'
    forwardingConsole = true
    try {
      console[method](`[elephant:${level}] ${message}`, entry.details || '')
    } finally {
      forwardingConsole = false
    }
  }
  forwardToTauriTerminal(entry)
  return entry
}

export const showDiagnosticOverlay = (title, error) => {
  const doc = globalThis.document
  if (!doc?.body) return
  let node = doc.getElementById('elephant-diagnostic-overlay')
  if (!node) {
    node = doc.createElement('pre')
    node.id = 'elephant-diagnostic-overlay'
    node.style.position = 'fixed'
    node.style.inset = '16px'
    node.style.zIndex = '2147483647'
    node.style.background = '#111'
    node.style.color = '#fff'
    node.style.border = '1px solid #ff5c5c'
    node.style.borderRadius = '12px'
    node.style.padding = '16px'
    node.style.overflow = 'auto'
    doc.body.appendChild(node)
  }
  const data = toErrorObject(error)
  node.textContent = `${title}\n\n${data.message}\n\n${data.stack}\n\nRead window.__ELEPHANT_DEBUG_LOGS__ for recent logs.`
}

export const installRendererDiagnostics = (app = null) => {
  installConsoleForwarding()
  pushDiagnosticLog('info', 'renderer diagnostics installed')

  globalThis.window?.addEventListener?.('error', (event) => {
    pushDiagnosticLog('error', 'window error', event.error || event.message)
    showDiagnosticOverlay('Renderer window error', event.error || event.message)
  })

  globalThis.window?.addEventListener?.('unhandledrejection', (event) => {
    pushDiagnosticLog('error', 'unhandled promise rejection', event.reason)
    showDiagnosticOverlay('Unhandled promise rejection', event.reason)
  })

  if (app?.config) {
    app.config.errorHandler = (error, instance, info) => {
      pushDiagnosticLog('error', `vue error: ${info}`, error)
      showDiagnosticOverlay(`Vue error: ${info}`, error)
      throw error
    }
    app.config.warnHandler = (message, instance, trace) => {
      pushDiagnosticLog('warn', `vue warning: ${message}`, { trace })
    }
  }
}

export const getDiagnosticLogs = () => (globalThis.window || globalThis).__ELEPHANT_DEBUG_LOGS__ || []
