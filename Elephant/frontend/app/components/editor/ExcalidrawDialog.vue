<template>
  <Teleport to="body">
    <div
      class="en-excalidraw-overlay"
      :style="themeTokens"
    >
      <section
        class="en-excalidraw-shell"
        data-testid="excalidraw-dialog"
        role="dialog"
        aria-modal="true"
        :aria-label="t('excalidraw.title')"
      >
        <header class="en-excalidraw-header">
          <div class="en-excalidraw-name-wrap">
            <input
              v-model="editableBaseName"
              type="text"
              class="en-excalidraw-name-input"
              spellcheck="false"
              :placeholder="t('excalidraw.drawingPlaceholder')"
              :aria-label="t('excalidraw.drawingName')"
              @input="markNameEdited"
              @pointerdown.stop
              @pointerup.stop
              @mousedown.stop
              @mouseup.stop
              @click.stop
              @keydown.stop
            >
          </div>
        </header>

        <div class="en-excalidraw-actions">
          <button
            type="button"
            class="en-excalidraw-button secondary"
            data-testid="excalidraw-close"
            :aria-label="t('excalidraw.cancel')"
            :title="`${t('excalidraw.cancel')} · Esc`"
            @pointerdown.stop
            @pointerup.stop.prevent="handleClose"
            @mousedown.stop
            @mouseup.stop.prevent="handleClose"
            @click.stop.prevent="handleClose"
          >
            <X
              :size="16"
              aria-hidden="true"
            />
          </button>
          <button
            type="button"
            class="en-excalidraw-button primary"
            data-testid="excalidraw-save"
            :disabled="isSaving || !apiRef || !!errorMessage"
            :aria-label="t('excalidraw.save')"
            :title="`${t('excalidraw.save')} · ${isMacOS ? '⌘S' : 'Ctrl S'}`"
            @pointerdown.stop
            @pointerup.stop.prevent="handleSave"
            @mousedown.stop
            @mouseup.stop.prevent="handleSave"
            @click.stop.prevent="handleSave"
          >
            <span
              v-if="isSaving"
              aria-hidden="true"
            >…</span>
            <Check
              v-else
              :size="16"
              aria-hidden="true"
            />
          </button>
        </div>

        <div
          v-if="isNamePromptOpen"
          class="en-excalidraw-name-prompt-backdrop"
          data-testid="excalidraw-name-prompt"
          role="presentation"
          @pointerdown.stop
          @click.stop
        >
          <form
            class="en-excalidraw-name-prompt"
            role="alertdialog"
            aria-modal="true"
            :aria-label="t('excalidraw.namePromptTitle')"
            @submit.prevent="confirmNameAndSave"
          >
            <h2>{{ t('excalidraw.namePromptTitle') }}</h2>
            <p>{{ t('excalidraw.namePromptDescription') }}</p>
            <input
              ref="namePromptInput"
              v-model.trim="promptName"
              type="text"
              :placeholder="t('excalidraw.drawingPlaceholder')"
              :aria-label="t('excalidraw.drawingName')"
              autofocus
            >
            <p
              v-if="promptError"
              class="en-excalidraw-name-prompt-error"
              role="alert"
            >
              {{ promptError }}
            </p>
            <div class="en-excalidraw-name-prompt-actions">
              <button
                type="button"
                class="en-excalidraw-button secondary"
                @click="cancelNamePrompt"
              >
                {{ t('common.cancel') }}
              </button>
              <button
                type="submit"
                class="en-excalidraw-button primary"
                :disabled="!promptName"
              >
                {{ t('common.save') }}
              </button>
            </div>
          </form>
        </div>

        <div
          v-if="errorMessage"
          class="en-excalidraw-error"
          role="alert"
        >
          <strong>{{ t('excalidraw.failedTitle') }}</strong>
          <p>{{ errorMessage }}</p>
        </div>

        <main
          v-show="!errorMessage"
          ref="mountEl"
          class="en-excalidraw-canvas"
        />
      </section>
    </div>
  </Teleport>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import React from 'react'
import { createRoot } from 'react-dom/client'
import { Check, X } from '@lucide/vue'
import { getThemeMode, getThemeTokens } from 'common/elephantnote/appearance'
import { getExcalidrawBackgroundColor } from 'elephant-shared/excalidrawAssets'
import {
  loadExcalidrawModule,
  createInitialExcalidrawData,
  exportExcalidrawBlob,
  exportExcalidrawSceneBlob,
  ensurePngName
} from '../../services/excalidraw'

const props = defineProps({
  title: {
    type: String,
    default: 'Excalidraw'
  },
  theme: {
    type: String,
    default: 'light'
  },
  fileName: {
    type: String,
    default: 'excalidraw.png'
  },
  initialBlob: {
    type: Blob,
    default: null
  },
  saveMode: {
    type: String,
    default: 'png'
  },
  insertOnSave: {
    type: Boolean,
    default: false
  },
  askNameOnClose: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['close', 'save'])
const { t } = useI18n()
const mountEl = ref(null)
const apiRef = shallowRef(null)
const root = shallowRef(null)
const excalidrawModule = shallowRef(null)
const isSaving = ref(false)
const initialData = shallowRef(null)
const errorMessage = ref('')
const isNamePromptOpen = ref(false)
const promptName = ref('')
const promptError = ref('')
const namePromptInput = ref(null)
const nameWasEdited = ref(false)
const isMacOS = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(`${navigator.platform || ''} ${navigator.userAgent || ''}`)

const logDialogError = (event, error) => {
  const details = {
    name: error?.name || 'Error',
    message: error?.message || String(error),
    stack: error?.stack || ''
  }
  window.__ELEPHANT_DEBUG_LOGS__ = Array.isArray(window.__ELEPHANT_DEBUG_LOGS__)
    ? window.__ELEPHANT_DEBUG_LOGS__
    : []
  window.__ELEPHANT_DEBUG_LOGS__.push({
    at: new Date().toISOString(),
    level: 'error',
    message: `[excalidraw-dialog] ${event}`,
    details
  })
  if (window.__ELEPHANT_DEBUG_LOGS__.length > 1000) {
    window.__ELEPHANT_DEBUG_LOGS__.splice(0, window.__ELEPHANT_DEBUG_LOGS__.length - 1000)
  }
  console.error(`[excalidraw-dialog] ${event}`, details)
}

// Elephant themes expose full palettes while Excalidraw accepts only its
// canonical light/dark modes. Keep the surrounding shell on the full palette.
const excalidrawTheme = computed(() => getThemeMode(props.theme))
const themeTokens = computed(() => getThemeTokens(props.theme))

const stripKnownExtensions = (value) => {
  return String(value || '')
    .replace(/\.excalidraw\.png$/i, '')
    .replace(/\.excalidraw$/i, '')
    .replace(/\.png$/i, '')
}

const blobToBytes = async(blob) => new Uint8Array(await blob.arrayBuffer())

const editableBaseName = ref(stripKnownExtensions(props.fileName) || 'drawing')
const normalizedBaseName = computed(() => {
  const cleaned = stripKnownExtensions(editableBaseName.value).trim()
  return cleaned || 'drawing'
})
const resolvedFileName = computed(() => ensurePngName(normalizedBaseName.value))

const handleClose = () => {
  if (!props.askNameOnClose) {
    emit('close')
    return
  }
  if (nameWasEdited.value) {
    void handleSave()
    return
  }
  promptError.value = ''
  promptName.value = ''
  isNamePromptOpen.value = true
  void nextTick(() => namePromptInput.value?.focus())
}

const markNameEdited = () => {
  nameWasEdited.value = true
}

const cancelNamePrompt = () => {
  isNamePromptOpen.value = false
  emit('close')
}

const confirmNameAndSave = () => {
  const nextName = promptName.value.trim()
  if (!nextName) {
    promptError.value = t('excalidraw.namePromptRequired')
    return
  }
  editableBaseName.value = nextName
  nameWasEdited.value = true
  isNamePromptOpen.value = false
  void handleSave()
}

const renderExcalidraw = () => {
  if (!root.value || !excalidrawModule.value) return
  root.value.render(
    React.createElement(excalidrawModule.value.Excalidraw, {
      initialData: initialData.value,
      theme: excalidrawTheme.value,
      name: normalizedBaseName.value,
      excalidrawAPI: (api) => {
        if (api && apiRef.value !== api) apiRef.value = api
      },
      UIOptions: {
        canvasActions: {
          saveToActiveFile: false,
          loadScene: false,
          export: false,
          clearCanvas: true,
          toggleTheme: false
        }
      }
    })
  )
}

const applyExcalidrawTheme = (theme) => {
  const api = apiRef.value
  if (!api?.updateScene) return
  const currentAppState = api.getAppState?.() || {}
  const viewBackgroundColor = getExcalidrawBackgroundColor(theme)
  if (currentAppState.theme === theme && currentAppState.viewBackgroundColor === viewBackgroundColor) return
  api.updateScene({
    appState: {
      ...currentAppState,
      theme,
      viewBackgroundColor
    }
  })
}

const renderCanvas = async () => {
  excalidrawModule.value = await loadExcalidrawModule()
  initialData.value = await createInitialExcalidrawData({
    blob: props.initialBlob,
    fileName: props.fileName,
    theme: excalidrawTheme.value
  })

  if (!mountEl.value) throw new Error('Excalidraw mount element is missing.')
  root.value = createRoot(mountEl.value)
  renderExcalidraw()
}

watch(excalidrawTheme, (theme) => {
  applyExcalidrawTheme(theme)
}, { flush: 'post' })

const handleSave = async () => {
  if (!apiRef.value || isSaving.value) return
  if (props.askNameOnClose && !nameWasEdited.value) {
    promptError.value = ''
    promptName.value = ''
    isNamePromptOpen.value = true
    void nextTick(() => namePromptInput.value?.focus())
    return
  }
  isSaving.value = true
  errorMessage.value = ''
  const reportSaveError = (error) => {
    logDialogError('save failed', error)
    errorMessage.value = error?.message || t('excalidraw.failedSave')
  }
  try {
    const sceneBlob = await exportExcalidrawSceneBlob({
      api: apiRef.value,
      theme: excalidrawTheme.value
    })
    const blob = await exportExcalidrawBlob({
      api: apiRef.value,
      theme: excalidrawTheme.value
    })
    emit('save', {
      blob,
      imageBlob: await blobToBytes(blob),
      fileName: resolvedFileName.value,
      baseName: normalizedBaseName.value,
      sceneBlob: await sceneBlob.text(),
      onError: reportSaveError
    })
  } catch (error) {
    logDialogError('save failed', error)
    errorMessage.value = error?.message || t('excalidraw.failedSave')
  } finally {
    isSaving.value = false
  }
}

const handleKeyboard = (event) => {
  if (event.key === 'Escape') {
    event.preventDefault()
    handleClose()
    return
  }
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
    event.preventDefault()
    void handleSave()
  }
}

onMounted(() => {
  document.body.classList.add('en-excalidraw-open')
  window.addEventListener('keydown', handleKeyboard, true)
  renderCanvas().catch((error) => {
    logDialogError('initialization failed', error)
    errorMessage.value = error?.message || t('excalidraw.failedInitialize')
  })
})

onBeforeUnmount(() => {
  document.body.classList.remove('en-excalidraw-open')
  window.removeEventListener('keydown', handleKeyboard, true)
  root.value?.unmount?.()
})
</script>

<style scoped>
.en-excalidraw-overlay {
  position: fixed;
  inset: 0;
  z-index: 5000;
  background: var(--en-bg, #0f172a);
  -webkit-app-region: no-drag;
}

.en-excalidraw-shell {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--en-bg, #0f172a);
  color: var(--en-text, #eef2ff);
  position: relative;
  padding-top: 28px;
  -webkit-app-region: no-drag;
}

.en-excalidraw-header {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 8px;
  padding: 0 8px 0 86px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.08);
  background: color-mix(in srgb, var(--en-bg, #0f172a) 94%, transparent);
  backdrop-filter: blur(16px);
  z-index: 1000;
}

.en-excalidraw-name-wrap {
  flex: 1;
  max-width: 420px;
}

.en-excalidraw-name-input {
  width: 100%;
  height: 20px;
  border: 0;
  outline: 0;
  border-radius: 4px;
  background: rgba(148, 163, 184, 0.12);
  color: inherit;
  font: inherit;
  font-size: 12px;
  padding: 0 8px;
}

.en-excalidraw-actions {
  position: absolute;
  top: 16px;
  /* Keep the shell actions in the same row as Excalidraw's Library control,
   * with enough horizontal space that neither control covers the other. */
  right: 176px;
  z-index: 1002;
  display: flex;
  align-items: center;
  gap: 6px;
}

.en-excalidraw-name-prompt-backdrop {
  position: absolute;
  inset: 28px 0 0;
  z-index: 1001;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgb(0 0 0 / 42%);
}

.en-excalidraw-name-prompt {
  width: min(420px, 100%);
  display: grid;
  gap: 14px;
  padding: 24px;
  border: 1px solid var(--en-border-strong, var(--en-border));
  border-radius: 16px;
  color: var(--en-text);
  background: var(--en-surface, #242424);
  box-shadow: 0 24px 64px rgb(0 0 0 / 34%);
}

.en-excalidraw-name-prompt h2,
.en-excalidraw-name-prompt p {
  margin: 0;
}

.en-excalidraw-name-prompt h2 {
  font-size: 18px;
}

.en-excalidraw-name-prompt p {
  color: var(--en-muted);
  font-size: 13px;
  line-height: 1.45;
}

.en-excalidraw-name-prompt input {
  width: 100%;
  min-height: 40px;
  box-sizing: border-box;
  padding: 0 12px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  color: var(--en-text);
  background: var(--en-bg);
  font: inherit;
}

.en-excalidraw-name-prompt-error {
  color: var(--en-danger, #ef4444) !important;
}

.en-excalidraw-name-prompt-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.en-excalidraw-name-prompt-actions .en-excalidraw-button {
  width: auto;
  height: 34px;
  min-width: 84px;
  padding: 0 14px;
  border-radius: 8px;
}

.en-excalidraw-button {
  width: 22px;
  height: 22px;
  border-radius: 999px;
  border: 1px solid rgba(148, 163, 184, 0.2);
  background: rgba(148, 163, 184, 0.12);
  color: inherit;
  cursor: pointer;
  line-height: 1;
}

.en-excalidraw-button.primary {
  background: var(--en-primary, #2563eb);
  color: white;
  border-color: var(--en-primary, #2563eb);
}

.en-excalidraw-button:disabled {
  opacity: 0.5;
  cursor: wait;
}

.en-excalidraw-error {
  margin: 96px auto;
  max-width: 520px;
  border-radius: 16px;
  padding: 24px;
  background: color-mix(in srgb, var(--en-danger, #ef4444) 12%, transparent);
  color: var(--en-text, #fecaca);
}

.en-excalidraw-canvas {
  flex: 1;
  min-height: 0;
  height: calc(100vh - 28px);
  background: #fff;
}
</style>
