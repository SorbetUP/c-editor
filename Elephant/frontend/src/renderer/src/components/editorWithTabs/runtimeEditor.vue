<template>
  <RustMuyaRuntimeEditor
    v-if="!sourceCode"
    :model-value="toEditorMarkdown(markdown)"
    :factory="rustRuntimeFactory"
    :on-file-drop="imageHandlers.dropped"
    :on-uri-drop="imageHandlers.uriDropped"
    :on-image-click="imageToolbar.open"
    mode="rust"
    class="rust-editor-runtime"
    @ready="handleRustRuntimeReady"
    @update:model-value="handleRustMarkdownChange"
  />
  <RuntimeTableDialog
    v-model="tableDialogVisible"
    @confirm="handleCreateTable"
  />
  <RuntimeImageToolbar
    :image="imageToolbar.state.active"
    @apply="imageToolbar.apply"
    @choose-file="imageToolbar.chooseFile"
    @delete="imageToolbar.remove"
    @close="imageToolbar.close"
  />
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'

import bus from '@/bus'
import { useEditorStore } from '@/store/editor'
import { usePreferencesStore } from '@/store/preferences'
import { useProjectStore } from '@/store/project'
import { debouncedSendBufferedState } from '@/store/bufferedState'
import { RustMuyaRuntimeEditor } from '@/muya'
import { createRustEditorRuntimeBinding } from '@/muya/editorRuntimeResource'
import RuntimeImageToolbar from './runtimeImageToolbar.vue'
import RuntimeTableDialog from './runtimeTableDialog.vue'
import { rustBusCommand } from './runtimeEditorCommands'
import { createRuntimeImageHandlers } from './runtimeEditorImages'
import { useRuntimeImageToolbar } from './runtimeImageToolbarState'
import { applyRustEditorMarkdown } from './runtimeEditorState'

const props = defineProps({
  markdown: { type: String, required: true },
  cursor: { type: Object, required: true },
  sourceCode: { type: Boolean, required: true },
  textDirection: { type: String, required: true },
  platform: { type: String, required: true },
  toEditorMarkdown: { type: Function, default: (markdown) => markdown },
  fromEditorMarkdown: { type: Function, default: (markdown) => markdown },
  rustRuntimeFactory: { type: Function, default: null }
})

const editorStore = useEditorStore()
const preferencesStore = usePreferencesStore()
const projectStore = useProjectStore()
const { currentFile } = storeToRefs(editorStore)
const { projectTree } = storeToRefs(projectStore)
const sourceCode = computed(() => props.sourceCode)
const rustRuntime = ref(null)
const tableDialogVisible = ref(false)
let editorRuntimeBinding = null
let disposeEditorRuntimeResource = null

const unpublishEditorRuntime = () => {
  disposeEditorRuntimeResource?.()
  disposeEditorRuntimeResource = null
  editorRuntimeBinding?.dispose()
  editorRuntimeBinding = null
}

const publishEditorRuntime = () => {
  if (!editorRuntimeBinding || disposeEditorRuntimeResource) return
  const host = globalThis.__ELEPHANT_ADDON_HOST__
  if (typeof host?.provide === 'function') {
    disposeEditorRuntimeResource = host.provide('editor.runtime', editorRuntimeBinding.resource)
  }
}

const handleRustRuntimeReady = (runtime) => {
  unpublishEditorRuntime()
  rustRuntime.value = runtime
  editorRuntimeBinding = createRustEditorRuntimeBinding({
    runtime,
    getMarkdown: () => currentFile.value?.markdown ?? props.markdown
  })
  publishEditorRuntime()
  editorRuntimeBinding.notify({ reason: 'ready' })
}

const dispatchRustBusCommand = (event, payload) => {
  if (props.sourceCode || !rustRuntime.value) return Promise.resolve(false)
  const command = rustBusCommand(event, payload)
  if (!command) return Promise.resolve(false)
  const result = rustRuntime.value.bridge.dispatch(command)
  result.catch((error) => console.error(`[Elephant Rust Editor] Failed to handle ${event}.`, error))
  return result
}

const imageHandlers = createRuntimeImageHandlers({
  currentFile,
  projectTree,
  preferencesStore,
  sourceCode,
  editorStore,
  dispatch: dispatchRustBusCommand
})
const imageToolbar = useRuntimeImageToolbar(imageHandlers)

const handleParagraphCommand = (type) => {
  if (type === 'table' && !props.sourceCode) {
    tableDialogVisible.value = true
    return
  }
  return dispatchRustBusCommand('paragraph', type)
}

const handleCreateTable = (table) => dispatchRustBusCommand('createTable', table)
const busHandlers = Object.freeze({
  undo: () => dispatchRustBusCommand('undo'),
  redo: () => dispatchRustBusCommand('redo'),
  format: (type) => dispatchRustBusCommand('format', type),
  paragraph: handleParagraphCommand,
  duplicate: () => dispatchRustBusCommand('duplicate'),
  deleteParagraph: () => dispatchRustBusCommand('deleteParagraph'),
  insertParagraph: () => dispatchRustBusCommand('insertParagraph'),
  createParagraph: () => dispatchRustBusCommand('createParagraph'),
  'insert-horizontal-rule': () => dispatchRustBusCommand('insert-horizontal-rule'),
  'insert-image': imageHandlers.insert,
  'image-uploaded': imageHandlers.uploaded
})

const handleRustMarkdownChange = (editorMarkdown) => {
  applyRustEditorMarkdown({
    editorStore,
    file: currentFile.value,
    editorMarkdown,
    fromEditorMarkdown: props.fromEditorMarkdown,
    persist: debouncedSendBufferedState
  })
  editorRuntimeBinding?.notify({
    reason: 'document-change',
    markdown: props.fromEditorMarkdown(String(editorMarkdown || '')),
    editorMarkdown: String(editorMarkdown || '')
  })
}

onMounted(() => {
  for (const [event, handler] of Object.entries(busHandlers)) bus.on(event, handler)
  globalThis.addEventListener?.('elephantnote:addons-ready', publishEditorRuntime)
  publishEditorRuntime()
})

onBeforeUnmount(() => {
  for (const [event, handler] of Object.entries(busHandlers)) bus.off(event, handler)
  globalThis.removeEventListener?.('elephantnote:addons-ready', publishEditorRuntime)
  unpublishEditorRuntime()
  rustRuntime.value = null
  tableDialogVisible.value = false
  imageToolbar.close()
})
</script>

<style scoped>
.rust-editor-runtime {
  height: 100%;
  min-height: 0;
  overflow: auto;
  padding: 24px var(--en-note-editor-gutter-right, 24px) 80px
    var(--en-note-editor-gutter-left, 32px);
  background: var(--editorBgColor);
}
</style>
