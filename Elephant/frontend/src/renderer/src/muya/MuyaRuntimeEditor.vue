<template>
  <div class="muya-runtime-shell" :data-muya-runtime-mode="mode">
    <div
      ref="rootRef"
      class="muya-runtime-editor"
      data-testid="muya-runtime-editor"
      @input="handleInput"
      @keydown="handleKeydown"
      @paste="handlePaste"
    />
  </div>
</template>

<script setup>
import { computed, toRef, watch } from 'vue'

import { handleMuyaKeydown } from './inputRulesRuntime.js'
import { useMuyaRuntimeEditor } from './useMuyaRuntimeEditor.js'

const props = defineProps({
  modelValue: { type: String, default: '' },
  mode: { type: String, default: 'active' }
})

const emit = defineEmits(['update:modelValue', 'ready', 'change'])

const markdown = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const mode = toRef(props, 'mode')
const runtime = useMuyaRuntimeEditor({ markdown, mode })
const { rootRef, runtimeRef, ready } = runtime
let inputSyncTimer = null
const LIVE_INPUT_DEBOUNCE_MS = 16

const syncAndEmit = () => {
  const next = runtime.syncFromRuntime()
  emit('change', next)
}

const handleInput = () => {
  if (inputSyncTimer) clearTimeout(inputSyncTimer)
  inputSyncTimer = setTimeout(() => {
    inputSyncTimer = null
    syncAndEmit()
  }, LIVE_INPUT_DEBOUNCE_MS)
}

const handleKeydown = (event) => {
  if (handleMuyaKeydown(runtimeRef.value, event)) syncAndEmit()
}

const handlePaste = (event) => {
  if (!runtimeRef.value) return
  const html = event.clipboardData?.getData('text/html') || ''
  const text = event.clipboardData?.getData('text/plain') || ''
  if (!html && !text) return
  event.preventDefault()
  runtimeRef.value.pasteClipboard({ html, text })
  syncAndEmit()
}

watch(ready, (value) => {
  if (value) emit('ready', runtimeRef.value)
}, { immediate: true })
</script>

<style scoped>
.muya-runtime-shell {
  width: 100%;
  height: 100%;
}

.muya-runtime-editor {
  min-height: 100%;
  outline: none;
  white-space: pre-wrap;
}
</style>
