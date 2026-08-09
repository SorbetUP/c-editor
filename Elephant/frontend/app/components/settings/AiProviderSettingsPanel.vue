<template>
  <section class="en-ai-settings">
    <section v-if="activePage === 'chat'" class="en-ai-card">
      <header class="en-ai-card-header">
        <div><h4>Chat route</h4><p>Select the runtime used by Elephant chat.</p></div>
        <span class="en-ai-badge" :class="{ active: form.routes.chat.source !== 'disabled' }">{{ routeProviderLabel('chat') }}</span>
      </header>
      <div class="en-ai-card-body en-ai-grid">
        <label>
          <span>Provider</span>
          <select v-model="form.routes.chat.source" @change="onRouteSourceChanged('chat')">
            <option value="disabled">Disabled</option>
            <option v-for="provider in chatProviderOptions" :key="provider.key" :value="provider.source">{{ provider.label }}</option>
          </select>
        </label>
        <label>
          <span>Model</span>
          <select v-if="selectedChatAddonProvider && selectedChatModels.length" v-model="form.routes.chat.model">
            <option value="">Select a model</option>
            <option v-for="model in selectedChatModels" :key="addonModelId(model)" :value="addonModelId(model)">{{ addonModelLabel(model) }}</option>
          </select>
          <input v-else v-model.trim="form.routes.chat.model" type="text" :disabled="form.routes.chat.source === 'disabled'" placeholder="Provider model id">
        </label>
        <label class="wide"><span>System prompt</span><textarea v-model="form.routes.chat.systemPrompt" rows="5" /></label>
      </div>
      <div class="en-ai-setting-row">
        <div class="en-ai-setting-copy">
          <strong>Retrieval-augmented answers</strong>
          <span>Include relevant Elephant notes in the request context.</span>
        </div>
        <button class="en-ai-switch" type="button" role="switch" :aria-checked="form.routes.chat.enableRag" :class="{ active: form.routes.chat.enableRag }" @click="form.routes.chat.enableRag = !form.routes.chat.enableRag"><span /></button>
      </div>
      <details class="en-ai-advanced">
        <summary>Advanced generation settings</summary>
        <div class="en-ai-card-body en-ai-grid">
          <label><span>Temperature</span><input v-model.number="form.routes.chat.temperature" type="number" min="0" max="2" step="0.05"></label>
          <label><span>Max tokens</span><input v-model.number="form.routes.chat.maxTokens" type="number" min="1" step="128"></label>
          <label><span>Context window</span><input v-model.number="form.routes.chat.contextWindow" type="number" min="512" step="512"></label>
          <label><span>RAG notes limit</span><input v-model.number="form.routes.chat.ragTopK" type="number" min="1" max="50"></label>
        </div>
      </details>
    </section>

    <section v-else-if="activePage === 'embedding'" class="en-ai-card">
      <header class="en-ai-card-header">
        <div><h4>Search and embeddings</h4><p>Configure the embedding route explicitly; no provider is inferred automatically.</p></div>
        <button class="secondary compact" type="button" :disabled="indexing" @click="rebuildEmbeddings"><RotateCw aria-hidden="true" />{{ indexing ? 'Rebuilding…' : 'Rebuild index' }}</button>
      </header>
      <div class="en-ai-card-body en-ai-grid">
        <label>
          <span>Provider</span>
          <select v-model="form.routes.embedding.source" @change="onRouteSourceChanged('embedding')">
            <option value="disabled">Disabled</option>
            <option v-for="provider in embeddingProviderOptions" :key="provider.key" :value="provider.source">{{ provider.label }}</option>
          </select>
        </label>
        <label>
          <span>Model</span>
          <select v-if="selectedEmbeddingAddonProvider && selectedEmbeddingModels.length" v-model="form.routes.embedding.model">
            <option value="">Select a model</option>
            <option v-for="model in selectedEmbeddingModels" :key="addonModelId(model)" :value="addonModelId(model)">{{ addonModelLabel(model) }}</option>
          </select>
          <input v-else v-model.trim="form.routes.embedding.model" type="text" :disabled="form.routes.embedding.source === 'disabled'" placeholder="Embedding model id">
        </label>
        <label><span>Search result limit</span><input v-model.number="form.routes.embedding.searchTopK" type="number" min="1" max="100"></label>
        <label><span>Semantic threshold</span><input v-model.number="form.routes.embedding.threshold" type="number" min="0" max="1" step="0.01"></label>
      </div>
      <details class="en-ai-advanced">
        <summary>Advanced indexing settings</summary>
        <div class="en-ai-card-body en-ai-grid">
          <label><span>Chunk strategy</span><select v-model="form.routes.embedding.chunkStrategy"><option value="markdown-heading">Markdown headings</option><option value="paragraph">Paragraphs</option><option value="fixed">Fixed size</option><option value="hybrid">Hybrid</option></select></label>
          <label><span>Distance</span><select v-model="form.routes.embedding.distance"><option value="cosine">Cosine</option><option value="dot">Dot product</option><option value="euclidean">Euclidean</option></select></label>
          <label><span>Chunk size</span><input v-model.number="form.routes.embedding.chunkSize" type="number" min="64" step="64"></label>
          <label><span>Chunk overlap</span><input v-model.number="form.routes.embedding.chunkOverlap" type="number" min="0" step="16"></label>
          <label><span>Dimensions</span><input v-model.number="form.routes.embedding.dimensions" type="number" min="0"></label>
          <label><span>Debounce (ms)</span><input v-model.number="form.routes.embedding.debounceMs" type="number" min="0" step="250"></label>
        </div>
      </details>
    </section>

    <section v-else-if="activePage === 'ocr'" class="en-ai-card">
      <header class="en-ai-card-header"><div><h4>OCR route</h4><p>Configure the provider used by the installed OCR addon.</p></div></header>
      <div class="en-ai-card-body en-ai-grid">
        <label>
          <span>Provider</span>
          <select v-model="form.routes.ocr.source" @change="onRouteSourceChanged('ocr')">
            <option value="disabled">Disabled</option>
            <option v-for="provider in ocrProviderOptions" :key="provider.key" :value="provider.source">{{ provider.label }}</option>
          </select>
        </label>
        <label>
          <span>Model</span>
          <select v-if="selectedOcrAddonProvider && selectedOcrModels.length" v-model="form.routes.ocr.model">
            <option value="">Select a model</option>
            <option v-for="model in selectedOcrModels" :key="addonModelId(model)" :value="addonModelId(model)">{{ addonModelLabel(model) }}</option>
          </select>
          <input v-else v-model.trim="form.routes.ocr.model" type="text" :disabled="form.routes.ocr.source === 'disabled'" placeholder="OCR model id">
        </label>
        <label><span>Languages</span><input v-model.trim="form.routes.ocr.languages" type="text" placeholder="eng,fra"></label>
        <label><span>Output</span><select v-model="form.routes.ocr.output"><option value="markdown">Markdown</option><option value="plain-text">Plain text</option><option value="layout-markdown">Layout Markdown</option></select></label>
      </div>
      <details class="en-ai-advanced">
        <summary>Advanced OCR settings</summary>
        <div class="en-ai-card-body en-ai-grid">
          <label><span>Confidence threshold</span><input v-model.number="form.routes.ocr.confidenceThreshold" type="number" min="0" max="1" step="0.01"></label>
          <label><span>PDF mode</span><select v-model="form.routes.ocr.pdfMode"><option value="missing-text-only">Only pages without text</option><option value="all-pages">All pages</option><option value="skip-text-pdf">Skip text PDFs</option></select></label>
        </div>
      </details>
    </section>

    <p v-if="providerMessage" class="en-ai-feedback">{{ providerMessage }}</p>
  </section>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RotateCw } from '@lucide/vue'
import log from '@/platform/runtimeLogShim'
import { useAddonsStore } from '@/store/addons'
import { normalizeAiConfig } from 'common/elephantnote/aiProviders'
import { elephantnoteClient } from '../../services/elephantnoteClient'
import { clonePlainObject } from './settingsModelHelpers'

const props = defineProps({
  page: { type: String, default: '' },
  initialPage: { type: String, default: 'chat' }
})
const SUPPORTED_PAGES = new Set(['chat', 'embedding', 'ocr'])
const CACHE_KEY = 'elephantnote:ai-settings-draft'
const addonsStore = useAddonsStore()
const activePage = computed(() => {
  const requested = props.page || props.initialPage
  return SUPPORTED_PAGES.has(requested) ? requested : 'chat'
})
const indexing = ref(false)
const providerMessage = ref('')
const currentConfig = ref(normalizeAiConfig())
const providerRows = ref([])
const addonModels = ref({})
const hydrated = ref(false)
const dirty = ref(false)
let autosaveTimer = 0

const defaultRoute = () => ({ source: 'disabled', model: '', endpoint: '' })
const defaultForm = () => ({
  routes: {
    chat: {
      ...defaultRoute(),
      systemPrompt: '',
      temperature: 0.2,
      maxTokens: 2048,
      contextWindow: 8192,
      ragTopK: 6,
      enableRag: true,
      enableTools: false,
      stream: true
    },
    embedding: {
      ...defaultRoute(),
      searchTopK: 20,
      threshold: 0.35,
      autoIndex: true,
      backgroundIndex: true,
      debounceMs: 1500,
      distance: 'cosine',
      dimensions: 0,
      chunkStrategy: 'markdown-heading',
      chunkSize: 700,
      chunkOverlap: 80
    },
    ocr: {
      ...defaultRoute(),
      languages: 'eng,fra',
      output: 'markdown',
      pdfMode: 'missing-text-only',
      confidenceThreshold: 0.55
    }
  }
})
const form = ref(defaultForm())

const providerSource = (provider) => provider.type === 'openai-compatible' ? 'api' : provider.type
const externalProviders = computed(() => providerRows.value
  .filter((provider) => provider?.enabled)
  .map((provider) => ({
    key: `external:${provider.id || providerSource(provider)}`,
    source: providerSource(provider),
    label: provider.label || providerSource(provider),
    kind: 'external',
    provider
  })))
const addonProviders = computed(() => addonsStore.getContributions('ai.providers')
  .map((entry) => ({ addonId: entry.addonId, ...(entry.contribution || {}) }))
  .filter((provider) => typeof provider.providerId === 'string' && provider.providerId.trim())
  .map((provider) => ({
    ...provider,
    providerId: provider.providerId.trim(),
    title: provider.title || provider.providerId,
    capabilities: Array.isArray(provider.capabilities) ? provider.capabilities : ['chat']
  })))
const addonSupports = (provider, capability) => provider.capabilities.includes(capability)
const providerOptions = (capability) => {
  const seen = new Set()
  return [...externalProviders.value, ...addonProviders.value
    .filter((provider) => addonSupports(provider, capability))
    .map((provider) => ({
      key: `addon:${provider.addonId}:${provider.providerId}`,
      source: provider.providerId,
      label: provider.title,
      kind: 'addon',
      provider
    }))]
    .filter((option) => option.source && option.source !== 'disabled' && !seen.has(option.source) && seen.add(option.source))
}
const chatProviderOptions = computed(() => providerOptions('chat'))
const embeddingProviderOptions = computed(() => providerOptions('embedding'))
const ocrProviderOptions = computed(() => providerOptions('ocr'))
const optionsForRoute = (routeName) => routeName === 'chat'
  ? chatProviderOptions.value
  : routeName === 'embedding'
    ? embeddingProviderOptions.value
    : ocrProviderOptions.value
const addonProviderFor = (source, capability) => addonProviders.value.find((provider) => provider.providerId === source && addonSupports(provider, capability)) || null
const selectedChatAddonProvider = computed(() => addonProviderFor(form.value.routes.chat.source, 'chat'))
const selectedEmbeddingAddonProvider = computed(() => addonProviderFor(form.value.routes.embedding.source, 'embedding'))
const selectedOcrAddonProvider = computed(() => addonProviderFor(form.value.routes.ocr.source, 'ocr'))
const modelsFor = (source) => addonModels.value[source] || []
const selectedChatModels = computed(() => modelsFor(form.value.routes.chat.source))
const selectedEmbeddingModels = computed(() => modelsFor(form.value.routes.embedding.source))
const selectedOcrModels = computed(() => modelsFor(form.value.routes.ocr.source))

const addonModelId = (model) => String(model?.model || model?.id || model?.value || '')
const addonModelLabel = (model) => String(model?.displayName || model?.label || model?.model || model?.id || model?.value || '')
const providerEndpoint = (source) => {
  const addon = addonProviders.value.find((provider) => provider.providerId === source)
  if (addon) return addon.endpoint || ''
  return providerRows.value.find((row) => providerSource(row) === source)?.endpoint || ''
}
const providerTransport = (source) => {
  if (!source || source === 'disabled') return 'disabled'
  return addonProviders.value.find((provider) => provider.providerId === source)?.transport || source
}
const routeProviderLabel = (routeName) => {
  const source = form.value.routes[routeName]?.source || 'disabled'
  if (source === 'disabled') return 'Disabled'
  return optionsForRoute(routeName).find((option) => option.source === source)?.label || 'Disabled'
}
const normalizeRoute = (route = {}, fallback) => ({
  ...fallback,
  ...route,
  source: route.source || route.provider || fallback.source
})
const applyConfig = (config = {}) => {
  const defaults = defaultForm()
  const routes = config.routes || {}
  providerRows.value = Array.isArray(config.providers?.list) ? clonePlainObject(config.providers.list) : []
  form.value = {
    routes: {
      chat: normalizeRoute(routes.chat, defaults.routes.chat),
      embedding: normalizeRoute(routes.embedding, defaults.routes.embedding),
      ocr: normalizeRoute(routes.ocr, defaults.routes.ocr)
    }
  }
}
const sanitizeRoute = (routeName) => {
  const route = form.value.routes[routeName]
  const allowed = new Set(['disabled', ...optionsForRoute(routeName).map((option) => option.source)])
  if (!allowed.has(route.source)) {
    route.source = 'disabled'
    route.model = ''
    route.endpoint = ''
    return true
  }
  if (route.source === 'disabled' && (route.model || route.endpoint)) {
    route.model = ''
    route.endpoint = ''
    return true
  }
  return false
}
const sanitizeRoutes = () => ['chat', 'embedding', 'ocr'].some((routeName) => sanitizeRoute(routeName))
const routePayload = (routeName) => {
  const route = form.value.routes[routeName]
  const source = route.source || 'disabled'
  return {
    ...clonePlainObject(route),
    source,
    provider: source,
    transport: providerTransport(source),
    endpoint: source === 'disabled' ? '' : providerEndpoint(source),
    model: source === 'disabled' ? '' : route.model
  }
}
const buildConfig = () => {
  const chat = routePayload('chat')
  const openModelsAvailable = addonProviders.value.some((provider) => provider.providerId === 'app-local')
  const localAi = clonePlainObject(currentConfig.value.localAi || {})
  if (!openModelsAvailable) localAi.enabled = false
  return {
    ...clonePlainObject(currentConfig.value),
    localAi,
    provider: chat.provider,
    transport: chat.transport,
    endpoint: chat.endpoint,
    model: chat.model,
    providers: {
      ...clonePlainObject(currentConfig.value.providers || {}),
      list: clonePlainObject(providerRows.value)
    },
    routes: {
      ...clonePlainObject(currentConfig.value.routes || {}),
      chat,
      embedding: routePayload('embedding'),
      ocr: routePayload('ocr')
    }
  }
}

const loadAddonModels = async (provider) => {
  if (!provider || typeof provider.getModels !== 'function') return []
  try {
    const result = await provider.getModels()
    const models = Array.isArray(result) ? result : Array.isArray(result?.data) ? result.data : []
    addonModels.value = { ...addonModels.value, [provider.providerId]: models }
    return models
  } catch (error) {
    providerMessage.value = `${provider.title}: ${error instanceof Error ? error.message : String(error)}`
    addonModels.value = { ...addonModels.value, [provider.providerId]: [] }
    return []
  }
}
const onRouteSourceChanged = async (routeName) => {
  const route = form.value.routes[routeName]
  if (route.source === 'disabled') {
    route.model = ''
    route.endpoint = ''
    return
  }
  const capability = routeName === 'embedding' ? 'embedding' : routeName
  const provider = addonProviderFor(route.source, capability)
  if (!provider) {
    route.model = ''
    return
  }
  const models = await loadAddonModels(provider)
  route.model = addonModelId(models.find((model) => model?.isDefault) || models[0])
}
const rebuildEmbeddings = async () => {
  indexing.value = true
  try {
    await elephantnoteClient.search.rebuild?.()
    providerMessage.value = 'Embedding rebuild started.'
  } catch (error) {
    providerMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    indexing.value = false
  }
}
const saveConfig = async (reason = 'change') => {
  if (!hydrated.value) return
  clearTimeout(autosaveTimer)
  const payload = buildConfig()
  log.info('[ai-route-settings] save:start', { page: activePage.value, reason })
  try {
    const saved = await elephantnoteClient.ai.setConfig(clonePlainObject(payload))
    currentConfig.value = normalizeAiConfig(saved || payload)
    providerRows.value = Array.isArray(currentConfig.value.providers?.list) ? clonePlainObject(currentConfig.value.providers.list) : []
    localStorage.setItem(CACHE_KEY, JSON.stringify(currentConfig.value))
    window.dispatchEvent(new CustomEvent('elephantnote:ai-config-changed', { detail: currentConfig.value }))
    dirty.value = false
    log.info('[ai-route-settings] save:done', { page: activePage.value, reason })
  } catch (error) {
    providerMessage.value = error instanceof Error ? error.message : String(error)
    log.warn('[ai-route-settings] save:failed', { page: activePage.value, reason, error: providerMessage.value })
  }
}
const scheduleAutosave = (reason = 'change') => {
  if (!hydrated.value) return
  dirty.value = true
  clearTimeout(autosaveTimer)
  autosaveTimer = window.setTimeout(() => saveConfig(reason), 700)
}
const loadConfig = async () => {
  hydrated.value = false
  try {
    currentConfig.value = normalizeAiConfig(await elephantnoteClient.ai.getConfig())
    applyConfig(currentConfig.value)
    const sanitized = sanitizeRoutes()
    for (const [routeName, capability] of [['chat', 'chat'], ['embedding', 'embedding'], ['ocr', 'ocr']]) {
      const provider = addonProviderFor(form.value.routes[routeName].source, capability)
      if (provider) await loadAddonModels(provider)
    }
    hydrated.value = true
    if (sanitized) {
      dirty.value = true
      await saveConfig('sanitize-unavailable-provider')
    }
  } catch (error) {
    providerMessage.value = error instanceof Error ? error.message : String(error)
    hydrated.value = true
  }
}

watch(form, () => scheduleAutosave('form-watch'), { deep: true, flush: 'sync' })
watch(() => addonProviders.value.map((provider) => `${provider.addonId}:${provider.providerId}:${provider.capabilities.join(',')}`).join('|'), async () => {
  if (!hydrated.value) return
  const changed = sanitizeRoutes()
  if (changed) scheduleAutosave('addon-provider-removed')
  const routeName = activePage.value
  const capability = routeName === 'embedding' ? 'embedding' : routeName
  const provider = addonProviderFor(form.value.routes[routeName]?.source, capability)
  if (provider && !addonModels.value[provider.providerId]) await loadAddonModels(provider)
})
onMounted(loadConfig)
onBeforeUnmount(() => {
  clearTimeout(autosaveTimer)
  if (dirty.value) void saveConfig('settings-close')
})
</script>

<style scoped>
.en-ai-settings { display: grid; gap: 14px; color: var(--en-text, #101828); }
h4, p { margin: 0; }
h4 { font-size: 14px; }
p, .en-ai-setting-copy span { color: var(--en-muted, #667085); font-size: 12px; }
.en-ai-card-header, .en-ai-setting-row { display: flex; align-items: center; gap: 10px; }
.en-ai-card { overflow: hidden; border: 1px solid var(--en-border); border-radius: 14px; background: var(--en-surface); }
.en-ai-card-header { justify-content: space-between; padding: 15px 16px; border-bottom: 1px solid var(--en-border); }
.en-ai-card-header > div { display: grid; gap: 4px; }
.en-ai-card-body { padding: 16px; }
.en-ai-setting-row { min-height: 66px; padding: 13px 16px; }
.en-ai-setting-copy { min-width: 0; flex: 1; display: grid; gap: 3px; }
.en-ai-badge { padding: 4px 8px; border-radius: 999px; background: var(--en-soft); color: var(--en-muted); font-size: 11px; }
.en-ai-badge.active { color: var(--en-primary); }
.en-ai-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.en-ai-grid label { min-width: 0; display: grid; gap: 5px; color: var(--en-muted); font-size: 11px; }
.en-ai-grid .wide { grid-column: 1 / -1; }
input, select, textarea { width: 100%; min-width: 0; box-sizing: border-box; border: 1px solid var(--en-border); border-radius: 8px; background: var(--en-surface); color: var(--en-text); padding: 8px 9px; }
input:disabled, select:disabled { opacity: .55; cursor: not-allowed; }
textarea { resize: vertical; }
button { min-height: 34px; display: inline-flex; align-items: center; justify-content: center; gap: 7px; padding: 0 11px; border: 1px solid var(--en-border); border-radius: 9px; background: var(--en-surface); color: var(--en-text); cursor: pointer; }
button:disabled { cursor: not-allowed; opacity: .55; }
button svg { width: 15px; height: 15px; }
.en-ai-advanced { border-top: 1px solid var(--en-border); }
.en-ai-advanced summary { padding: 12px 16px; cursor: pointer; color: var(--en-muted); font-size: 12px; }
.en-ai-feedback { margin: 0; color: var(--en-muted); font-size: 11px; }
.en-ai-switch { flex: 0 0 auto; }
@media (max-width: 760px) {
  .en-ai-grid { grid-template-columns: 1fr; }
  .en-ai-grid .wide { grid-column: auto; }
}
</style>
