<template>
  <aside class="en-sidebar">
    <div class="en-sidebar-scroll">
      <button
        class="en-all-notes"
        :class="{
          active: !activeAddonViewId && store.activeWorkspaceView === 'notes' && store.currentPath === '' && !store.openedNotePath,
          'is-drop-target': isRootDropTarget,
          'is-drop-disabled': isRootDropDisabled
        }"
        type="button"
        @dragover.prevent="handleRootDragOver"
        @dragleave="handleRootDragLeave"
        @drop.prevent="handleRootDrop"
        @click="openAllNotes"
      >
        <Inbox class="en-all-notes-icon" />
        <span>All notes</span>
      </button>

      <div class="en-tags-header">
        <span class="en-tags-label">Tags</span>
        <button class="en-tags-search-btn" type="button" title="Search tags" @click="emit('search')">
          <Search class="en-tags-search-icon" />
        </button>
      </div>

      <div class="en-sidebar-main">
        <SidebarTreeEntry
          v-for="item in sidebarEntries"
          :key="item.id || item.path"
          :entry="item"
          :depth="0"
          :active-path="store.currentPath"
          :active-note-path="store.openedNotePath"
          :load-directory="loadDirectory"
          :open-directory="openDirectory"
          :open-note="openNote"
          :detach-entry="store.detachEntryFromSidebar"
        />
      </div>

      <component
        :is="entry.contribution.component"
        v-for="entry in sidebarAfterTreeZones"
        :key="entry.contribution.id"
      />
    </div>
  </aside>
</template>

<script setup>
import { computed, ref } from 'vue'
import { Inbox, Search } from '@lucide/vue'
import { useVaultStore } from '../../stores/vaultStore'
import { useAddonsStore } from '@/store/addons'
import SidebarTreeEntry from './SidebarTreeEntry.vue'
import { elephantnoteClient } from '../../services/elephantnoteClient'
import { canDropEntryOnDirectory, parseDraggedEntry } from '../../utils/entryDragDrop'

const props = defineProps({
  activeAddonViewId: {
    type: String,
    default: ''
  }
})
const emit = defineEmits(['search', 'open-addon-view', 'close-addon-view'])
const store = useVaultStore()
const addonsStore = useAddonsStore()
const isRootDropTarget = ref(false)
const isRootDropDisabled = ref(false)

const normalizePath = (value = '') => String(value || '').replaceAll('\\', '/')
const isHiddenPath = (value = '') => normalizePath(value)
  .split('/')
  .filter(Boolean)
  .some((part) => part.startsWith('.'))
const isFolderEntry = (entry) => (entry?.kind || entry?.type) === 'folder'
const isMarkdownEntry = (entry) => /\.md$/i.test(normalizePath(entry?.path || entry?.filename || ''))
const isVisibleSidebarEntry = (entry) => {
  if (!entry?.path || isHiddenPath(entry.path)) return false
  return isFolderEntry(entry) || isMarkdownEntry(entry)
}
const filterSidebarEntries = (entries) => Array.isArray(entries)
  ? entries.filter(isVisibleSidebarEntry)
  : []

const activeAddonViewId = computed(() => props.activeAddonViewId)
const sidebarAfterTreeZones = computed(() => addonsStore.getContributions('layout.zones')
  .filter((entry) => entry?.contribution?.zone === 'sidebar.after-tree' && entry?.contribution?.component)
  .sort((left, right) => Number(left.contribution.order || 0) - Number(right.contribution.order || 0)))
const sidebarEntries = computed(() => filterSidebarEntries(store.rootSidebarEntries))

const loadDirectory = async (relativePath = '') => {
  if (!store.activeVault?.path) return []
  return filterSidebarEntries(await elephantnoteClient.directory.list(relativePath))
}

const openAllNotes = async () => {
  emit('close-addon-view')
  await store.openDirectory('')
}

const openDirectory = async (...args) => {
  emit('close-addon-view')
  return store.openDirectory(...args)
}

const openNote = (...args) => {
  emit('close-addon-view')
  return store.openNote(...args)
}

const handleRootDragOver = (event) => {
  const entry = parseDraggedEntry(event)
  const canDrop = canDropEntryOnDirectory(entry, '')
  isRootDropTarget.value = canDrop
  isRootDropDisabled.value = !!entry && !canDrop
  if (event.dataTransfer) event.dataTransfer.dropEffect = canDrop ? 'move' : 'none'
}

const handleRootDragLeave = () => {
  isRootDropTarget.value = false
  isRootDropDisabled.value = false
}

const handleRootDrop = async (event) => {
  const entry = parseDraggedEntry(event)
  const canDrop = canDropEntryOnDirectory(entry, '')
  isRootDropTarget.value = false
  isRootDropDisabled.value = false
  if (!canDrop) return
  await store.moveEntry(entry, '')
}
</script>

<style scoped>
.en-sidebar { min-width: 0; border-right: 1px solid var(--en-border); background: var(--en-sidebar-bg, var(--en-bg)); overflow: hidden; }
.en-sidebar-scroll { height: 100%; display: flex; flex-direction: column; gap: 0; padding: 8px 0 0; overflow-y: auto; }
.en-all-notes { min-height: 38px; display: flex; align-items: center; gap: 10px; margin: 0 8px 8px; padding: 0 12px; border: 0; border-radius: 8px; color: var(--en-text); background: var(--en-soft); font: inherit; font-size: 14px; font-weight: 600; text-align: left; cursor: pointer; }
.en-all-notes:hover { background: var(--en-soft-strong); }
.en-all-notes.active { background: color-mix(in srgb, var(--en-primary, #7c3aed) 20%, var(--en-soft)); color: var(--en-text); }
.en-all-notes.is-drop-target { outline: 1px solid var(--en-primary); background: color-mix(in srgb, var(--en-primary) 16%, var(--en-soft)); }
.en-all-notes.is-drop-disabled { outline: 1px solid var(--en-danger); }
.en-all-notes-icon { width: 18px; height: 18px; flex-shrink: 0; }
.en-tags-header { display: flex; align-items: center; justify-content: space-between; padding: 4px 14px 8px; }
.en-tags-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: .06em; color: var(--en-muted); }
.en-tags-search-btn { width: 24px; height: 24px; display: inline-flex; align-items: center; justify-content: center; border: 0; border-radius: 6px; color: var(--en-muted); background: transparent; cursor: pointer; }
.en-tags-search-btn:hover { color: var(--en-text); background: var(--en-soft); }
.en-tags-search-icon { width: 14px; height: 14px; }
.en-sidebar-main { display: flex; flex-direction: column; gap: 2px; min-height: 0; padding: 0 6px; flex: 1; overflow-y: auto; }
</style>
