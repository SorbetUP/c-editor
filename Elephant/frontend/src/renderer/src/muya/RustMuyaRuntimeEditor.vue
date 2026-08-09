<template>
  <div
    class="muya-rust-runtime-shell"
    :data-muya-runtime-mode="mode"
  >
    <div
      ref="rootRef"
      class="muya-rust-runtime-editor"
      data-testid="muya-rust-runtime-editor"
    />
    <div
      v-if="errorMessage"
      class="muya-rust-runtime-error"
      role="alert"
    >
      {{ errorMessage }}
    </div>
  </div>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

import StableCompleteMuyaWithRustCore from './completeMuyaRustAdapter.js.wrapper.js'
import { parseEditorResponse } from '../editor-rust/protocol'
import { initializeExperimentalRustRuntime } from '../editor-rust/runtime'

const props = defineProps({
  modelValue: { type: String, default: '' },
  mode: { type: String, default: 'rust' },
  factory: { type: Function, default: null },
  onFileDrop: { type: Function, default: null },
  onUriDrop: { type: Function, default: null },
  onImageClick: { type: Function, default: null }
})

const emit = defineEmits(['update:modelValue', 'ready', 'change', 'error'])
const rootRef = ref(null)
const errorMessage = ref('')
let runtime = null
let runtimeMarkdown = ''
let mountGeneration = 0
let syncTimer = null
let syncRequested = false
let syncInFlight = null
let internalPropUpdatePending = false
let internalPropResetTimer = null
let userMutationObserved = false
let disposeUserMutationBoundary = () => {}

const markUserMutation = (reason) => {
  if (userMutationObserved) return
  userMutationObserved = true
  console.info('[elephantnote:rust-editor] user-mutation-boundary:opened', { reason })
}

const isMutatingBeforeInput = (event) => {
  const inputType = String(event?.inputType || '')
  return inputType.startsWith('insert') ||
    inputType.startsWith('delete') ||
    inputType.startsWith('history') ||
    inputType.startsWith('format')
}

const installUserMutationBoundary = () => {
  const root = rootRef.value
  if (!root) return () => {}
  const beforeInput = (event) => {
    if (isMutatingBeforeInput(event)) markUserMutation(`beforeinput:${event.inputType}`)
  }
  const paste = () => markUserMutation('paste')
  const drop = () => markUserMutation('drop')
  root.addEventListener('beforeinput', beforeInput, true)
  root.addEventListener('paste', paste, true)
  root.addEventListener('drop', drop, true)
  return () => {
    root.removeEventListener('beforeinput', beforeInput, true)
    root.removeEventListener('paste', paste, true)
    root.removeEventListener('drop', drop, true)
  }
}

const reportError = (error) => {
  errorMessage.value = error?.message || String(error)
  const details = {
    name: error?.name || 'Error',
    message: errorMessage.value,
    revision: runtime?.bridge?.revision ?? null,
    selection: runtime?.bridge?.selection ?? null,
    markdownLength: runtimeMarkdown.length
  }
  window.__ELEPHANT_DEBUG_LOGS__ = Array.isArray(window.__ELEPHANT_DEBUG_LOGS__)
    ? window.__ELEPHANT_DEBUG_LOGS__
    : []
  window.__ELEPHANT_DEBUG_LOGS__.push({
    at: new Date().toISOString(),
    level: 'error',
    message: '[elephantnote:rust-editor] runtime error',
    details
  })
  if (window.__ELEPHANT_DEBUG_LOGS__.length > 1000) {
    window.__ELEPHANT_DEBUG_LOGS__.splice(0, window.__ELEPHANT_DEBUG_LOGS__.length - 1000)
  }
  console.error('[elephantnote:rust-editor] runtime error', details)
  emit('error', error)
}

const readRuntimeMarkdown = async () => {
  if (runtime?.muya?.getMarkdown) return String(runtime.muya.getMarkdown() || '')
  if (runtime?.bridge?.engine?.snapshot_json) {
    const response = parseEditorResponse(await runtime.bridge.engine.snapshot_json())
    if (response.type !== 'snapshot') throw new TypeError(`Expected a Rust snapshot, received ${response.type}.`)
    return String(response.payload.markdown || '')
  }
  return runtimeMarkdown
}

const markInternalPropUpdate = () => {
  internalPropUpdatePending = true
  if (internalPropResetTimer) window.clearTimeout(internalPropResetTimer)
  internalPropResetTimer = window.setTimeout(() => {
    internalPropResetTimer = null
    internalPropUpdatePending = false
  }, 0)
}

const flushMarkdownSync = () => {
  if (syncInFlight) return syncInFlight
  const generation = mountGeneration
  syncInFlight = (async () => {
    try {
      while (true) {
        syncRequested = false
        const next = await readRuntimeMarkdown()
        if (generation !== mountGeneration) return
        runtimeMarkdown = next
        if (next !== props.modelValue) {
          if (!userMutationObserved) {
            console.warn('[elephantnote:rust-editor] ignored programmatic markdown change before user mutation', {
              generation,
              previousLength: String(props.modelValue || '').length,
              nextLength: next.length,
              revision: runtime?.bridge?.revision ?? null,
              reason: 'initial-rust-muya-synchronization'
            })
            if (!syncRequested) break
            continue
          }
          markInternalPropUpdate()
          emit('update:modelValue', next)
          emit('change', next)
        }
        if (!syncRequested) break
      }
    } catch (error) {
      if (generation === mountGeneration) reportError(error)
    } finally {
      syncInFlight = null
      if (syncRequested && generation === mountGeneration) scheduleMarkdownSync([true])
    }
  })()
  return syncInFlight
}

const scheduleMarkdownSync = (patches = []) => {
  if (!patches.length) return
  console.info('[elephantnote:rust-editor] patches:received', {
    generation: mountGeneration,
    patchCount: patches.length,
    revision: runtime?.bridge?.revision ?? null
  })
  syncRequested = true
  if (syncTimer || syncInFlight) return
  syncTimer = window.setTimeout(() => {
    syncTimer = null
    void flushMarkdownSync()
  }, 0)
}

const destroyRuntime = () => {
  if (syncTimer) {
    window.clearTimeout(syncTimer)
    syncTimer = null
  }
  if (internalPropResetTimer) {
    window.clearTimeout(internalPropResetTimer)
    internalPropResetTimer = null
  }
  internalPropUpdatePending = false
  userMutationObserved = false
  disposeUserMutationBoundary()
  disposeUserMutationBoundary = () => {}
  syncRequested = false
  runtime?.destroy?.()
  runtime = null
}

const createCompatBridge = (muya) => {
  const mirror = muya.__rustMirror
  const state = () => mirror?.state || { revision: 0, selection: { anchor: 0, focus: 0 } }
  const dispatch = (command = {}) => {
    const type = String(command.type || '')
    const handlers = {
      insert_text: () => mirror?.replaceRange(state().selection.anchor, state().selection.focus, command.text || ''),
      paste_markdown: () => mirror?.pasteClipboard('', command.markdown || ''),
      delete_backward: () => mirror?.deleteBackward(),
      insert_paragraph: () => mirror?.insertParagraph('after', ''),
      undo: () => muya.undo(),
      redo: () => muya.redo(),
      set_selection: () => mirror?.setSelection(command.selection?.anchor?.offset_utf16 || 0, command.selection?.focus?.offset_utf16 || 0),
      toggle_strong: () => muya.format('strong'),
      toggle_emphasis: () => muya.format('emphasis'),
      toggle_strike: () => muya.format('strikethrough'),
      insert_horizontal_rule: () => muya.updateParagraph('hr'),
      create_table: () => muya.createTable({ rows: command.rows, columns: command.columns }),
      insert_image: () => muya.insertImage({ source: command.source, alt: command.alt, title: command.title }),
      delete_block: () => muya.deleteParagraph(),
      duplicate_block: () => muya.duplicate(),
      commit_composition: () => mirror?.commitComposition(state().selection, command.text || ''),
      cancel_composition: () => undefined
    }
    const handler = handlers[type]
    return handler ? Promise.resolve(handler()) : Promise.reject(new Error(`Unsupported Muya compatibility command: ${type}`))
  }
  const snapshot = async () => ({ ...state(), markdown: muya.getMarkdown() })
  return {
    get revision () { return Number(state().revision) || 0 },
    get selection () { return state().selection },
    dispatch,
    snapshot,
    engine: { snapshot_json: async () => JSON.stringify({ type: 'snapshot', payload: await snapshot() }) }
  }
}

const mountRuntime = async (markdown) => {
  const generation = ++mountGeneration
  console.info('[elephantnote:rust-editor] mount:start', {
    generation,
    markdownLength: String(markdown || '').length,
    mode: props.mode,
    hasFactory: typeof props.factory === 'function'
  })
  destroyRuntime()
  errorMessage.value = ''
  runtimeMarkdown = String(markdown || '')
  rootRef.value?.replaceChildren()
  disposeUserMutationBoundary()
  disposeUserMutationBoundary = installUserMutationBoundary()

  try {
    let nextRuntime
    if (typeof props.factory === 'function') {
      nextRuntime = await initializeExperimentalRustRuntime(
        { markdown: runtimeMarkdown },
        {
          factory: props.factory,
          domContainer: rootRef.value,
          captureInput: true,
          applyPatches: scheduleMarkdownSync,
          onFileDrop: props.onFileDrop,
          onUriDrop: props.onUriDrop,
          onImageClick: props.onImageClick
        },
        reportError
      )
    } else {
      const muya = new StableCompleteMuyaWithRustCore(rootRef.value, {
        markdown: runtimeMarkdown,
        t: (key) => key,
        onFileDrop: props.onFileDrop,
        onUriDrop: props.onUriDrop,
        onImageClick: props.onImageClick
      })
      // The compatibility adapter starts the Rust session asynchronously in its
      // constructor. Do not expose the editor or let Vue reconcile a canonical
      // change until that session exists on the Tauri side.
      await muya.__rustMirror?.ready
      await muya.__rustCanonicalReady
      nextRuntime = {
        muya,
        bridge: createCompatBridge(muya),
        domContainer: muya.container,
        inputController: muya.keyboard,
        markUserMutation,
        destroy: () => {
          muya.destroy?.()
          muya.__rustMirror?.destroy?.()
          if (globalThis.__ELEPHANT_ACTIVE_MUYA__ === muya) delete globalThis.__ELEPHANT_ACTIVE_MUYA__
        }
      }
      globalThis.__ELEPHANT_ACTIVE_MUYA__ = muya
      muya.__onUserMutation = markUserMutation
      muya.on('change', (detail = {}) => {
        runtimeMarkdown = String(detail.markdown ?? muya.getMarkdown() ?? '')
        if (!userMutationObserved) {
          console.warn('[elephantnote:rust-editor] ignored Muya change before user mutation', {
            generation,
            markdownLength: runtimeMarkdown.length,
            revision: runtime?.bridge?.revision ?? null,
            reason: 'initial-rust-muya-synchronization'
          })
          return
        }
        // The parent receives this value from the active Rust/Muya instance.
        // Mark it before emitting so Vue does not interpret its own echo as an
        // external document replacement and remount the editor mid-selection.
        markInternalPropUpdate()
        emit('update:modelValue', runtimeMarkdown)
        emit('change', runtimeMarkdown)
      })
      muya.on('crashed', () => reportError(new Error('Muya JS/Rust editor crashed.')))
    }
    if (generation !== mountGeneration) {
      nextRuntime.destroy()
      return
    }
    runtime = nextRuntime
    console.info('[elephantnote:rust-editor] mount:ready', {
      generation,
      revision: runtime.bridge.revision,
      markdownLength: runtimeMarkdown.length,
      hasDomContainer: Boolean(runtime.domContainer),
      contentEditable: runtime.domContainer?.getAttribute?.('contenteditable') || null
    })
    emit('ready', runtime)
  } catch (error) {
    if (generation === mountGeneration) reportError(error)
  }
}

onMounted(() => {
  console.info('[elephantnote:rust-editor] component:mounted', {
    mode: props.mode,
    markdownLength: String(props.modelValue || '').length
  })
  void mountRuntime(props.modelValue)
})

watch(
  () => props.modelValue,
  (next, previous) => {
    const normalized = String(next || '')
    console.info('[elephantnote:rust-editor] model-value-change', {
      generation: mountGeneration,
      previousLength: String(previous || '').length,
      nextLength: normalized.length,
      runtimeLength: runtimeMarkdown.length,
      revision: runtime?.bridge?.revision ?? null
    })
    if (internalPropUpdatePending) {
      internalPropUpdatePending = false
      if (internalPropResetTimer) {
        window.clearTimeout(internalPropResetTimer)
        internalPropResetTimer = null
      }
      return
    }
    if (normalized !== runtimeMarkdown) void mountRuntime(normalized)
  }
)

watch(
  () => props.onFileDrop,
  (callback) => {
    if (runtime?.inputController) runtime.inputController.onFileDrop = callback || null
  }
)

watch(
  () => props.onUriDrop,
  (callback) => {
    if (runtime?.inputController) runtime.inputController.onUriDrop = callback || null
  }
)

watch(
  () => props.onImageClick,
  (callback) => {
    if (runtime?.inputController) runtime.inputController.onImageClick = callback || null
  }
)

onBeforeUnmount(() => {
  console.info('[elephantnote:rust-editor] component:before-unmount', {
    generation: mountGeneration,
    revision: runtime?.bridge?.revision ?? null,
    markdownLength: runtimeMarkdown.length
  })
  mountGeneration += 1
  destroyRuntime()
})
</script>

<style scoped>
.muya-rust-runtime-shell {
  position: relative;
  width: 100%;
  min-height: 100%;
}

.muya-rust-runtime-editor {
  min-height: 100%;
  outline: none;
  white-space: pre-wrap;
}

.muya-rust-runtime-error {
  position: absolute;
  inset: 16px 16px auto;
  padding: 12px 14px;
  border: 1px solid var(--errorColor, #c33);
  border-radius: 6px;
  background: var(--editorBgColor);
  color: var(--errorColor, #c33);
  font-size: 13px;
  white-space: normal;
}
</style>
