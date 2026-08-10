import Muya from 'muya/lib'
import { initSync as initRustSync, MuyaEditor } from 'muya-rust-wasm-bundle'
import rustWasmBytes from 'muya-rust-wasm-inline'
import 'muya/themes/default.css'
import './style.css'

const host = document.querySelector('#muya-host')

const resolveBridge = () => {
  const chromeWebView = globalThis.chrome?.webview
  if (typeof chromeWebView?.postMessage === 'function') {
    return { name: 'chrome.webview', postMessage: chromeWebView.postMessage.bind(chromeWebView) }
  }

  const webkitHandlers = globalThis.webkit?.messageHandlers
  const webkitHandler = webkitHandlers?.avalonia || webkitHandlers?.invokeCSharpAction
  if (typeof webkitHandler?.postMessage === 'function') {
    return { name: 'webkit.messageHandlers', postMessage: webkitHandler.postMessage.bind(webkitHandler) }
  }

  // Kept only for the legacy browser harness. NativeWebView must expose one
  // of the platform transports above; this fallback is never installed by
  // Avalonia itself.
  if (typeof globalThis.invokeCSharpAction === 'function') {
    return { name: 'legacy-test-bridge', postMessage: globalThis.invokeCSharpAction }
  }

  return null
}

const bridge = (type, payload = {}) => {
  const transport = resolveBridge()
  const message = JSON.stringify({ type, ...payload })
  if (!transport) {
    document.documentElement.dataset.muyaNativeBridge = 'missing'
    console.error(`[muya] NativeWebView bridge unavailable for ${type}`)
    return false
  }

  document.documentElement.dataset.muyaNativeBridge = transport.name
  try {
    transport.postMessage(message)
    return true
  } catch (error) {
    document.documentElement.dataset.muyaNativeBridge = 'failed'
    console.error('[muya] NativeWebView bridge failed', error)
    return false
  }
}

const waitForBridge = async(timeoutMs = 5000) => {
  const deadline = Date.now() + timeoutMs
  while (!resolveBridge()) {
    if (Date.now() >= deadline) return false
    await new Promise((resolve) => setTimeout(resolve, 50))
  }
  return true
}

let editor = null
let ready = false
let documentId = null
let suppressChanges = 0
let rustReady = false
// Rust validates immutable Markdown snapshots. It is not a second live editor:
// Muya JS owns DOM/caret/selection/IME/history and remains the sole writer.
let rustValidationEditor = null
let rustValidationSnapshot = null
let rustQueue = Promise.resolve()
let rustInitialized = false

const publishError = (error) => bridge('error', {
  code: error?.code || 'rust-validation-failed',
  message: error?.message || String(error),
  documentId,
  engine: 'muya-js+muya-rust'
})

const rustSnapshotFor = (validationEditor) => {
  const response = JSON.parse(validationEditor.snapshot_json())
  if (response?.type !== 'snapshot' || !response.payload?.document?.nodes) {
    throw new Error('Muya Rust returned an invalid document snapshot.')
  }
  return response.payload
}

const validateMarkdownWithRust = (markdown) => {
  const job = rustQueue.then(async() => {
    if (!rustInitialized) {
      initRustSync(rustWasmBytes)
      rustInitialized = true
    }
    const nextValidationEditor = new MuyaEditor(String(markdown || ''))
    const nextSnapshot = rustSnapshotFor(nextValidationEditor)
    rustValidationEditor?.free?.()
    rustValidationEditor = nextValidationEditor
    rustValidationSnapshot = nextSnapshot
    rustReady = true
    return nextSnapshot
  })
  rustQueue = job.catch(() => {})
  return job
}

const publishReady = async() => {
  if (!await waitForBridge()) {
    publishError(new Error('NativeWebView bridge was not available after Muya initialization.'))
    return
  }
  bridge('ready', {
    engine: 'muya-js+muya-rust',
    rustRevision: rustValidationSnapshot?.revision ?? null
  })
}

const publishChange = async({ markdown = '' } = {}) => {
  if (suppressChanges > 0 || !documentId) return
  try {
    const snapshot = await validateMarkdownWithRust(markdown)
    bridge('content-changed', {
      documentId,
      content: markdown,
      engine: 'muya-js+muya-rust',
      rustRevision: snapshot.revision
    })
  } catch (error) {
    publishError(error)
  }
}

const setEditorMarkdown = async(markdown) => {
  if (!editor) initialize()
  const content = typeof markdown === 'string' ? markdown : ''
  suppressChanges += 1
  try {
    editor?.setMarkdown(content)
    await validateMarkdownWithRust(content)
  } catch (error) {
    publishError(error)
  } finally {
    suppressChanges = Math.max(0, suppressChanges - 1)
  }
}

const initialize = async() => {
  if (!host || editor) return
  editor = new Muya(host, {
    markdown: '',
    autoPairBracket: true,
    autoPairMarkdownSyntax: true,
    autoPairQuote: true,
    spellcheckEnabled: false,
    hideQuickInsertHint: true,
    t: (key) => key
  })
  editor.on('change', publishChange)
  await validateMarkdownWithRust('')
  ready = true
  document.documentElement.dataset.muyaNativeReady = 'true'
  document.documentElement.dataset.muyaEngine = 'muya-js+muya-rust'
  await publishReady()
}

globalThis.__ELEPHANT_MUYA__ = {
  get ready() {
    return ready
  },
  get rustReady() {
    return rustReady
  },
  getMarkdown() {
    return editor?.getMarkdown() || ''
  },
  getRustSnapshot() {
    return rustValidationSnapshot
  },
  setMarkdown(markdown) {
    return setEditorMarkdown(markdown)
  },
  focus() {
    editor?.focus()
  }
}

globalThis.__ELEPHANT_MUYA_HOST__ = {
  receive(message) {
    if (!message || typeof message !== 'object') return
    if (message.type === 'open-document') {
      documentId = typeof message.documentId === 'string' ? message.documentId : null
      void setEditorMarkdown(message.content)
      return
    }
    if (message.type === 'set-content') void setEditorMarkdown(message.content)
  }
}

window.addEventListener('keydown', async(event) => {
  if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== 's' || !documentId) return
  event.preventDefault()
  const content = editor?.getMarkdown() || ''
  try {
    const snapshot = await validateMarkdownWithRust(content)
    bridge('save-request', {
      requestId: globalThis.crypto?.randomUUID?.() || String(Date.now()),
      documentId,
      content,
      engine: 'muya-js+muya-rust',
      rustRevision: snapshot.revision
    })
  } catch (error) {
    publishError(error)
  }
}, true)

void initialize().catch(publishError)
