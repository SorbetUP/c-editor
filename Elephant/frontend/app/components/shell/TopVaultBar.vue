<template>
  <header
    class="en-topstrip"
    :class="{ 'en-topstrip-sidebar-hidden': !sidebarVisible }"
    data-tauri-drag-region
  >
    <navigation-bar
      class="en-topstrip-nav"
      :class="{ 'en-topstrip-nav-macos': isMac }"
    />
    <div
      class="en-topstrip-drag"
      data-tauri-drag-region
      @dblclick="handleMaximizeClick"
    />
    <div
      v-if="!isMac"
      class="en-window-controls"
      aria-label="Window controls"
    >
      <button
        class="en-window-control"
        type="button"
        title="Minimize"
        aria-label="Minimize window"
        @click.stop="handleMinimizeClick"
      >
        <Minus class="en-window-control-icon" />
      </button>
      <button
        class="en-window-control"
        type="button"
        :title="isMaximized ? 'Restore' : 'Maximize'"
        :aria-label="isMaximized ? 'Restore window' : 'Maximize window'"
        @click.stop="handleMaximizeClick"
      >
        <Copy v-if="isMaximized" class="en-window-control-icon en-window-control-restore" />
        <Square v-else class="en-window-control-icon" />
      </button>
      <button
        class="en-window-control en-window-control-close"
        type="button"
        title="Close"
        aria-label="Close window"
        @click.stop="handleCloseClick"
      >
        <X class="en-window-control-icon" />
      </button>
    </div>
  </header>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { Copy, Minus, Square, X } from '@lucide/vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import NavigationBar from '../navigation/NavigationBar.vue'

defineProps({
  sidebarVisible: {
    type: Boolean,
    default: true
  }
})

const platform = globalThis.navigator?.platform || ''
const userAgent = globalThis.navigator?.userAgent || ''
const isMac = platform.startsWith('Mac') || /mac/i.test(userAgent)
const isMaximized = ref(false)
let unlistenResize = null

const syncWindowState = async () => {
  if (isMac) return
  try {
    isMaximized.value = await getCurrentWindow().isMaximized()
  } catch (error) {
    console.warn('[window-controls] unable to read maximized state', error)
  }
}

const handleMinimizeClick = async () => {
  try {
    await getCurrentWindow().minimize()
  } catch (error) {
    console.warn('[window-controls] minimize failed', error)
  }
}

const handleMaximizeClick = async () => {
  if (isMac) return
  try {
    const win = getCurrentWindow()
    if (await win.isFullscreen()) {
      await win.setFullscreen(false)
    } else if (await win.isMaximized()) {
      await win.unmaximize()
    } else {
      await win.maximize()
    }
    await syncWindowState()
  } catch (error) {
    console.warn('[window-controls] maximize toggle failed', error)
  }
}

const handleCloseClick = async () => {
  try {
    await getCurrentWindow().close()
  } catch (error) {
    console.warn('[window-controls] close failed', error)
  }
}

onMounted(async () => {
  if (isMac) return
  await syncWindowState()
  try {
    unlistenResize = await getCurrentWindow().onResized(syncWindowState)
  } catch (error) {
    console.warn('[window-controls] resize listener unavailable', error)
  }
})

onBeforeUnmount(() => {
  unlistenResize?.()
})
</script>

<style scoped>
.en-topstrip {
  position: relative;
  height: 32px;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  -webkit-app-region: drag;
  background:
    linear-gradient(
      to right,
      var(--en-sidebar-bg, var(--en-bg)) 0,
      var(--en-sidebar-bg, var(--en-bg)) calc(48px + var(--en-sidebar-width) + 1px),
      var(--en-bg) calc(48px + var(--en-sidebar-width) + 1px),
      var(--en-bg) 100%
    );
}
.en-topstrip-sidebar-hidden { background: var(--en-bg); }
.en-topstrip::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: calc(48px + var(--en-sidebar-width));
  width: 1px;
  background: var(--en-border);
  z-index: 1;
  pointer-events: none;
}
.en-topstrip-sidebar-hidden::after { display: none; }
.en-topstrip-nav {
  position: absolute;
  left: 56px;
  top: 4px;
  z-index: 3;
  width: 76px;
  height: 24px;
  -webkit-app-region: no-drag;
  pointer-events: auto;
}
.en-topstrip-nav-macos {
  left: 84px;
  top: -8px;
}
.en-topstrip-drag {
  flex: 1;
  height: 100%;
  min-width: 0;
  position: relative;
  z-index: 0;
  margin-left: 180px;
  -webkit-app-region: drag;
}
.en-window-controls {
  position: relative;
  z-index: 4;
  align-self: stretch;
  display: flex;
  margin-left: auto;
  -webkit-app-region: no-drag;
  user-select: none;
}
.en-window-control {
  width: 46px;
  min-width: 46px;
  height: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 0;
  color: var(--en-muted);
  background: transparent;
  cursor: default;
  -webkit-app-region: no-drag;
}
.en-window-control:hover {
  color: var(--en-text);
  background: var(--en-soft);
}
.en-window-control:focus-visible {
  outline: 2px solid var(--en-primary);
  outline-offset: -2px;
}
.en-window-control-close:hover {
  color: #ffffff;
  background: #c42b1c;
}
.en-window-control-icon {
  width: 15px;
  height: 15px;
  stroke-width: 1.8;
  pointer-events: none;
}
.en-window-control-restore {
  width: 14px;
  height: 14px;
}
</style>
