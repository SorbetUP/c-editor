export const PI_DEFAULT_BASE_URL = 'http://127.0.0.1:8317/v1'
export const PI_DEFAULT_CODEX_CLIENT_VERSION = 'elephantnote'

const asObject = (payload = {}) => (payload && typeof payload === 'object' ? payload : {})
const text = (value = '') => String(value ?? '').trim()
const trimBaseUrl = (value = PI_DEFAULT_BASE_URL) => (text(value) || PI_DEFAULT_BASE_URL).replace(/\/+$/, '')

export const piModelsUrl = ({ baseUrl = PI_DEFAULT_BASE_URL, codexClient = false, clientVersion = PI_DEFAULT_CODEX_CLIENT_VERSION } = {}) => {
  const url = `${trimBaseUrl(baseUrl)}/models`
  return codexClient ? `${url}?client_version=${encodeURIComponent(text(clientVersion) || PI_DEFAULT_CODEX_CLIENT_VERSION)}` : url
}

export const normalizePiModels = (data = {}) => {
  const records = Array.isArray(data?.data) ? data.data : []
  return records
    .map((model) => {
      const id = text(model.id)
      if (!id) return null
      return { id, name: id, provider: 'pi', source: 'openai-compatible', object: model.object || 'model', ownedBy: model.owned_by || null, raw: model }
    })
    .filter(Boolean)
}

export const normalizePiCodexModels = (data = {}) => {
  const records = Array.isArray(data?.models) ? data.models : []
  return records
    .map((model) => {
      const id = text(model.slug || model.id)
      if (!id) return null
      return {
        id,
        name: text(model.display_name || model.name || id),
        provider: 'pi',
        source: 'codex-client',
        description: text(model.description),
        contextLength: Number(model.context_window || 0) || null,
        maxContextLength: Number(model.max_context_window || 0) || null,
        visibility: model.visibility || 'show',
        raw: model
      }
    })
    .filter(Boolean)
}

const requestJson = async({ url, fetchImpl = globalThis.fetch, signal }) => {
  if (typeof fetchImpl !== 'function') {
    return { ok: false, provider: 'pi', status: 0, error: 'fetch_unavailable', message: 'Fetch API is unavailable.', data: null }
  }
  let response
  try {
    response = await fetchImpl(url, { method: 'GET', headers: { Accept: 'application/json' }, signal })
  } catch (error) {
    return { ok: false, provider: 'pi', status: 0, error: 'network_error', message: error?.message || String(error), data: null }
  }
  if (!response.ok) {
    return { ok: false, provider: 'pi', status: response.status, error: response.status === 401 || response.status === 403 ? 'authentication_required' : 'pi_error', message: response.statusText || `HTTP ${response.status}`, data: null }
  }
  try {
    return { ok: true, provider: 'pi', status: response.status, data: await response.json() }
  } catch (error) {
    return { ok: false, provider: 'pi', status: response.status, error: 'invalid_json', message: error?.message || 'PI response was not valid JSON.', data: null }
  }
}

export const listPiModels = async(options = {}) => {
  const payload = asObject(options)
  const result = await requestJson({ ...payload, url: piModelsUrl(payload) })
  if (!result.ok) return { ...result, models: [], count: 0, codexClient: false }
  const models = normalizePiModels(result.data)
  return { ...result, models, count: models.length, codexClient: false }
}

export const listPiCodexModels = async(options = {}) => {
  const payload = { ...asObject(options), codexClient: true }
  const result = await requestJson({ ...payload, url: piModelsUrl(payload) })
  if (!result.ok) return { ...result, models: [], count: 0, codexClient: true }
  const models = normalizePiCodexModels(result.data)
  return { ...result, models, count: models.length, codexClient: true }
}

export const installPiProviderBridge = (target = globalThis) => {
  const root = target?.elephantnote
  if (!root) return false
  const fetchImpl = target.fetch?.bind(target)
  const listModels = (payload = {}) => listPiModels({ ...asObject(payload), fetchImpl })
  const listCodexModels = (payload = {}) => listPiCodexModels({ ...asObject(payload), fetchImpl })

  root.ai = root.ai || {}
  root.ai.providers = { ...(root.ai.providers || {}), listModels, listPiModels: listModels, listPiCodexModels: listCodexModels }
  root.ai.pi = { ...(root.ai.pi || {}), listModels, listCodexModels }
  root.ai.codex = { ...(root.ai.codex || {}), listModels: listCodexModels }

  const previousApi = root.api || {}
  const previousDescribe = previousApi.describe?.bind(previousApi)
  const previousCall = previousApi.call?.bind(previousApi)
  root.api = {
    ...previousApi,
    describe: async() => {
      const description = previousDescribe ? await previousDescribe() : { actions: [] }
      const actions = Array.from(new Set([...(description.actions || []), 'pi.models.list', 'pi.codex.models.list', 'codex.models.list']))
      return { ...description, actions }
    },
    call: async(action, payload = {}) => {
      if (action === 'pi.models.list' || action === 'ai.providers.listModels') return { ok: true, data: await listModels(payload) }
      if (action === 'pi.codex.models.list' || action === 'codex.models.list') return { ok: true, data: await listCodexModels(payload) }
      if (!previousCall) throw new Error(`ElephantNote PI bridge does not implement API action: ${action}`)
      return previousCall(action, payload)
    }
  }
  return true
}
