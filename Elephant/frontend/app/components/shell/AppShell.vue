<template>
  <empty-vault-picker
    v-if="!store.hasVault"
    @create-local="createLocalVault"
    @choose="store.chooseVault"
  />
  <div
    v-else
    class="en-shell"
    :class="[
      `en-theme-${themeMode}`,
      `en-theme-${themeClassId}`,
      {
        'en-pinned-card-halo': preferences.pinnedCardHalo,
        'en-floating-surfaces': preferences.floatingSurfaces,
        'en-mobile-shell': isMobileShell,
        'en-mobile-drawer-open': isMobileShell && drawerProgress > 0,
        'en-mobile-drawer-dragging': isMobileShell && drawerDragging,
        'en-mobile-drawer-settling': isMobileShell && drawerSettling
      }
    ]"
    :style="shellStyle"
    @pointerdown="handleDrawerPointerDown"
    @pointermove="handleDrawerPointerMove"
    @pointerup="handleDrawerPointerEnd"
    @pointercancel="handleDrawerPointerCancel"
  >
    <div class="en-shell-main">
      <top-vault-bar
        v-if="!isMobileShell"
        :sidebar-visible="sidebarVisible"
      />
      <header
        v-else
        class="en-mobile-topbar"
      >
        <button
          class="en-mobile-icon-button"
          type="button"
          :aria-label="drawerProgress > 0 ? 'Close navigation' : 'Open navigation'"
          :title="drawerProgress > 0 ? 'Close navigation' : 'Open navigation'"
          @click="toggleMobileSidebar"
        >
          <Menu class="en-mobile-icon" />
        </button>
        <button
          class="en-mobile-search"
          type="button"
          @click="openSearch"
        >
          <Search class="en-mobile-icon" />
          <span>Search notes</span>
        </button>
        <button
          class="en-mobile-icon-button"
          type="button"
          aria-label="Settings"
          @click="openSettings"
        >
          <Settings class="en-mobile-icon" />
        </button>
      </header>
      <div class="en-layout">
        <icon-rail
          v-if="!isMobileShell"
          :active-addon-view-id="activeAddonViewId"
          :sidebar-visible="sidebarVisible"
          @open-settings="openSettings"
          @search="openSearch"
          @toggle-sidebar="toggleSidebar"
          @open-addon-view="openAddonView"
          @close-addon-view="closeAddonView"
        />
        <button
          v-if="isMobileShell"
          class="en-mobile-scrim"
          :class="{ visible: drawerProgress > 0 }"
          type="button"
          aria-label="Close navigation"
          :aria-hidden="drawerProgress <= 0"
          :tabindex="drawerProgress > 0 ? 0 : -1"
          @click="closeMobileSidebar"
        />
        <div
          class="en-body"
          :class="{ 'en-sidebar-hidden': !sidebarVisible }"
        >
          <sidebar-nav
            v-if="sidebarVisible || isMobileShell"
            :active-addon-view-id="activeAddonViewId"
            @search="openSearch"
            @open-addon-view="openAddonView"
            @close-addon-view="closeAddonView"
            @click="handleMobileSidebarClick"
          />
          <div
            v-if="sidebarVisible && !isMobileShell"
            class="en-sidebar-resizer"
            data-sidebar-resizer
            role="separator"
            aria-orientation="vertical"
            aria-label="Resize sidebar"
            tabindex="0"
            aria-valuemin="184"
            aria-valuemax="320"
            :aria-valuenow="sidebarWidth"
            @pointerdown="startResize"
            @keydown="handleResizeKeydown"
          />
          <main-content
            class="en-body-main"
            :active-addon-view-id="activeAddonViewId"
            @close-addon-view="closeAddonView"
          />
        </div>
      </div>
      <CreateEntryMenu
        v-if="isMobileShell && !store.openedNotePath"
        mobile
        :disabled="mobileCreateBusy"
        @select="handleMobileCreate"
      >
        <template #trigger="{ toggle, open, disabled }">
          <button
            class="en-mobile-fab"
            type="button"
            aria-label="Create"
            :aria-expanded="open"
            aria-haspopup="menu"
            :disabled="disabled"
            @click="toggle"
          >
            <Plus class="en-mobile-fab-icon" />
          </button>
        </template>
      </CreateEntryMenu>
    </div>
    <template
      v-for="entry in shellRightZones"
      :key="entry.contribution.id"
    >
      <component
        :is="entry.contribution.component"
        v-if="isLayoutZoneVisible(entry)"
      />
    </template>
    <search-modal />
    <settings-panel
      v-if="isSettingsOpen"
      :theme="theme"
      :vaults="store.vaults"
      :active-vault-name="store.activeVault?.name || 'No vault'"
      :active-vault-path="store.activeVault?.path || ''"
      :initial-section="settingsInitialSection"
      @close="isSettingsOpen = false"
      @update-theme="setTheme"
    />
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, provide, ref, watch } from 'vue'
import { Menu, Plus, Search, Settings } from '@lucide/vue'
import { useVaultStore } from '../../stores/vaultStore'
import { useNavigationStore } from '../../stores/navigationStore'
import { usePreferencesStore } from '@/store/preferences'
import { useAddonsStore } from '@/store/addons'
import { useEditorStore } from '@/store/editor'
import { useSearchStore } from '../../stores/searchStore'
import { useCanvasStore } from '../../stores/canvasStore'
import bus from '../../../src/renderer/src/bus'
import { elephantnoteClient } from '../../services/elephantnoteClient'
import {
  ELEPHANTNOTE_THEME_STORAGE_KEY,
  getThemeMode,
  getThemeTokens,
  normalizeThemeId
} from 'common/elephantnote/appearance'
import EmptyVaultPicker from './EmptyVaultPicker.vue'
import TopVaultBar from './TopVaultBar.vue'
import IconRail from '../navigation/IconRail.vue'
import SidebarNav from '../navigation/SidebarNav.vue'
import MainContent from './MainContent.vue'
import SettingsPanel from '../settings/SettingsPanel.vue'
import SearchModal from '../../search/SearchModal.vue'
import CreateEntryMenu from '../library/CreateEntryMenu.vue'
import { openNewDrawing } from '../library/createEntryActions'
import '../../styles/app-shell.css'
import '../../styles/app-shell-runtime-fixes.css'

const CORE_WORKSPACE_VIEWS = new Set(['notes', 'dashboard', 'canvas'])
const store = useVaultStore()
const preferences = usePreferencesStore()
const addonsStore = useAddonsStore()
const editorStore = useEditorStore()
const searchStore = useSearchStore()
const navigationStore = useNavigationStore()
const canvasStore = useCanvasStore()
const isSettingsOpen = ref(false)
const mobileCreateBusy = ref(false)
const settingsInitialSection = ref('appearance')
const activeAddonViewId = ref('')
const theme = ref(normalizeThemeId(window.localStorage.getItem(ELEPHANTNOTE_THEME_STORAGE_KEY)))
const sidebarWidth = ref(232)
const sidebarVisible = ref(true)
    const drawerProgress = ref(1)
    const drawerDragging = ref(false)
    const drawerSettling = ref(false)
    const isMobileShell = ref(false)
    let drawerGesture = null
    let drawerSettleTimer = null
    let sidebarResizeFrame = null
let pendingSidebarWidth = null

const activeThemeTokens = computed(() => getThemeTokens(theme.value))
const themeMode = computed(() => getThemeMode(theme.value))
const themeClassId = computed(() => theme.value.replace(/[^a-z0-9-]/gi, '-'))
const availableAddonViewIds = computed(() => addonsStore.getContributions('views')
  .map((entry) => entry?.contribution?.id)
  .filter(Boolean))
const shellRightZones = computed(() => {
  const seen = new Set()
  return addonsStore.getContributions('layout.zones')
    .filter((entry) => entry?.contribution?.zone === 'shell.right' && entry?.contribution?.component)
    .filter((entry) => {
      const id = entry?.contribution?.id || `${entry?.addonId || 'addon'}:shell.right`
      if (seen.has(id)) return false
      seen.add(id)
      return true
    })
    .sort((left, right) => Number(left.contribution.order || 0) - Number(right.contribution.order || 0))
})
const shellStyle = computed(() => ({
  ...activeThemeTokens.value,
  '--en-sidebar-width': `${sidebarWidth.value}px`,
  '--en-sidebar-runtime-width': `${sidebarWidth.value}px`,
  '--en-mobile-drawer-progress': String(drawerProgress.value)
}))

const isLayoutZoneVisible = (entry) => {
  const predicate = entry?.contribution?.when
  if (typeof predicate !== 'function') return true
  try {
    return predicate() === true
  } catch (error) {
    console.warn('[addons] layout visibility predicate failed', {
      id: entry?.contribution?.id || '',
      error
    })
    return false
  }
}

const applyThemeVariables = () => {
  const root = document.documentElement
  for (const [key, value] of Object.entries(activeThemeTokens.value)) {
    root.style.setProperty(key, value)
  }
  root.dataset.elephantnoteTheme = theme.value
}

const openSettings = (section = 'appearance') => {
  settingsInitialSection.value = typeof section === 'string' && section ? section : 'appearance'
  isSettingsOpen.value = true
}

const handleOpenSettingsEvent = (event) => {
  openSettings(event?.detail?.section || 'appearance')
}

const openSearch = () => {
  searchStore.open()
}

const openAddonView = (viewId) => {
  const normalized = typeof viewId === 'string' ? viewId.trim() : ''
  if (!normalized || !availableAddonViewIds.value.includes(normalized)) return
  store.closeNote()
  store.activeWorkspaceView = 'notes'
  activeAddonViewId.value = normalized
}

const handleOpenAddonViewEvent = (event) => {
  openAddonView(event?.detail?.viewId)
}

const closeAddonView = () => {
  activeAddonViewId.value = ''
}

const toggleSidebar = () => {
  sidebarVisible.value = !sidebarVisible.value
}

const clampDrawerProgress = (value) => Math.min(1, Math.max(0, Number(value) || 0))
const drawerWidth = () => Math.max(
  1,
  document.querySelector('.en-mobile-shell .en-sidebar')?.getBoundingClientRect?.().width ||
    Math.min(window.innerWidth * 0.86, 340)
)

const settleDrawer = (open) => {
  window.clearTimeout(drawerSettleTimer)
  drawerDragging.value = false
  drawerSettling.value = true
  drawerProgress.value = open ? 1 : 0
  sidebarVisible.value = open
  drawerSettleTimer = window.setTimeout(() => {
    drawerSettling.value = false
  }, 280)
}

const openMobileSidebar = () => {
  if (isMobileShell.value) settleDrawer(true)
}

const closeMobileSidebar = () => {
  if (isMobileShell.value) settleDrawer(false)
}

const toggleMobileSidebar = () => {
  if (drawerProgress.value > 0) closeMobileSidebar()
  else openMobileSidebar()
}

const handleDrawerPointerDown = (event) => {
  if (!isMobileShell.value || event.isPrimary === false || event.button > 0 || drawerGesture) return
  const open = drawerProgress.value > 0
  const insideDrawer = !!event.target?.closest?.('.en-sidebar')
  const onScrim = !!event.target?.closest?.('.en-mobile-scrim.visible')
  if (!open && event.clientX > 30) return
  if (open && !insideDrawer && !onScrim) return
  window.clearTimeout(drawerSettleTimer)
  drawerSettling.value = false
  drawerDragging.value = true
  drawerGesture = {
    pointerId: event.pointerId,
    startX: event.clientX,
    startY: event.clientY,
    lastX: event.clientX,
    lastAt: performance.now(),
    velocityX: 0,
    initialProgress: drawerProgress.value,
    axis: null
  }
  event.currentTarget?.setPointerCapture?.(event.pointerId)
}

const handleDrawerPointerMove = (event) => {
  const gesture = drawerGesture
  if (!gesture || event.pointerId !== gesture.pointerId) return
  const dx = event.clientX - gesture.startX
  const dy = event.clientY - gesture.startY
  if (!gesture.axis) {
    if (Math.abs(dx) < 7 && Math.abs(dy) < 7) return
    if (Math.abs(dy) > Math.abs(dx) * 1.15) {
      drawerGesture = null
      drawerDragging.value = false
      drawerProgress.value = gesture.initialProgress
      return
    }
    gesture.axis = 'x'
  }
  event.preventDefault()
  const now = performance.now()
  const elapsed = Math.max(1, now - gesture.lastAt)
  const instantaneous = (event.clientX - gesture.lastX) / elapsed
  gesture.velocityX = gesture.velocityX * 0.65 + instantaneous * 0.35
  gesture.lastX = event.clientX
  gesture.lastAt = now
  drawerProgress.value = clampDrawerProgress(gesture.initialProgress + dx / drawerWidth())
}

const handleDrawerPointerEnd = (event) => {
  const gesture = drawerGesture
  if (!gesture || event.pointerId !== gesture.pointerId) return
  drawerGesture = null
  if (gesture.axis !== 'x') {
    drawerDragging.value = false
    drawerProgress.value = gesture.initialProgress
    return
  }
  const projected = drawerProgress.value + (gesture.velocityX * 180) / drawerWidth()
  settleDrawer(projected >= 0.5)
}

const handleDrawerPointerCancel = (event) => {
  const gesture = drawerGesture
  if (!gesture || event.pointerId !== gesture.pointerId) return
  drawerGesture = null
  drawerDragging.value = false
  drawerProgress.value = gesture.initialProgress
}

const handleMobileSidebarClick = (event) => {
  if (!isMobileShell.value) return
  if (event.target?.closest?.('.en-sidebar-tree-toggle, .en-recent-heading, .en-recent-more, [aria-expanded]')) return
  if (event.target?.closest?.('button, a, [role="button"]')) {
    window.requestAnimationFrame(closeMobileSidebar)
  }
}

const updateMobileShell = () => {
  const coarsePointer = window.matchMedia?.('(pointer: coarse)').matches
  const narrowViewport = window.matchMedia?.('(max-width: 760px)').matches
  const wasMobile = isMobileShell.value
  isMobileShell.value = !!(coarsePointer || narrowViewport)
  if (isMobileShell.value && !wasMobile) {
    sidebarVisible.value = false
    drawerProgress.value = 0
  } else if (!isMobileShell.value && !sidebarVisible.value) {
    sidebarVisible.value = true
    drawerProgress.value = 1
  }
}

const createLocalVault = async () => {
  const payload = await elephantnoteClient.vaults.createLocal()
  if (payload?.canceled) return false
  store.applyPayload(payload)
  return true
}

const handleMobileCreate = async (key) => {
  if (mobileCreateBusy.value) return
  mobileCreateBusy.value = true
  try {
    if (key === 'note') await store.createNote?.()
    else if (key === 'folder') await store.createFolder?.()
    else if (key === 'drawing') openNewDrawing()
  } catch (error) {
    console.error(`[shell] create ${key} failed`, error)
  } finally {
    mobileCreateBusy.value = false
  }
}

const refreshVisibleVaultFiles = async () => {
  if (!store.activeVault?.path) return
  try {
    const currentPath = store.currentPath || ''
    const entries = await elephantnoteClient.directory.list(currentPath)
    store.entries = Array.isArray(entries) ? entries : []
    const rootEntries = currentPath ? await elephantnoteClient.directory.list('') : store.entries
    store.rootEntries = Array.isArray(rootEntries) ? rootEntries : []
  } catch (error) {
    console.warn('Unable to refresh the vault after synchronization:', error)
  }
}

const setTheme = (value) => {
  const nextTheme = normalizeThemeId(value)
  theme.value = nextTheme
  window.localStorage.setItem(ELEPHANTNOTE_THEME_STORAGE_KEY, nextTheme)
}

watch(theme, (mode) => {
  canvasStore.setAppMode(getThemeMode(mode))
  applyThemeVariables()
}, { immediate: true })

watch(
  () => [store.activeWorkspaceView, store.openedNotePath, store.activeVaultId],
  ([workspaceView, openedNotePath]) => {
    if (!CORE_WORKSPACE_VIEWS.has(workspaceView)) {
      store.setWorkspaceView('notes', { record: false })
      activeAddonViewId.value = ''
      return
    }
    if (openedNotePath || workspaceView !== 'notes') activeAddonViewId.value = ''
  }
)

watch(availableAddonViewIds, (ids) => {
  if (activeAddonViewId.value && !ids.includes(activeAddonViewId.value)) closeAddonView()
})

provide('elephantnoteTheme', theme)
provide('setElephantnoteTheme', setTheme)

const normalizeSidebarWidth = (value) => Math.min(320, Math.max(184, Number(value) || 232))

const setSidebarWidth = (value, { persist = true } = {}) => {
  sidebarWidth.value = normalizeSidebarWidth(value)
  if (persist) {
    window.localStorage.setItem('elephantnote:sidebarWidth', String(sidebarWidth.value))
  }
}

const scheduleSidebarWidth = (value) => {
  pendingSidebarWidth = value
  if (sidebarResizeFrame) return
  sidebarResizeFrame = window.requestAnimationFrame(() => {
    sidebarResizeFrame = null
    setSidebarWidth(pendingSidebarWidth, { persist: false })
  })
}

const startResize = (event) => {
  const startX = event.clientX
  const startWidth = sidebarWidth.value
  event.currentTarget.setPointerCapture(event.pointerId)
  document.documentElement.classList.add('en-resizing-sidebar')

  const onMove = (moveEvent) => {
    scheduleSidebarWidth(startWidth + moveEvent.clientX - startX)
  }
  const onUp = () => {
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
    document.documentElement.classList.remove('en-resizing-sidebar')
    if (sidebarResizeFrame) {
      window.cancelAnimationFrame(sidebarResizeFrame)
      sidebarResizeFrame = null
    }
    setSidebarWidth(pendingSidebarWidth ?? sidebarWidth.value)
    pendingSidebarWidth = null
  }
  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
}

const handleResizeKeydown = (event) => {
  const delta = event.key === 'ArrowLeft' ? -16 : event.key === 'ArrowRight' ? 16 : 0
  if (!delta) return
  event.preventDefault()
  setSidebarWidth(sidebarWidth.value + delta)
}

const handleShortcut = (event) => {
  const key = String(event.key || '')
  if (event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey) {
    if (key === 'ArrowLeft') {
      event.preventDefault()
      const entry = navigationStore.back()
      if (entry) store.navigateTo(entry)
      return
    }
    if (key === 'ArrowRight') {
      event.preventDefault()
      const entry = navigationStore.forward()
      if (entry) store.navigateTo(entry)
      return
    }
  }

  if ((event.ctrlKey || event.metaKey) && !event.altKey && !event.shiftKey && key.toLowerCase() === 'r') {
    event.preventDefault()
    navigationStore.syncWorkspace(store.activeVault?.path)
    return
  }

  const isFindShortcut =
    (event.ctrlKey || event.metaKey) &&
    !event.altKey &&
    !event.shiftKey &&
    key.toLowerCase() === 'f'

  if (isFindShortcut && store.hasVault) {
    if (isSettingsOpen.value) return
    event.preventDefault()
    if (store.openedNotePath) {
      bus.emit('find', 'find')
    } else {
      openSearch()
    }
    return
  }

  const isSearchShortcut =
    (event.ctrlKey || event.metaKey) &&
    !event.altKey &&
    !event.shiftKey &&
    key.toLowerCase() === 'k'

  if (!isSearchShortcut || !store.hasVault) return
  event.preventDefault()
  openSearch()
}

const handleTabSaved = (_event, tabId) => {
  const savedTab = editorStore.tabs.find((tab) => tab.id === tabId)
  if (savedTab?.pathname) {
    store.refreshSavedNote(savedTab.pathname).catch((error) => {
      console.warn('Unable to refresh ElephantNote entry after save:', error)
    })
  }
}

onMounted(() => {
      updateMobileShell()
      window.addEventListener('resize', updateMobileShell)
      window.screen?.orientation?.addEventListener?.('change', updateMobileShell)
      window.addEventListener('elephantnote:vault-files-changed', refreshVisibleVaultFiles)
      window.addEventListener('keydown', handleShortcut)
  window.addEventListener('elephantnote:open-settings', handleOpenSettingsEvent)
  window.addEventListener('elephantnote:open-addon-view', handleOpenAddonViewEvent)
  window.tauri.ipcRenderer.on('mt::tab-saved', handleTabSaved)
  setTheme(theme.value)
  const storedWidth = Number(window.localStorage.getItem('elephantnote:sidebarWidth'))
  if (storedWidth && storedWidth <= 320) {
    setSidebarWidth(storedWidth)
  } else {
    setSidebarWidth(232)
  }
  if (!preferences.autoSave) {
    preferences.SET_SINGLE_PREFERENCE({ type: 'autoSave', value: true })
  }
  store.load()
})

onBeforeUnmount(() => {
      window.clearTimeout(drawerSettleTimer)
      window.removeEventListener('resize', updateMobileShell)
      window.screen?.orientation?.removeEventListener?.('change', updateMobileShell)
      window.removeEventListener('elephantnote:vault-files-changed', refreshVisibleVaultFiles)
      window.removeEventListener('keydown', handleShortcut)
  window.removeEventListener('elephantnote:open-settings', handleOpenSettingsEvent)
  window.removeEventListener('elephantnote:open-addon-view', handleOpenAddonViewEvent)
  document.documentElement.classList.remove('en-resizing-sidebar')
  if (sidebarResizeFrame) {
    window.cancelAnimationFrame(sidebarResizeFrame)
    sidebarResizeFrame = null
  }
  const ipcRenderer = window.tauri.ipcRenderer
  if (ipcRenderer.off) {
    ipcRenderer.off('mt::tab-saved', handleTabSaved)
  } else {
    ipcRenderer.removeListener?.('mt::tab-saved', handleTabSaved)
  }
})
</script>

<style scoped>
.en-shell {
  --en-bg: #0f141d;
  --en-surface: #141a24;
  --en-soft: #1b2432;
  --en-border: #283244;
  --en-border-strong: #3a465a;
  --en-text: #eef3fb;
  --en-muted: #98a3b6;
  --en-primary: #5ea1ff;
  height: 100vh;
  height: 100dvh;
  color: var(--en-text);
  background: var(--en-bg);
  overflow: hidden;
  display: flex;
  flex-direction: row;
}

.en-shell,
.en-shell :deep(*) {
  box-sizing: border-box;
}

.en-shell-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.en-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}

.en-body {
  flex: 1;
  display: grid;
  grid-template-columns: var(--en-sidebar-width) 0 minmax(0, 1fr);
  overflow: hidden;
}

.en-body.en-sidebar-hidden {
  grid-template-columns: 0 0 minmax(0, 1fr);
}

.en-body-main {
  grid-column: 3;
}

.en-sidebar-resizer {
  position: relative;
  z-index: 9;
  width: 0;
  background: transparent;
  border: 0;
  cursor: col-resize;
  touch-action: none;
}

.en-sidebar-resizer::before {
  content: '';
  position: absolute;
  top: 0;
  right: -6px;
  bottom: 0;
  width: 12px;
  z-index: 1;
  background: transparent;
  cursor: col-resize;
}

.en-sidebar-resizer::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 3px;
  height: 64px;
  border-radius: 999px;
  background: var(--en-primary);
  opacity: 0;
  pointer-events: none;
  transform: translate(-50%, -50%);
  transition: opacity 120ms ease;
}

.en-sidebar-resizer:hover::after,
.en-sidebar-resizer:focus-visible::after,
:global(.en-resizing-sidebar) .en-sidebar-resizer::after {
  opacity: 1;
}

.en-floating-surfaces .en-sidebar-resizer {
  width: 0;
  background: transparent;
  border: 0;
}

.en-floating-surfaces .en-sidebar {
  border-right: 0 !important;
}

.en-floating-surfaces .en-body-main {
  border-left: 0 !important;
}

.en-floating-surfaces .en-sidebar-resizer::after {
  background: color-mix(in srgb, var(--en-primary) 72%, var(--en-border));
}

:global(.en-resizing-sidebar),
:global(.en-resizing-sidebar *) {
  cursor: col-resize !important;
  user-select: none !important;
}

:global(.en-settings-close) {
  width: 32px !important;
  height: 32px !important;
  min-height: 32px !important;
  flex: 0 0 32px !important;
  padding: 0 !important;
  border-radius: 10px !important;
}

:global(.en-settings-close .en-icon) {
  width: 17px !important;
  height: 17px !important;
}

.en-mobile-drawer-settling :deep(.en-sidebar),
.en-mobile-drawer-settling .en-mobile-scrim {
  transition: transform 280ms cubic-bezier(0.22, 1, 0.36, 1), opacity 280ms cubic-bezier(0.22, 1, 0.36, 1);
}
</style>
