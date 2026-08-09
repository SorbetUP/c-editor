import log from '@/platform/runtimeLogShim'
import RendererPaths from './node/paths'
import { createConsoleMirror } from '../../common/logging'

let exceptionLogger = (s) => console.error(s)
let consoleMirror = null

const configureLogger = () => {
  log.transports.console.level = process.env.NODE_ENV === 'development' ? 'info' : false
  exceptionLogger = log.error
  consoleMirror = createConsoleMirror({
    enabled: process.env.NODE_ENV === 'development',
    emit: (entry) => {
      window.tauri?.ipcRenderer?.send('mt::renderer-log', entry)
    }
  })

  console.log = consoleMirror.log
  console.info = consoleMirror.info
  console.warn = consoleMirror.warn
  console.error = consoleMirror.error
  console.debug = consoleMirror.debug
}

const parseUrlArgs = () => {
  const params = new URLSearchParams(window.location.search)
  const codeFontFamily = params.get('cff')
  const codeFontSize = params.get('cfs')
  const debug = params.get('debug') === '1'
  const hideScrollbar = params.get('hsb') === '1'
  const theme = params.get('theme')
  const titleBarStyle = params.get('tbs')
  const userDataPath = params.get('udp') || window.__MARKTEXT_USER_DATA_PATH__ || ''
  const rawWindowId = params.get('wid') || window.__MARKTEXT_WINDOW_ID__ || 1
  const parsedWindowId = Number(rawWindowId)
  const windowId = Number.isNaN(parsedWindowId) ? 1 : parsedWindowId
  const type = params.get('type') || window.__MARKTEXT_WINDOW_TYPE__ || 'editor'

  return {
    type,
    debug,
    userDataPath,
    windowId,
    initialState: {
      codeFontFamily,
      codeFontSize,
      hideScrollbar,
      theme,
      titleBarStyle
    }
  }
}

/**
 * Check if an error is a known non-fatal CodeMirror race condition.
 * These errors occur when clicking in the editor during rapid state changes
 * and don't affect functionality - the user can simply click again.
 *
 * @param {Error} error - The error to check
 * @returns {boolean} True if this is a suppressible CodeMirror error
 */
const isCodeMirrorRaceCondition = (error) => {
  if (!error || !error.stack) return false

  // CodeMirror internal error when line measurement data is unavailable during mouse click
  // This happens when the document state is out of sync with the display during rapid changes
  const isMapOnUndefined =
    error.message === "Cannot read properties of undefined (reading 'map')"
  const isInPrepareMeasure = error.stack.includes('prepareMeasureForLine')
  const isInCoordsChar =
    error.stack.includes('coordsChar') || error.stack.includes('posFromMouse')

  return isMapOnUndefined && isInPrepareMeasure && isInCoordsChar
}

const handleRendererError = (event) => {
  if (event.error) {
    // Suppress known non-fatal CodeMirror race conditions
    // These occur during rapid clicking/editing and don't affect functionality
    if (isCodeMirrorRaceCondition(event.error)) {
      log.warn('[renderer] suppressed non-fatal CodeMirror race condition', {
        message: event.error.message
      })
      return
    }

    const { message, name, stack } = event.error
    const copy = {
      message,
      name,
      stack
    }

    exceptionLogger(event.error)

    // Pass exception to main process exception handler to show a error dialog.
    window.tauri?.ipcRenderer?.send('mt::handle-renderer-error', copy)
  } else {
    log.error('[renderer] uncaught non-error event', event)
  }
}

const bootstrapRenderer = () => {
  // Register renderer exception handler
  window.addEventListener('error', handleRendererError)
  window.addEventListener('unhandledrejection', handleRendererError)

  const { debug, initialState, userDataPath, windowId, type } = parseUrlArgs()
  const paths = new RendererPaths(userDataPath)
  const marktext = {
    initialState,
    env: {
      debug,
      paths,
      windowId,
      type
    },
    paths
  }
  globalThis.marktext = marktext

  configureLogger()
}

export default bootstrapRenderer
