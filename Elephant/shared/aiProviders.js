export const ELEPHANTNOTE_AI_PRESETS = Object.freeze({
  custom: {
    id: 'custom',
    label: 'Custom HTTP',
    transport: 'openai-compatible',
    endpoint: '',
    model: ''
  },
  tauriRustLocal: {
    id: 'tauriRustLocal',
    label: 'Tauri Rust local',
    transport: 'tauri-rust',
    endpoint: 'tauri-rust://local',
    model: 'hf:bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M'
  },
  nodeLlamaCpp: {
    id: 'nodeLlamaCpp',
    label: 'node-llama-cpp (compatibility alias)',
    transport: 'tauri-rust',
    endpoint: 'tauri-rust://local',
    model: 'hf:bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M'
  },
  mlx: {
    id: 'mlx',
    label: 'MLX local server',
    transport: 'openai-compatible',
    endpoint: 'http://127.0.0.1:8080/v1/chat/completions',
    model: 'mlx-community/Qwen2.5-7B-Instruct-4bit'
  },
  lmstudio: {
    id: 'lmstudio',
    label: 'LM Studio',
    transport: 'openai-compatible',
    endpoint: 'http://127.0.0.1:1234/v1/chat/completions',
    model: 'local-model'
  },
  openrouter: {
    id: 'openrouter',
    label: 'OpenRouter',
    transport: 'openai-compatible',
    endpoint: 'https://openrouter.ai/api/v1/chat/completions',
    model: 'anthropic/claude-3.5-sonnet'
  },
  codex: {
    id: 'codex',
    label: 'Codex',
    transport: 'openai-compatible',
    endpoint: 'http://127.0.0.1:1455/v1/chat/completions',
    model: 'codex-local'
  }
})

const LOCALHOST_PATTERN = /^(localhost|127(?:\.\d{1,3}){3}|0\.0\.0\.0|\d{1,3}(?:\.\d{1,3}){3}|\[[^\]]+\]):\d+(?:\/.*)?$/i

const LOCAL_APP_SOURCE_IDS = new Set(['app-local', 'node-llama-cpp', 'nodeLlamaCpp', 'tauri-rust', 'tauriRustLocal'])
const HTTP_COMPATIBLE_SOURCE_IDS = new Set([
  'api',
  'openai-compatible',
  'openrouter',
  'mistral',
  'lmstudio',
  'llamacpp',
  'llama.cpp',
  'llamacpp-server'
])

export const normalizeAiEndpoint = (endpoint = '') => {
  const value = String(endpoint || '').trim()
  if (!value) return ''
  if (/^https?:\/\//i.test(value)) return value
  if (LOCALHOST_PATTERN.test(value)) return `http://${value}`
  return value
}

export const normalizeAiEndpointForTransport = (endpoint = '', transport = 'openai-compatible') => {
  const normalized = normalizeAiEndpoint(endpoint)
  if (transport === 'browser') return normalized || 'browser://local'
  if (transport !== 'ollama' || !normalized) return normalized
  const value = normalized.replace(/\/+$/, '')
  if (/\/api\/(chat|generate)$/i.test(value)) return value.replace(/\/api\/generate$/i, '/api/chat')
  return `${value}/api/chat`
}

export const resolveAiEndpoint = ({ endpoint = '', transport = 'openai-compatible' } = {}) => {
  return normalizeAiEndpointForTransport(endpoint, transport)
}

export const createDefaultLocalAiConfig = () => ({
  enabled: false,
  showModelLibraryInSidebar: false,
  allowHuggingFaceDownloads: false,
  allowLocalRuntimeAutostart: false
})

export const normalizeLocalAiConfig = (localAi = {}) => {
  const defaults = createDefaultLocalAiConfig()
  const input = localAi && typeof localAi === 'object' ? localAi : {}
  return {
    ...defaults,
    ...input,
    enabled: input.enabled === true,
    showModelLibraryInSidebar: input.showModelLibraryInSidebar === true,
    allowHuggingFaceDownloads: input.allowHuggingFaceDownloads === true,
    allowLocalRuntimeAutostart: input.allowLocalRuntimeAutostart === true
  }
}

const getObject = (value) => (value && typeof value === 'object' && !Array.isArray(value) ? value : {})

const normalizeSourceId = (value = '') => String(value || '').trim()

const sourceToTransport = (source = '') => {
  const value = normalizeSourceId(source)
  if (LOCAL_APP_SOURCE_IDS.has(value)) return 'tauri-rust'
  if (value === 'ollama') return 'ollama'
  if (HTTP_COMPATIBLE_SOURCE_IDS.has(value)) return 'openai-compatible'
  if (value === 'codex') return 'openai-compatible'
  if (value === 'browser') return 'browser'
  return value || 'openai-compatible'
}

const providerMatchesSource = (provider = {}, source = '') => {
  const normalizedSource = normalizeSourceId(source)
  const type = normalizeSourceId(provider.type)
  if (!normalizedSource || !type) return false
  if (normalizedSource === type) return true
  if (normalizedSource === 'api' && type === 'openai-compatible') return true
  if (normalizedSource === 'llamacpp' && ['llama.cpp', 'llamacpp-server'].includes(type)) return true
  return false
}

const resolveProviderForSource = (providers = {}, source = '') => {
  const list = Array.isArray(providers.list) ? providers.list : []
  const fromList = list.find((provider) => provider.enabled !== false && providerMatchesSource(provider, source))
  if (fromList) return fromList
  const normalizedSource = source === 'api' ? 'api' : normalizeSourceId(source)
  return getObject(providers[normalizedSource])
}

const normalizeRoute = (route = {}) => getObject(route)

const resolveFeatureRouteRuntime = ({ config = {}, localAi, preset } = {}) => {
  const routes = getObject(config.routes)
  const chatRoute = normalizeRoute(routes.chat)
  const source = normalizeSourceId(chatRoute.source || chatRoute.provider)
  const hasUsableRoute = source && source !== 'disabled'
  if (!hasUsableRoute) return null

  if (LOCAL_APP_SOURCE_IDS.has(source) && localAi.enabled === false) return null

  const providers = getObject(config.providers)
  const provider = resolveProviderForSource(providers, source)
  const transport = sourceToTransport(source)
  const endpoint = chatRoute.endpoint || provider.endpoint || preset.endpoint || ''
  const model = String(chatRoute.model || config.model || preset.model || '').trim()
  const apiKey = String(provider.apiKey || config.apiKey || '').trim()

  return {
    provider: source,
    transport,
    endpoint,
    model,
    apiKey
  }
}

export const normalizeAiConfig = (config = {}) => {
  const { enabled: _compatibilityEnabled, ...restConfig } = getObject(config)
  const rawLocalAi = getObject(restConfig.localAi)
  const localAi = normalizeLocalAiConfig(rawLocalAi)
  const rawProvider = String(restConfig.preset || restConfig.provider || 'custom')
  const compatibilityPresetId = rawProvider === 'nodeLlamaCpp' ? 'tauriRustLocal' : rawProvider
  const localPresetBlocked = LOCAL_APP_SOURCE_IDS.has(compatibilityPresetId) && localAi.enabled === false
  const presetId = rawProvider === 'disabled' || localPresetBlocked ? 'custom' : compatibilityPresetId
  const preset = ELEPHANTNOTE_AI_PRESETS[presetId] || ELEPHANTNOTE_AI_PRESETS.custom
  const routeRuntime = resolveFeatureRouteRuntime({ config: restConfig, localAi, preset })
  const configuredTransport = String(routeRuntime?.transport || restConfig.transport || preset.transport)
  const localTransportBlocked = !routeRuntime && localAi.enabled === false && configuredTransport === 'tauri-rust'
  const rawTransport = localTransportBlocked ? 'disabled' : configuredTransport
  const transport = rawTransport === 'disabled' ? 'openai-compatible' : rawTransport
  const endpoint = localTransportBlocked
    ? ''
    : normalizeAiEndpointForTransport(routeRuntime?.endpoint || restConfig.endpoint || preset.endpoint, transport)
  const model = localTransportBlocked ? '' : String(routeRuntime?.model || restConfig.model || preset.model || '')
  const apiKey = String(routeRuntime?.apiKey || restConfig.apiKey || '')
  const provider = routeRuntime?.provider ||
    (localPresetBlocked || localTransportBlocked || restConfig.provider === 'disabled' ? 'disabled' : restConfig.provider)

  return {
    ...restConfig,
    preset: preset.id,
    name: String(restConfig.name || preset.label),
    provider,
    transport,
    endpoint,
    model,
    apiKey,
    codexLinkEnabled: restConfig.codexLinkEnabled !== false,
    defaultProvider: String(restConfig.defaultProvider || (localAi.enabled ? 'app-local' : 'api')),
    localAi,
    providers: getObject(restConfig.providers),
    routes: getObject(restConfig.routes),
    rag: getObject(restConfig.rag),
    tools: getObject(restConfig.tools),
    search: getObject(restConfig.search),
    indexing: getObject(restConfig.indexing),
    ocr: getObject(restConfig.ocr)
  }
}

export const createAiRequestBody = ({ transport, model, messages }) => {
  if (transport === 'ollama') {
    return {
      model,
      messages,
      stream: false
    }
  }
  return {
    model,
    messages,
    stream: false
  }
}

export const extractAiResponseText = (data = {}) => {
  if (typeof data?.message?.content === 'string') return data.message.content
  const content = data?.choices?.[0]?.message?.content
  if (typeof content === 'string') return content
  if (typeof data?.response === 'string') return data.response
  return ''
}
