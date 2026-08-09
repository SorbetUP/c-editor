import log from 'electron-log'
import { validateApiPayload } from './apiSchemas'
import { ELEPHANTNOTE_API_ACTIONS, ELEPHANTNOTE_API_VERSION } from 'common/elephantnote/apiActions'
import { toPlainObject } from '../../shared/plainObject.js'
export {
  ELEPHANTNOTE_API_ACTIONS,
  ELEPHANTNOTE_API_DOMAINS,
  ELEPHANTNOTE_API_VERSION,
  listApiContracts
} from 'common/elephantnote/apiActions'

const normalizeAction = (action) => String(action || '').trim()
const shouldTraceApi = () =>
  process.env.ELEPHANTNOTE_TRACE_API === 'true' ||
  String(process.env.DEBUG || '').includes('elephantnote:api')

export const createApiResponse = ({ ok, action, data, error }) => ({
  ok,
  version: ELEPHANTNOTE_API_VERSION,
  action,
  data: ok
    ? data
    : undefined,
  error: ok
    ? undefined
    : {
      message: error?.message || String(error || 'API request failed.'),
      code: error?.code || 'ELEPHANTNOTE_API_ERROR'
    }
})

export const createElephantNoteApi = ({ handlers = {} } = {}) => {
  const registry = new Map(Object.entries(handlers))

  const describe = () => ({
    version: ELEPHANTNOTE_API_VERSION,
    actions: [...registry.keys()].sort()
  })

  registry.set(ELEPHANTNOTE_API_ACTIONS.API_DESCRIBE, async() => describe())

  const call = async(actionName, payload = {}, context = {}) => {
    const action = normalizeAction(actionName)
    const handler = registry.get(action)
    if (!handler) {
      const error = new Error(`Unknown ElephantNote API action: ${action || '(empty)'}.`)
      error.code = 'ELEPHANTNOTE_UNKNOWN_API_ACTION'
      throw error
    }
    return handler(validateApiPayload(action, payload), context)
  }

  const callEnvelope = async(actionName, payload = {}, context = {}) => {
    const action = normalizeAction(actionName)
    const traceApi = shouldTraceApi()
    if (traceApi) {
      log.info('[api] callEnvelope:start', {
        action,
        payloadType: Array.isArray(payload) ? 'array' : typeof payload
      })
    }
    try {
      const data = await call(action, payload, context)
      if (traceApi) {
        log.info('[api] callEnvelope:done', {
          action,
          dataType: Array.isArray(data) ? 'array' : typeof data
        })
      }
      return createApiResponse({
        ok: true,
        action,
        data: toPlainObject(data)
      })
    } catch (error) {
      log.error('[api] callEnvelope:error', {
        action,
        error: error instanceof Error ? error.message : String(error || '')
      })
      return createApiResponse({ ok: false, action, error })
    }
  }

  return {
    version: ELEPHANTNOTE_API_VERSION,
    describe,
    call,
    callEnvelope,
    hasAction: (actionName) => registry.has(normalizeAction(actionName)),
    listActions: () => [...registry.keys()].sort()
  }
}

export const registerElephantNoteApiIpc = ({ ipcMain, api }) => {
  ipcMain.handle('elephantnote:api:describe', async() => api.describe())
  ipcMain.handle('elephantnote:api:call', async(event, request = {}) => {
    return api.callEnvelope(request.action, request.payload || {}, {
      event,
      windowId: request.windowId || request.payload?.windowId
    })
  })
}
