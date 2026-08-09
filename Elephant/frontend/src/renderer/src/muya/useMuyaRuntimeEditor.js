import { computed, onBeforeUnmount, onMounted, ref, shallowRef, watch } from 'vue'

import { createMuyaFullEditorRuntime } from './fullEditorRuntime.js'
import { isMuyaRuntimeActive, isMuyaRuntimeEnabled, readMuyaRuntimeMode } from './runtimeFlags.js'

export const useMuyaRuntimeEditor = ({ markdown = ref(''), mode = ref(readMuyaRuntimeMode()), documentRef = null } = {}) => {
  const rootRef = shallowRef(null)
  const runtimeRef = shallowRef(null)
  const ready = ref(false)
  const mounted = ref(false)
  const enabled = computed(() => isMuyaRuntimeEnabled(mode.value))
  const active = computed(() => isMuyaRuntimeActive(mode.value))
  const html = computed(() => runtimeRef.value?.html || '')
  const state = computed(() => runtimeRef.value?.state || null)

  const mount = () => {
    if (!rootRef.value || !enabled.value) return null
    runtimeRef.value = createMuyaFullEditorRuntime(rootRef.value, markdown.value || '', { document: documentRef?.value || globalThis.document })
    ready.value = true
    mounted.value = true
    return runtimeRef.value
  }

  const destroy = () => {
    runtimeRef.value?.live?.cancel?.()
    runtimeRef.value = null
    ready.value = false
    mounted.value = false
  }

  const setMarkdown = (next, group = 'external') => {
    markdown.value = next
    if (runtimeRef.value) runtimeRef.value.setMarkdown(next, group)
  }

  const syncFromRuntime = () => {
    if (!runtimeRef.value) return markdown.value
    runtimeRef.value.renderLiveNow?.('input')
    markdown.value = runtimeRef.value.markdown
    return markdown.value
  }

  const scheduleLiveRender = () => {
    runtimeRef.value?.scheduleLiveRender?.()
  }

  onMounted(mount)
  onBeforeUnmount(destroy)

  watch(markdown, (next) => {
    if (!runtimeRef.value || runtimeRef.value.markdown === next) return
    runtimeRef.value.setMarkdown(next || '', 'external')
  })

  watch(mode, () => {
    if (enabled.value && rootRef.value && !runtimeRef.value) mount()
    if (!enabled.value && runtimeRef.value) destroy()
  })

  return {
    rootRef,
    runtimeRef,
    ready,
    mounted,
    enabled,
    active,
    html,
    state,
    mount,
    destroy,
    setMarkdown,
    syncFromRuntime,
    scheduleLiveRender
  }
}
