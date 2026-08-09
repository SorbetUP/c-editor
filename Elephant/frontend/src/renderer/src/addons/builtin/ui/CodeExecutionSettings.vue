<template>
  <section class="en-code-settings-page">
    <section class="en-code-card">
      <div class="en-code-row">
        <div><strong>Retained output</strong><span>Keep only the final stdout and stderr lines when a program produces too much output.</span></div>
        <select v-model.number="form.outputLineLimit" class="en-compact-select" :disabled="loading || saving" @change="saveConfig('output-limit')">
          <option v-for="limit in OUTPUT_LINE_LIMITS" :key="limit" :value="limit">{{ limit === 5000 ? '5,000 lines' : `${limit} lines` }}</option>
        </select>
      </div>
    </section>

    <section class="en-code-card">
      <header class="en-code-section-header" :class="{ collapsed: !interpretersExpanded }">
        <button class="en-code-section-toggle-copy" type="button" :aria-expanded="interpretersExpanded" aria-controls="en-code-interpreters-content" @click="toggleInterpreters">
          <strong>Interpreters</strong>
          <span>Choose detected runtimes or add a specific environment such as another Python installation.</span>
        </button>
        <div class="en-code-section-actions">
          <button v-if="interpretersExpanded" class="en-code-add" type="button" :disabled="loading || saving" @click="addingInterpreter = !addingInterpreter"><Plus aria-hidden="true" /> Add</button>
          <button v-if="interpretersExpanded" class="en-code-refresh" type="button" :disabled="loading || saving" @click="loadConfig"><RefreshCw :class="{ spinning: loading }" aria-hidden="true" /> Detect again</button>
          <button class="en-code-collapse" type="button" :title="interpretersExpanded ? 'Hide interpreters' : 'Show interpreters'" :aria-label="interpretersExpanded ? 'Hide interpreters' : 'Show interpreters'" :aria-expanded="interpretersExpanded" aria-controls="en-code-interpreters-content" @click="toggleInterpreters"><ChevronDown :class="{ collapsed: !interpretersExpanded }" aria-hidden="true" /></button>
        </div>
      </header>

      <div v-if="interpretersExpanded" id="en-code-interpreters-content">
        <form v-if="addingInterpreter" class="en-code-add-form" @submit.prevent="addInterpreter">
          <label><span>Template</span><select v-model="draft.template" class="en-compact-select" @change="applyTemplate"><option v-for="template in form.templates" :key="template.id" :value="template.id">{{ template.label }}</option></select></label>
          <label><span>Name</span><input v-model.trim="draft.label" class="en-compact-input" type="text" placeholder="Python ML" required></label>
          <label><span>Fence language</span><input v-model.trim="draft.id" class="en-compact-input" type="text" placeholder="python-ml" required></label>
          <label class="en-code-add-path"><span>Executable</span><input v-model.trim="draft.executable" class="en-compact-input" type="text" placeholder="/path/to/python" required></label>
          <label><span>Aliases</span><input v-model.trim="draft.aliases" class="en-compact-input" type="text" placeholder="py-ml, ml-python"></label>
          <label><span>Arguments</span><input v-model.trim="draft.args" class="en-compact-input" type="text" placeholder="-u -"></label>
          <div class="en-code-add-actions"><button class="en-secondary-button" type="button" @click="cancelAddInterpreter">Cancel</button><button class="en-primary-button" type="submit" :disabled="saving">Add interpreter</button></div>
        </form>

        <div v-if="loading" class="en-code-empty">Detecting local environments…</div>
        <template v-else>
          <article v-for="environment in form.environments" :key="environment.id" class="en-code-environment">
            <div class="en-code-environment-main">
              <div class="en-code-environment-title"><strong>{{ environment.label }}</strong><span class="en-code-status" :class="{ active: environment.available }">{{ environment.available ? 'Detected' : 'Not detected' }}</span></div>
              <span>{{ environment.available ? [environment.version, environment.configuredExecutable ? 'Custom path' : 'Auto-detected'].filter(Boolean).join(' · ') : 'Install the runtime or provide an executable path.' }}</span>
            </div>
            <label class="en-code-path"><span>Executable</span><input v-model.trim="environment.configuredExecutable" class="en-compact-input" type="text" :placeholder="environment.executable || 'Executable path'" :disabled="saving" @change="saveConfig(`path:${environment.id}`)"></label>
            <button v-if="environment.configuredExecutable" class="en-code-reset" type="button" :disabled="saving" title="Use automatic detection" @click="clearExecutable(environment)"><RotateCcw aria-hidden="true" /></button>
            <button class="en-switch" type="button" role="switch" :aria-label="`Enable ${environment.label}`" :aria-checked="environment.enabled !== false" :class="{ active: environment.enabled !== false }" :disabled="saving" @click="toggleEnvironment(environment)"><span /></button>
          </article>

          <article v-for="environment in form.customEnvironments" :key="environment.id" class="en-code-environment custom">
            <div class="en-code-environment-main">
              <div class="en-code-environment-title"><strong>{{ environment.label }}</strong><span class="en-code-status custom">{{ environment.id }}</span><span class="en-code-status" :class="{ active: environment.available }">{{ environment.available ? 'Detected' : 'Not detected' }}</span></div>
              <span>{{ [environment.version, environment.template && `Template: ${environment.template}`].filter(Boolean).join(' · ') || 'User-defined interpreter' }}</span>
            </div>
            <label class="en-code-path"><span>Executable</span><input v-model.trim="environment.configuredExecutable" class="en-compact-input" type="text" placeholder="Executable path" :disabled="saving" @change="saveConfig(`custom-path:${environment.id}`)"></label>
            <button class="en-code-remove" type="button" :disabled="saving" title="Remove interpreter" @click="removeInterpreter(environment.id)"><Trash2 aria-hidden="true" /></button>
            <button class="en-switch" type="button" role="switch" :aria-label="`Enable ${environment.label}`" :aria-checked="environment.enabled !== false" :class="{ active: environment.enabled !== false }" :disabled="saving" @click="toggleCustomEnvironment(environment)"><span /></button>
          </article>

          <div v-if="!form.environments.length && !form.customEnvironments.length" class="en-code-empty">No interpreter is configured.</div>
        </template>
      </div>
    </section>

    <p v-if="message" class="en-code-message" :class="{ error: messageIsError }">{{ message }}</p>
  </section>
</template>

<script setup>
import { onMounted, reactive, ref } from 'vue'
import { ChevronDown, Plus, RefreshCw, RotateCcw, Trash2 } from '@lucide/vue'
import log from '@/platform/runtimeLogShim'

const OUTPUT_LINE_LIMITS = [10, 20, 50, 100, 200, 500, 1000, 5000]
const INTERPRETERS_COLLAPSED_KEY = 'elephantnote:code-settings:interpreters-collapsed'
const readInterpretersExpanded = () => {
  try { return globalThis.localStorage?.getItem(INTERPRETERS_COLLAPSED_KEY) !== 'true' } catch { return true }
}
const loading = ref(false)
const saving = ref(false)
const message = ref('')
const messageIsError = ref(false)
const interpretersExpanded = ref(readInterpretersExpanded())
const addingInterpreter = ref(false)
const form = reactive({ outputLineLimit: 200, environments: [], customEnvironments: [], templates: [] })
const draft = reactive({ template: 'python', label: '', id: '', executable: '', aliases: '', args: '-u -' })

const programs = () => {
  const api = globalThis.elephantnote?.programs
  if (!api?.list || !api?.set) throw new Error('Code execution settings API is unavailable')
  return api
}

const normalizeLimit = (value) => {
  const parsed = Number.parseInt(value, 10)
  if (!Number.isFinite(parsed)) return 200
  return Math.min(5000, Math.max(10, parsed))
}
const normalizeToken = (value) => String(value || '').trim().toLowerCase().replace(/[^a-z0-9._-]+/g, '-')
const splitList = (value) => String(value || '').split(',').map((item) => normalizeToken(item)).filter(Boolean)
const splitArgs = (value) => String(value || '').trim().split(/\s+/).filter(Boolean)

const applyState = (state = {}) => {
  form.outputLineLimit = normalizeLimit(state.outputLineLimit)
  form.environments = Array.isArray(state.environments)
    ? state.environments.map((environment) => ({ ...environment, enabled: environment.enabled !== false, configuredExecutable: environment.configuredExecutable || '' }))
    : []
  form.customEnvironments = Array.isArray(state.customEnvironments)
    ? state.customEnvironments.map((environment) => ({ ...environment, enabled: environment.enabled !== false, configuredExecutable: environment.configuredExecutable || environment.executable || '' }))
    : []
  form.templates = Array.isArray(state.interpreterTemplates) ? state.interpreterTemplates.map((template) => ({ ...template })) : []
  if (!form.templates.some((template) => template.id === draft.template)) draft.template = form.templates[0]?.id || 'custom'
}

const customPayload = () => form.customEnvironments.map((environment) => ({
  id: environment.id,
  label: environment.label,
  aliases: Array.isArray(environment.aliases) ? environment.aliases : [],
  executable: environment.configuredExecutable || environment.executable || '',
  args: Array.isArray(environment.args) ? environment.args : [],
  enabled: environment.enabled !== false,
  template: environment.template || 'custom'
}))

const buildPayload = () => ({
  environments: {
    executionEnabled: true,
    outputLineLimit: normalizeLimit(form.outputLineLimit),
    environments: Object.fromEntries(form.environments.map((environment) => [environment.id, {
      enabled: environment.enabled !== false,
      executable: environment.configuredExecutable || ''
    }])),
    customEnvironments: customPayload()
  }
})

const loadConfig = async () => {
  if (loading.value) return
  loading.value = true
  message.value = ''
  messageIsError.value = false
  try {
    const state = await programs().list()
    applyState(state)
    log.info('[code-settings] load:done', { environments: form.environments.length, customEnvironments: form.customEnvironments.length, outputLineLimit: form.outputLineLimit })
  } catch (error) {
    message.value = error instanceof Error ? error.message : String(error)
    messageIsError.value = true
    log.error('[code-settings] load:failed', error)
  } finally {
    loading.value = false
  }
}

const saveConfig = async (reason) => {
  if (saving.value) return
  saving.value = true
  message.value = ''
  messageIsError.value = false
  try {
    const result = await programs().set(buildPayload())
    applyState(result)
    log.info('[code-settings] save:done', { reason, outputLineLimit: form.outputLineLimit, customEnvironments: form.customEnvironments.length })
  } catch (error) {
    message.value = error instanceof Error ? error.message : String(error)
    messageIsError.value = true
    log.error('[code-settings] save:failed', { reason, error })
  } finally {
    saving.value = false
  }
}

const toggleEnvironment = async (environment) => {
  environment.enabled = environment.enabled === false
  await saveConfig(`environment-toggle:${environment.id}`)
}
const toggleCustomEnvironment = async (environment) => {
  environment.enabled = environment.enabled === false
  await saveConfig(`custom-toggle:${environment.id}`)
}
const clearExecutable = async (environment) => {
  environment.configuredExecutable = ''
  await saveConfig(`environment-auto:${environment.id}`)
}
const toggleInterpreters = () => {
  interpretersExpanded.value = !interpretersExpanded.value
  try { globalThis.localStorage?.setItem(INTERPRETERS_COLLAPSED_KEY, String(!interpretersExpanded.value)) } catch {}
}

const applyTemplate = () => {
  const template = form.templates.find((item) => item.id === draft.template)
  if (!template) return
  if (!draft.label) draft.label = template.id === 'custom' ? 'Custom interpreter' : `${template.label} environment`
  if (!draft.id) draft.id = template.id === 'custom' ? 'custom-runtime' : `${template.id}-custom`
  draft.args = Array.isArray(template.args) ? template.args.join(' ') : ''
}
const cancelAddInterpreter = () => {
  addingInterpreter.value = false
  Object.assign(draft, { template: form.templates[0]?.id || 'python', label: '', id: '', executable: '', aliases: '', args: '' })
  applyTemplate()
}
const addInterpreter = async () => {
  const id = normalizeToken(draft.id)
  if (!id || !draft.label || !draft.executable) {
    message.value = 'Name, fence language and executable are required.'
    messageIsError.value = true
    return
  }
  if (form.environments.some((environment) => environment.id === id) || form.customEnvironments.some((environment) => environment.id === id)) {
    message.value = `An interpreter already uses ${id}.`
    messageIsError.value = true
    return
  }
  form.customEnvironments.push({
    id,
    label: draft.label,
    aliases: splitList(draft.aliases),
    configuredExecutable: draft.executable,
    executable: draft.executable,
    args: splitArgs(draft.args),
    enabled: true,
    available: false,
    template: draft.template
  })
  await saveConfig(`custom-add:${id}`)
  cancelAddInterpreter()
}
const removeInterpreter = async (id) => {
  form.customEnvironments = form.customEnvironments.filter((environment) => environment.id !== id)
  await saveConfig(`custom-remove:${id}`)
}

onMounted(async () => {
  await loadConfig()
  applyTemplate()
})
</script>

<style scoped>
.en-code-settings-page { display: grid; gap: 12px; }
.en-code-card { overflow: hidden; border: 1px solid var(--en-border, #c5cfdd); border-radius: 12px; background: var(--en-surface, #fff); }
.en-code-section-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 13px 15px; border-bottom: 1px solid var(--en-border, #c5cfdd); }
.en-code-section-header.collapsed { border-bottom: 0; }
.en-code-section-toggle-copy, .en-code-environment-main, .en-code-row > div { min-width: 0; display: grid; gap: 3px; }
.en-code-section-toggle-copy { flex: 1; padding: 0; border: 0; background: transparent; color: inherit; text-align: left; cursor: pointer; }
.en-code-section-header strong, .en-code-environment strong, .en-code-row strong { font-size: 12.5px; }
.en-code-section-header span, .en-code-environment-main > span, .en-code-row span { color: var(--en-muted, #667085); font-size: 10.5px; line-height: 1.45; }
.en-code-section-actions { display: flex; flex: 0 0 auto; align-items: center; gap: 7px; }
.en-code-row, .en-code-environment { display: grid; align-items: center; gap: 12px; padding: 13px 15px; }
.en-code-row { grid-template-columns: minmax(0, 1fr) auto; }
.en-code-environment { grid-template-columns: minmax(150px, 1fr) minmax(220px, 1.2fr) 30px auto; }
.en-code-environment + .en-code-environment { border-top: 1px solid var(--en-border, #c5cfdd); }
.en-code-environment.custom { background: color-mix(in srgb, var(--en-primary, #2563eb) 3%, transparent); }
.en-code-environment-title { display: flex; align-items: center; gap: 7px; flex-wrap: wrap; }
.en-code-path { min-width: 0; display: grid; gap: 4px; }
.en-code-path > span, .en-code-add-form label > span { color: var(--en-muted, #667085); font-size: 9.5px; }
.en-code-path input { width: 100%; min-width: 0; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.en-code-status { padding: 3px 7px; border-radius: 999px; background: var(--en-soft, #e9eff7); color: var(--en-muted, #667085); font-size: 9px; font-weight: 650; }
.en-code-status.active { background: color-mix(in srgb, #16a34a 14%, transparent); color: #15803d; }
.en-code-status.custom { color: var(--en-primary, #2563eb); }
.en-code-refresh, .en-code-add, .en-code-reset, .en-code-remove, .en-code-collapse { display: inline-flex; align-items: center; justify-content: center; border: 1px solid var(--en-border, #c5cfdd); background: transparent; color: inherit; cursor: pointer; }
.en-code-refresh, .en-code-add { min-height: 30px; gap: 6px; padding: 0 10px; border-radius: 8px; font-size: 10.5px; }
.en-code-refresh svg, .en-code-add svg, .en-code-reset svg, .en-code-remove svg, .en-code-collapse svg { width: 14px; height: 14px; }
.en-code-reset, .en-code-remove, .en-code-collapse { width: 30px; height: 30px; border-radius: 8px; }
.en-code-remove:hover { color: #dc2626; }
.en-code-collapse svg { transition: transform 150ms ease; }
.en-code-collapse svg.collapsed { transform: rotate(-90deg); }
.en-code-add-form { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; padding: 14px 15px; border-bottom: 1px solid var(--en-border, #c5cfdd); background: color-mix(in srgb, var(--en-soft, #e9eff7) 42%, transparent); }
.en-code-add-form label { min-width: 0; display: grid; gap: 4px; }
.en-code-add-form input, .en-code-add-form select { width: 100%; min-width: 0; }
.en-code-add-path { grid-column: span 2; }
.en-code-add-actions { grid-column: 1 / -1; display: flex; justify-content: flex-end; gap: 7px; }
.en-code-empty { padding: 22px; color: var(--en-muted, #667085); font-size: 11px; text-align: center; }
.en-code-message { margin: 0; padding: 9px 11px; border-radius: 8px; background: color-mix(in srgb, #16a34a 10%, var(--en-surface, #fff)); color: #15803d; font-size: 10.5px; }
.en-code-message.error { background: color-mix(in srgb, #dc2626 10%, var(--en-surface, #fff)); color: #b91c1c; }
.spinning { animation: en-code-spin 800ms linear infinite; }
@keyframes en-code-spin { to { transform: rotate(360deg); } }
@media (max-width: 880px) {
  .en-code-environment { grid-template-columns: 1fr auto auto; }
  .en-code-path { grid-column: 1 / -1; grid-row: 2; }
  .en-code-add-form { grid-template-columns: 1fr 1fr; }
}
@media (max-width: 620px) {
  .en-code-row { grid-template-columns: 1fr; }
  .en-code-section-header { align-items: flex-start; }
  .en-code-section-actions { flex-wrap: wrap; justify-content: flex-end; }
  .en-code-add-form { grid-template-columns: 1fr; }
  .en-code-add-path, .en-code-add-actions { grid-column: 1; }
}
</style>
