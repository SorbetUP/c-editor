import './executableCodeNativeRuntime.css'

const ROOT_SELECTOR = '.muya-container, .en-editor-host, .ag-editor'
const OUTPUT_TAG = 'elephant-code-output'
const DEFAULT_OUTPUT_LINES = 200
const RUN_TIMEOUT_MS = 22_000
const STOP_TIMEOUT_MS = 10_000
const DETACHED_TTL_MS = 1000

let executionSequence = 0

const errorMessage = (error) => error?.message || String(error || 'Unknown error')
const byteLength = (value = '') => {
  try { return new TextEncoder().encode(String(value)).byteLength } catch { return String(value).length }
}
const normalizeLanguage = (value = '') => String(value)
  .trim()
  .toLowerCase()
  .replace(/^language-/, '')
  .replace(/^lang-/, '')

const withTimeout = (promise, timeoutMs, label) => {
  let timer
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject(new Error(label)), timeoutMs)
  })
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer))
}

const invokePrograms = async(target, action, payload = {}) => {
  const invoke = target?.__TAURI__?.core?.invoke
  if (typeof invoke !== 'function') {
    throw new Error('The Tauri command API is unavailable for code execution.')
  }

  if (action === 'list') return invoke('tauri_programs_list_with_custom')
  if (action === 'set') return invoke('tauri_programs_set_with_custom', { environments: payload })

  const request = action === 'stop'
    ? { id: '', command: '', cwd: null, executionId: payload.executionId, stop: true }
    : {
        id: payload.id,
        command: payload.command,
        cwd: payload.cwd || null,
        executionId: payload.executionId,
        stop: false
      }
  const timeoutMs = action === 'run' ? RUN_TIMEOUT_MS : STOP_TIMEOUT_MS
  return withTimeout(
    Promise.resolve(invoke('tauri_programs_run_with_custom', request)),
    timeoutMs,
    `Code execution IPC did not answer within ${timeoutMs} ms.`
  )
}

const installProgramsApi = (target) => {
  target.elephantnote = target.elephantnote || {}
  target.elephantnote.programs = {
    list: () => invokePrograms(target, 'list'),
    set: (payload = {}) => invokePrograms(target, 'set', payload.environments || payload),
    run: (payload = {}) => invokePrograms(target, 'run', payload),
    stop: (payload = {}) => invokePrograms(target, 'stop', payload)
  }

  const api = target.elephantnote.api
  if (!api?.call || api.call.__elephantProgramsPatched) return
  const original = api.call.bind(api)
  const patched = async(action, payload = {}) => {
    if (action === 'programs.list') return { ok: true, data: await invokePrograms(target, 'list') }
    if (action === 'programs.set') {
      return { ok: true, data: await invokePrograms(target, 'set', payload.environments || payload) }
    }
    if (action === 'programs.run') return { ok: true, data: await invokePrograms(target, 'run', payload) }
    if (action === 'programs.stop') return { ok: true, data: await invokePrograms(target, 'stop', payload) }
    return original(action, payload)
  }
  patched.__elephantProgramsPatched = true
  api.call = patched
}

const languageFromPre = (pre) => {
  const native = pre?.querySelector?.('.ag-language-input')
  const explicit = native?.textContent || native?.dataset?.value || pre?.dataset?.language
  if (explicit) return normalizeLanguage(explicit)
  for (const className of pre?.classList || []) {
    if (className.startsWith('language-')) return normalizeLanguage(className.slice(9))
  }
  return ''
}

const sourceFromPre = (pre) => String(
  pre?.querySelector?.('code')?.innerText ??
  pre?.querySelector?.('code')?.textContent ??
  ''
).replace(/\u00a0/g, ' ')

const outputText = (result = {}) => [result.stdout, result.stderr || result.error]
  .filter(Boolean)
  .join('\n')

const copyText = async(target, text) => {
  if (target.navigator?.clipboard?.writeText) return target.navigator.clipboard.writeText(text)
  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.style.cssText = 'position:fixed;left:-10000px;top:0;opacity:0'
  document.body.append(textarea)
  textarea.select()
  document.execCommand?.('copy')
  textarea.remove()
}

const outputStyles = `
  :host {
    display: block;
    box-sizing: border-box;
    width: 100%;
    color: inherit;
    font-family: var(--font-family, system-ui, sans-serif);
  }
  :host([hidden]) { display: none !important; }
  .panel {
    border-top: 1px solid color-mix(in srgb, currentColor 10%, transparent);
    background: color-mix(in srgb, var(--editorBgColor, #29292c) 96%, currentColor 4%);
  }
  .panel.error { border-top-color: color-mix(in srgb, #e05252 58%, transparent); }
  header {
    display: flex;
    min-height: 38px;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 5px 9px 5px 12px;
    border-bottom: 1px solid color-mix(in srgb, currentColor 7%, transparent);
  }
  .identity, .actions { display: flex; align-items: center; }
  .identity { min-width: 0; gap: 7px; font-size: 12px; }
  .actions { flex: 0 0 auto; gap: 2px; }
  .dot { width: 6px; height: 6px; flex: 0 0 auto; border-radius: 50%; background: #22a657; }
  .running .dot { background: var(--primary-color, #1687ff); }
  .error .dot { background: #e05252; }
  .meta { overflow: hidden; color: color-mix(in srgb, currentColor 48%, transparent); text-overflow: ellipsis; white-space: nowrap; }
  button {
    padding: 4px 7px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: color-mix(in srgb, currentColor 60%, transparent);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }
  button:hover:not(:disabled) { background: color-mix(in srgb, currentColor 8%, transparent); color: inherit; }
  button:disabled { opacity: .35; cursor: default; }
  .body { max-height: 300px; overflow: auto; padding: 10px 12px 12px; }
  .stream + .stream { margin-top: 10px; }
  .label { margin-bottom: 4px; color: color-mix(in srgb, currentColor 45%, transparent); font-size: 10px; font-weight: 700; letter-spacing: .05em; text-transform: uppercase; }
  .stream.error .label { color: #e05252; }
  pre { margin: 0; padding: 0; overflow: visible; border: 0; background: transparent; color: inherit; font: 12px/1.55 var(--codeFontFamily, ui-monospace, SFMono-Regular, Menlo, monospace); white-space: pre-wrap; word-break: break-word; }
  .empty { margin: 0; color: color-mix(in srgb, currentColor 52%, transparent); font-size: 12px; }
  .progress { height: 2px; overflow: hidden; border-radius: 999px; background: color-mix(in srgb, var(--primary-color, #1687ff) 16%, transparent); }
  .progress::after { content: ''; display: block; width: 36%; height: 100%; border-radius: inherit; background: var(--primary-color, #1687ff); animation: progress 1s ease-in-out infinite alternate; }
  @keyframes progress { from { transform: translateX(-20%); } to { transform: translateX(220%); } }
`

const createOutputElementClass = (target) => class ElephantCodeOutputElement extends target.HTMLElement {
  constructor() {
    super()
    this.attachShadow({ mode: 'open' })
  }

  connectedCallback() {
    target.__ELEPHANT_NATIVE_CODE_RUNTIME__?.registerOutput(this)
  }

  disconnectedCallback() {
    target.__ELEPHANT_NATIVE_CODE_RUNTIME__?.unregisterOutput(this)
  }

  renderState(state, runtime) {
    const shadow = this.shadowRoot
    shadow.replaceChildren()
    const style = document.createElement('style')
    style.textContent = outputStyles
    shadow.append(style)

    const visible = state.status !== 'idle' || Boolean(state.result)
    this.hidden = !visible
    if (!visible) return

    const running = state.status === 'running' || state.status === 'stopping'
    const result = state.result || {}
    const panel = document.createElement('section')
    panel.className = `panel${running ? ' running' : ''}${!running && result.success !== true && !result.interrupted ? ' error' : ''}`

    const header = document.createElement('header')
    const identity = document.createElement('div')
    identity.className = 'identity'
    const dot = document.createElement('span')
    dot.className = 'dot'
    const title = document.createElement('strong')
    title.textContent = running
      ? (state.status === 'stopping' ? 'Stopping' : 'Running')
      : result.interrupted ? 'Stopped' : result.success ? 'Output' : 'Error'
    const meta = document.createElement('span')
    meta.className = 'meta'
    meta.textContent = running ? state.language : [
      result.environment || result.language || state.language,
      Number.isFinite(Number(result.durationMs)) ? `${result.durationMs} ms` : '',
      result.exitCode !== null && result.exitCode !== undefined ? `exit ${result.exitCode}` : '',
      result.timedOut ? 'timed out' : '',
      result.truncated ? `last ${result.outputLineLimit || DEFAULT_OUTPUT_LINES} lines` : ''
    ].filter(Boolean).join(' · ')
    identity.append(dot, title, meta)

    const actions = document.createElement('div')
    actions.className = 'actions'
    const copy = document.createElement('button')
    const collapse = document.createElement('button')
    const clear = document.createElement('button')
    copy.type = collapse.type = clear.type = 'button'
    copy.textContent = 'Copy'
    collapse.textContent = state.collapsed ? 'Expand' : 'Collapse'
    clear.textContent = 'Clear'
    copy.disabled = running || !outputText(result)
    collapse.disabled = clear.disabled = running
    copy.addEventListener('click', async() => {
      try {
        await copyText(target, outputText(result))
        copy.textContent = 'Copied'
        setTimeout(() => { copy.textContent = 'Copy' }, 900)
      } catch { copy.textContent = 'Copy failed' }
    })
    collapse.addEventListener('click', () => runtime.setCollapsed(state, !state.collapsed))
    clear.addEventListener('click', () => runtime.clear(state))
    actions.append(copy, collapse, clear)
    header.append(identity, actions)
    panel.append(header)

    const body = document.createElement('div')
    body.className = 'body'
    body.hidden = state.collapsed
    if (running) {
      const progress = document.createElement('div')
      progress.className = 'progress'
      body.append(progress)
    } else {
      const appendStream = (labelText, value, isError = false) => {
        if (!value) return
        const stream = document.createElement('section')
        stream.className = `stream${isError ? ' error' : ''}`
        const label = document.createElement('div')
        label.className = 'label'
        label.textContent = labelText
        const pre = document.createElement('pre')
        pre.textContent = value
        stream.append(label, pre)
        body.append(stream)
      }
      appendStream('stdout', result.stdout)
      appendStream('stderr', result.stderr || result.error, true)
      if (!result.stdout && !result.stderr && !result.error) {
        const empty = document.createElement('p')
        empty.className = 'empty'
        empty.textContent = result.interrupted
          ? 'The program was stopped before producing output.'
          : 'The program completed without output.'
        body.append(empty)
      }
    }
    panel.append(body)
    shadow.append(panel)
  }
}

const defineOutputElement = (target) => {
  if (!target.customElements || target.customElements.get(OUTPUT_TAG)) return
  target.customElements.define(OUTPUT_TAG, createOutputElementClass(target))
}

export const installExecutableCodeNativeRuntime = (target = globalThis) => {
  const existing = target.__ELEPHANT_NATIVE_CODE_RUNTIME__
  if (existing) return existing

  installProgramsApi(target)
  defineOutputElement(target)

  const states = new Map()
  const roots = new WeakMap()
  let rootSequence = 0
  let disposed = false

  const rootId = (element) => {
    const root = element?.closest?.(ROOT_SELECTOR) || document.documentElement
    if (!roots.has(root)) roots.set(root, `editor-${++rootSequence}`)
    return roots.get(root)
  }
  const stateKey = (element) => `${rootId(element)}:${element.dataset.blockKey || element.closest('pre')?.id || ''}`
  const ensureState = (element) => {
    const key = stateKey(element)
    let state = states.get(key)
    if (!state) {
      state = {
        key,
        element,
        output: null,
        language: '',
        status: 'idle',
        result: null,
        executionId: '',
        collapsed: false,
        detachedAt: 0
      }
      states.set(key, state)
    } else {
      state.element = element
      state.detachedAt = 0
    }
    return state
  }

  const render = (state) => {
    state.element?.setRuntimeState?.(state, runtime)
    state.output?.renderState?.(state, runtime)
  }

  const registerRunButton = (element) => {
    const state = ensureState(element)
    render(state)
  }
  const unregisterRunButton = (element) => {
    const state = states.get(stateKey(element))
    if (!state) return
    if (state.element === element) {
      state.element = null
      state.detachedAt = Date.now()
    }
  }
  const registerOutput = (element) => {
    const state = ensureState(element)
    state.output = element
    render(state)
  }
  const unregisterOutput = (element) => {
    const state = states.get(stateKey(element))
    if (state?.output === element) state.output = null
  }

  const setCollapsed = (state, collapsed) => {
    state.collapsed = collapsed === true
    render(state)
  }
  const clear = (state) => {
    state.status = 'idle'
    state.result = null
    state.executionId = ''
    state.collapsed = false
    render(state)
  }

  const run = async(element) => {
    const state = ensureState(element)
    const pre = element.closest('pre')
    const language = languageFromPre(pre)
    const source = sourceFromPre(pre)
    const sourceBytes = byteLength(source)
    if (!language) {
      state.status = 'error'
      state.result = { success: false, error: 'Choose a supported language before running this code block.' }
      render(state)
      return
    }
    state.language = language
    state.executionId = `code-${Date.now()}-${++executionSequence}`
    state.status = 'running'
    state.result = null
    state.collapsed = false
    render(state)
    console.info(`[Code:UI] run:start execution_id=${state.executionId} language=${language} source_bytes=${sourceBytes}`)
    try {
      const result = await invokePrograms(target, 'run', {
        id: language,
        command: source,
        cwd: pre?.dataset?.cwd || null,
        executionId: state.executionId
      })
      state.result = result || { success: false, error: 'Code execution returned no result.' }
      state.status = state.result.interrupted ? 'stopped' : state.result.success ? 'done' : 'error'
      console.info(`[Code:UI] run:complete execution_id=${state.executionId} success=${state.result.success === true} duration_ms=${state.result.durationMs ?? 0}`)
    } catch (error) {
      state.status = 'error'
      state.result = { success: false, error: errorMessage(error) }
      console.error(`[Code:UI] run:error execution_id=${state.executionId} error=${errorMessage(error)}`)
    }
    render(state)
  }

  const stop = async(element) => {
    const state = ensureState(element)
    if (state.status !== 'running' || !state.executionId) return
    state.status = 'stopping'
    render(state)
    console.info(`[Code:UI] stop:start execution_id=${state.executionId}`)
    try {
      const result = await invokePrograms(target, 'stop', { executionId: state.executionId })
      state.status = 'stopped'
      state.result = {
        success: false,
        interrupted: true,
        stderr: result?.stderr || 'Execution interrupted by user.',
        executionId: state.executionId
      }
      console.info(`[Code:UI] stop:complete execution_id=${state.executionId}`)
    } catch (error) {
      state.status = 'error'
      state.result = { success: false, error: errorMessage(error), executionId: state.executionId }
      console.error(`[Code:UI] stop:error execution_id=${state.executionId} error=${errorMessage(error)}`)
    }
    render(state)
  }

  const prune = () => {
    const now = Date.now()
    for (const [key, state] of states.entries()) {
      if (!state.element && !state.output && state.detachedAt && now - state.detachedAt > DETACHED_TTL_MS) {
        states.delete(key)
      }
    }
  }
  const timer = setInterval(prune, 500)

  const runtime = {
    version: 'native-v1',
    registerRunButton,
    unregisterRunButton,
    registerOutput,
    unregisterOutput,
    run,
    stop,
    clear,
    setCollapsed,
    states,
    dispose() {
      if (disposed) return
      disposed = true
      clearInterval(timer)
      states.clear()
      delete target.__ELEPHANT_NATIVE_CODE_RUNTIME__
      console.info('[Code:UI] dispose:complete')
    }
  }

  target.__ELEPHANT_NATIVE_CODE_RUNTIME__ = runtime
  console.info('[Code:UI] install:complete', { runtime: runtime.version })
  return runtime
}
