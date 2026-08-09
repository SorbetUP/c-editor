export const OPENROUTER_MODELS_URL = 'https://openrouter.ai/api/v1/models'
export const OPENROUTER_CHAT_URL = 'https://openrouter.ai/api/v1/chat/completions'

const normalizePayload = (payload = {}) => (payload && typeof payload === 'object' ? payload : {})
const text = (value = '') => String(value ?? '').trim()

export const normalizeOpenRouterModels = (data = {}) => {
  const records = Array.isArray(data?.data) ? data.data : Array.isArray(data) ? data : []
  return records
    .map((model) => {
      const id = text(model.id || model.slug || model.name)
      if (!id) return null
      return {
        id,
        name: text(model.name || model.id),
        provider: 'openrouter',
        contextLength: Number(model.context_length || model.contextLength || 0) || null,
        pricing: model.pricing || null,
        architecture: model.architecture || null,
        topProvider: model.top_provider || model.topProvider || null,
        supportedParameters: Array.isArray(model.supported_parameters) ? model.supported_parameters : []
      }
    })
    .filter(Boolean)
}

export const buildOpenRouterHeaders = ({ apiKey = '', appTitle = 'ElephantNote', referer = '' } = {}) => {
  const headers = {
    Accept: 'application/json',
    'Content-Type': 'application/json',
    'X-OpenRouter-Title': appTitle
  }
  if (text(apiKey)) headers.Authorization = `Bearer ${text(apiKey)}`
  if (text(referer)) headers['HTTP-Referer'] = text(referer)
  return headers
}

const parseError = async(response) => {
  try {
    const data = await response.json()
    return data?.error?.message || data?.message || JSON.stringify(data)
  } catch {
    return response.statusText || `HTTP ${response.status}`
  }
}

export const listOpenRouterModels = async(options = {}) => {
  const {
    apiKey = '',
    fetchImpl = globalThis.fetch,
    signal,
    appTitle = 'ElephantNote',
    referer = ''
  } = normalizePayload(options)

  if (typeof fetchImpl !== 'function') {
    return {
      ok: false,
      provider: 'openrouter',
      status: 0,
      authenticated: Boolean(text(apiKey)),
      models: [],
      error: 'fetch_unavailable',
      message: 'Fetch API is unavailable in this runtime.'
    }
  }

  let response
  try {
    response = await fetchImpl(OPENROUTER_MODELS_URL, {
      method: 'GET',
      headers: buildOpenRouterHeaders({ apiKey, appTitle, referer }),
      signal
    })
  } catch (error) {
    return {
      ok: false,
      provider: 'openrouter',
      status: 0,
      authenticated: Boolean(text(apiKey)),
      models: [],
      error: 'network_error',
      message: error?.message || String(error)
    }
  }

  if (!response.ok) {
    const message = await parseError(response)
    const authError = response.status === 401 || response.status === 403
    return {
      ok: false,
      provider: 'openrouter',
      status: response.status,
      authenticated: Boolean(text(apiKey)),
      models: [],
      error: authError ? 'authentication_required' : 'openrouter_error',
      message
    }
  }

  const data = await response.json()
  const models = normalizeOpenRouterModels(data)
  return {
    ok: true,
    provider: 'openrouter',
    status: response.status,
    authenticated: Boolean(text(apiKey)),
    models,
    count: models.length
  }
}

export const testOpenRouterChatAccess = async(options = {}) => {
  const { apiKey = '', fetchImpl = globalThis.fetch, appTitle = 'ElephantNote', referer = '' } = normalizePayload(options)
  if (!text(apiKey)) {
    return {
      ok: false,
      provider: 'openrouter',
      endpoint: 'chat/completions',
      error: 'authentication_required',
      message: 'OpenRouter chat/completions requires an API key.'
    }
  }
  if (typeof fetchImpl !== 'function') {
    return { ok: false, provider: 'openrouter', error: 'fetch_unavailable', message: 'Fetch API is unavailable.' }
  }
  const response = await fetchImpl(OPENROUTER_CHAT_URL, {
    method: 'POST',
    headers: buildOpenRouterHeaders({ apiKey, appTitle, referer }),
    body: JSON.stringify({ model: 'openrouter/auto', messages: [{ role: 'user', content: 'ping' }], max_tokens: 1 })
  })
  if (!response.ok) {
    return { ok: false, provider: 'openrouter', endpoint: 'chat/completions', status: response.status, error: 'openrouter_error', message: await parseError(response) }
  }
  return { ok: true, provider: 'openrouter', endpoint: 'chat/completions', status: response.status }
}

export const listCodexProviderModels = async(options = {}) => {
  const provider = text(options.provider || options.providerId || 'openrouter').toLowerCase()
  if (provider === 'openrouter') return listOpenRouterModels(options)
  return {
    ok: false,
    provider,
    models: [],
    error: 'provider_not_supported',
    message: `Provider ${provider} is not supported by the Codex Provider Interface yet.`
  }
}

export const installCodexProviderBridge = (target = globalThis) => {
  const root = target?.elephantnote
  if (!root) return false

  const listModels = (payload = {}) => listCodexProviderModels({ ...normalizePayload(payload), fetchImpl: target.fetch?.bind(target) })
  const testProvider = (payload = {}) => testOpenRouterChatAccess({ ...normalizePayload(payload), fetchImpl: target.fetch?.bind(target) })

  root.ai = root.ai || {}
  root.ai.providers = {
    ...(root.ai.providers || {}),
    listModels,
    listOpenRouterModels: (payload = {}) => listOpenRouterModels({ ...normalizePayload(payload), fetchImpl: target.fetch?.bind(target) }),
    testOpenRouter: testProvider
  }
  root.ai.codex = {
    ...(root.ai.codex || {}),
    listModels,
    testProvider
  }

  const previousApi = root.api || {}
  const previousDescribe = previousApi.describe?.bind(previousApi)
  const previousCall = previousApi.call?.bind(previousApi)
  root.api = {
    ...previousApi,
    describe: async() => {
      const description = previousDescribe ? await previousDescribe() : { actions: [] }
      const actions = Array.from(new Set([...(description.actions || []), 'ai.providers.listModels', 'codex.models.list', 'codex.providers.test']))
      return { ...description, actions }
    },
    call: async(action, payload = {}) => {
      if (action === 'ai.providers.listModels' || action === 'codex.models.list') {
        return { ok: true, data: await listModels(payload) }
      }
      if (action === 'codex.providers.test') {
        return { ok: true, data: await testProvider(payload) }
      }
      if (!previousCall) throw new Error(`ElephantNote provider bridge does not implement API action: ${action}`)
      return previousCall(action, payload)
    }
  }
  return true
}
