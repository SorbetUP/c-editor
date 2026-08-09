<template>
  <Teleport to="body">
    <div class="en-excalidraw-overlay" :style="themeTokens">
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
              @pointerdown.stop
              @pointerup.stop
              @mousedown.stop
              @mouseup.stop
              @click.stop
              @keydown.stop
            >
          </div>

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
              ✕
            </button>
            <button
              type="button"
              class="en-excalidraw-button primary"
              :disabled="isSaving || !apiRef || !!errorMessage"
              :aria-label="t('excalidraw.save')"
              :title="`${t('excalidraw.save')} · ${isMacOS ? '⌘S' : 'Ctrl S'}`"
              @pointerdown.stop
              @pointerup.stop.prevent="handleSave"
              @mousedown.stop
              @mouseup.stop.prevent="handleSave"
              @click.stop.prevent="handleSave"
            >
              {{ isSaving ? '…' : '✓' }}
            </button>
          </div>
        </header>

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
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import React from 'react'
import { createRoot } from 'react-dom/client'
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
  }
})

const emit = defineEmits(['close', 'save'])
const { t } = useI18n()
const mountEl = ref(null)
const apiRef = ref(null)
const root = ref(null)
const excalidrawModule = ref(null)
const isSaving = ref(false)
const initialData = ref(null)
const errorMessage = ref('')
const isMacOS = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(`${navigator.platform || ''} ${navigator.userAgent || ''}`)

const logDialogError = (event, error) => {
  const details = {
    name: error?.name || 'Error',
    message: error?.message || String(error)
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
  emit('close')
}

const renderExcalidraw = () => {
  if (!root.value || !excalidrawModule.value) return
  root.value.render(
    React.createElement(excalidrawModule.value.Excalidraw, {
      initialData: initialData.value,
      theme: excalidrawTheme.value,
      name: normalizedBaseName.value,
      excalidrawAPI: (api) => {
        apiRef.value = api
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
  renderExcalidraw()
  const api = apiRef.value
  if (!api?.updateScene) return
  api.updateScene({
    appState: {
      ...api.getAppState?.(),
      theme,
      viewBackgroundColor: getExcalidrawBackgroundColor(theme)
    }
  })
}

const renderCanvas = async () => {
  excalidrawModule.value = await loadExcalidrawModule()
  initialData.value = await createInitialExcalidrawData({
    blob: props.initialBlob,
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
  isSaving.value = true
  errorMessage.value = ''
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
      sceneBlob: await sceneBlob.text()
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
  z-index: 4;
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
  display: flex;
  align-items: center;
  gap: 6px;
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
