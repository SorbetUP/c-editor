<template>
  <div class="editor-container">
    <div class="editor-middle elephantnote-middle">
      <div
        v-if="!init"
        class="editor-placeholder"
      />
      <app-shell v-if="init" />
      <command-palette />
      <about-dialog />
      <export-setting-dialog />
      <rename />
      <tweet />
      <import-modal />
    </div>
  </div>
</template>

<script setup>
import { watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { useMainStore } from '@/store'
import { storeToRefs } from 'pinia'
import { addStyles, addThemeStyle, addCustomStyle } from '@/util/theme'
import AboutDialog from '@/components/about'
import CommandPalette from '@/components/commandPalette'
import ExportSettingDialog from '@/components/exportSettings'
import Rename from '@/components/rename'
import Tweet from '@/components/tweet'
import ImportModal from '@/components/import'
import bus from '@/bus'
import { DEFAULT_STYLE } from '@/config'
import { useTweetStore } from '@/store/tweet'
import { useLayoutStore } from '@/store/layout'
import { useListenForMainStore } from '@/store/listenForMain'
import { usePreferencesStore } from '@/store/preferences'
import { useEditorStore } from '@/store/editor'
import { useCommandCenterStore } from '@/store/commandCenter'
import { useProjectStore } from '@/store/project'
import { useAutoUpdatesStore } from '@/store/autoUpdates'
import { useNotificationStore } from '@/store/notification'
import AppShell from 'elephant-front/components/shell/AppShell.vue'

const isTauriRuntime = Boolean(window.__TAURI__ || window.__MARKTEXT_RUNTIME__)
const mainStore = useMainStore()
const editorStore = useEditorStore()
const preferencesStore = usePreferencesStore()
const layoutStore = useLayoutStore()
const projectStore = useProjectStore()
const tweetStore = useTweetStore()
const listenForMainStore = useListenForMainStore()
const autoUpdateStore = useAutoUpdatesStore()
const commandCenterStore = useCommandCenterStore()
const notificationStore = useNotificationStore()

let importDialogHideTimer = null
let dragOverHandler = null

const { init } = storeToRefs(mainStore)
const { theme, customCss, zoom } = storeToRefs(preferencesStore)

watch(theme, (value, oldValue) => {
  if (value !== oldValue) addThemeStyle(value)
})

watch(customCss, (value, oldValue) => {
  if (value !== oldValue) addCustomStyle({ customCss: value })
})

watch(zoom, (zoomValue) => {
  bus.emit('mt::window-zoom', zoomValue)
})

const hideImportDialogSoon = () => {
  if (importDialogHideTimer) window.clearTimeout(importDialogHideTimer)
  importDialogHideTimer = window.setTimeout(() => {
    importDialogHideTimer = null
    bus.emit('importDialog', false)
  }, 300)
}

const handleDragOver = (event) => {
  const dataTransfer = event.dataTransfer
  if (!dataTransfer?.types?.length) return

  if (dataTransfer.types.includes('Files')) {
    const isSingleImage = dataTransfer.items.length === 1 && dataTransfer.items[0].type.includes('image')
    if (!isSingleImage) {
      event.preventDefault()
      hideImportDialogSoon()
      bus.emit('importDialog', true)
    }
    dataTransfer.dropEffect = 'copy'
  } else {
    event.stopPropagation()
    dataTransfer.dropEffect = 'none'
  }
}

const setupDragDropHandler = () => {
  if (dragOverHandler) return
  dragOverHandler = handleDragOver
  window.addEventListener('dragover', dragOverHandler, false)
}

const cleanupDragDropHandler = () => {
  if (dragOverHandler) {
    window.removeEventListener('dragover', dragOverHandler, false)
    dragOverHandler = null
  }
  if (importDialogHideTimer) {
    window.clearTimeout(importDialogHideTimer)
    importDialogHideTimer = null
  }
}

onMounted(async () => {
  if (global.marktext.initialState) {
    preferencesStore.SET_USER_PREFERENCE(global.marktext.initialState)
  }

  mainStore.LISTEN_WIN_STATUS()
  await commandCenterStore.LISTEN_COMMAND_CENTER_BUS()
  tweetStore.LISTEN_FOR_TWEET()
  layoutStore.LISTEN_FOR_LAYOUT()
  listenForMainStore.LISTEN_FOR_EDIT()
  preferencesStore.LISTEN_FOR_VIEW()
  listenForMainStore.LISTEN_FOR_SHOW_DIALOG()
  listenForMainStore.LISTEN_FOR_PARAGRAPH_INLINE_STYLE()
  projectStore.LISTEN_FOR_UPDATE_PROJECT()
  projectStore.LISTEN_FOR_LOAD_PROJECT()
  projectStore.LISTEN_FOR_SIDEBAR_CONTEXT_MENU()
  autoUpdateStore.LISTEN_FOR_UPDATE()
  preferencesStore.ASK_FOR_USER_PREFERENCE()
  preferencesStore.LISTEN_TOGGLE_VIEW()
  editorStore.LISTEN_SCREEN_SHOT()
  editorStore.LISTEN_FOR_CLOSE()
  editorStore.LISTEN_FOR_SAVE_AS()
  editorStore.LISTEN_FOR_MOVE_TO()
  editorStore.LISTEN_FOR_SAVE()
  editorStore.LISTEN_FOR_SET_PATHNAME()
  editorStore.LISTEN_FOR_BOOTSTRAP_WINDOW()
  editorStore.LISTEN_FOR_SAVE_CLOSE()
  editorStore.LISTEN_FOR_RENAME()
  editorStore.LINTEN_FOR_SET_LINE_ENDING()
  editorStore.LINTEN_FOR_SET_ENCODING()
  editorStore.LINTEN_FOR_SET_FINAL_NEWLINE()
  editorStore.LISTEN_FOR_NEW_TAB()
  editorStore.LISTEN_FOR_CLOSE_TAB()
  editorStore.LISTEN_FOR_TAB_CYCLE()
  editorStore.LISTEN_FOR_SWITCH_TABS()
  editorStore.LINTEN_FOR_PRINT_SERVICE_CLEARUP()
  editorStore.LINTEN_FOR_EXPORT_SUCCESS()
  editorStore.LISTEN_FOR_FILE_CHANGE()
  editorStore.LISTEN_FOR_RELOAD_IMAGES()
  editorStore.LISTEN_FOR_CONTEXT_MENU()
  editorStore.LISTEN_FOR_STATE_REPLACE()
  notificationStore.listenForNotification()

  setupDragDropHandler()

  if (isTauriRuntime) {
    window.setTimeout(() => {
      if (!mainStore.init) mainStore.SET_INITIALIZED()
    }, 250)
  }

  nextTick(() => {
    addStyles(global.marktext.initialState || DEFAULT_STYLE)
  })
})

onBeforeUnmount(cleanupDragDropHandler)
</script>

<style scoped>
.editor-placeholder,
.editor-container {
  display: flex;
  flex-direction: row;
  position: absolute;
  width: 100vw;
  height: 100vh;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
}

.editor-container .hide {
  z-index: -1;
  opacity: 0;
  position: absolute;
  left: -10000px;
}

.editor-placeholder {
  background: var(--editorBgColor);
}

.editor-middle {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 100vh;
  position: relative;

  & > .editor {
    flex: 1;
  }
}
</style>
