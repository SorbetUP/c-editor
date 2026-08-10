<template>
  <div
    class="en-library-grid"
    :class="{
      list: store.viewMode === 'list',
      'is-drop-target': isRootDropTarget,
      'is-empty': !visibleEntries.length
    }"
    :data-view-mode="store.viewMode"
    @scroll.passive="handleGridScroll"
    @dragover.prevent="handleRootDragOver"
    @dragleave="handleRootDragLeave"
    @drop.prevent="handleRootDrop"
  >
    <div
      v-if="store.viewMode === 'grid'"
      class="en-library-grid-surface en-library-grid-surface--grid"
      data-layout="grid"
    >
      <NoteCard
        v-for="(entry, index) in visibleEntries"
        :key="entry.path"
        :entry="entry"
        :featured="index === 0 && visibleEntries.length > 3"
        @open="openEntry"
        @rename="renameEntry"
        @delete="deleteEntry"
      />
    </div>
    <div
      v-else
      class="en-library-grid-surface en-library-grid-surface--list"
      data-layout="list"
    >
      <NoteCard
        v-for="entry in visibleEntries"
        :key="entry.path"
        :entry="entry"
        :featured="false"
        @open="openEntry"
        @rename="renameEntry"
        @delete="deleteEntry"
      />
    </div>
  </div>
</template>

<script setup>
import { computed, nextTick, ref, watch } from 'vue'
import log from '@/platform/runtimeLogShim'
import bus from '@/bus'
import { useVaultStore } from '../../stores/vaultStore'
import { useNavigationStore } from '../../stores/navigationStore'
import { elephantnoteClient } from '../../services/elephantnoteClient'
import NoteCard from './NoteCard.vue'
import {
  canDropEntryOnDirectory,
  getEntryKind,
  parseDraggedEntry
} from '../../utils/entryDragDrop'
import { getNoteCardDrawingPreview } from '../../utils/noteCardView'

const DIRECTORY_PAGE_SIZE = 120
const RENDER_CHUNK_SIZE = 72
const SCROLL_PREFETCH_PX = 720

const store = useVaultStore()
const navigationStore = useNavigationStore()
const isRootDropTarget = ref(false)
const visibleEntryLimit = ref(RENDER_CHUNK_SIZE)
const loadingMoreEntries = ref(false)
const directoryMayHaveMore = ref(true)
const directoryGeneration = ref(0)

const normalizeSlashPath = (value = '') => String(value || '').split(String.fromCharCode(92)).join('/')
const isMarkdownNotePath = (path = '') => /[.]md$/i.test(String(path || ''))
const isExcalidrawPath = (path = '') => /[.]excalidraw(?:[.]png)?$/i.test(String(path || ''))
const entryArray = (value) => Array.isArray(value) ? value : []

const isCompatibilityRootWikiEntry = (entry) => {
  const pathname = normalizeSlashPath(entry?.path).replace(/\/+$/g, '')
  return store.activeWorkspaceView === 'notes' && store.currentPath === '' && /^wiki$/i.test(pathname)
}

const filteredEntries = computed(() => entryArray(store.activeEntries).filter((entry) => !isCompatibilityRootWikiEntry(entry)))
const visibleEntries = computed(() => filteredEntries.value.slice(0, visibleEntryLimit.value))

const resetVisibleWindow = () => {
  visibleEntryLimit.value = RENDER_CHUNK_SIZE
  directoryMayHaveMore.value = entryArray(store.entries).length >= DIRECTORY_PAGE_SIZE
  directoryGeneration.value += 1
}

watch(
  () => [store.currentPath, store.activeWorkspaceView, store.activeVaultId],
  resetVisibleWindow
)

watch(
  () => entryArray(store.entries).length,
  (length, previousLength) => {
    if (length < previousLength) resetVisibleWindow()
  }
)

const shouldApplyFolderResult = (relativePath, vaultId, view) => {
  return store.activeWorkspaceView === view &&
    store.currentPath === relativePath &&
    store.activeVaultId === vaultId
}

const normalizeDirectoryPage = (items) => Array.isArray(items) ? items : []

const fetchDirectoryPage = async(relativePath, offset = 0, generation = directoryGeneration.value) => {
  const items = normalizeDirectoryPage(await elephantnoteClient.directory.list({
    relativePath,
    offset,
    limit: DIRECTORY_PAGE_SIZE + 1,
    includePreview: true
  }))
  if (generation !== directoryGeneration.value) return null
  directoryMayHaveMore.value = items.length > DIRECTORY_PAGE_SIZE
  return items.slice(0, DIRECTORY_PAGE_SIZE)
}

const openPagedFolder = async(relativePath, view) => {
  const vaultId = store.activeVaultId
  log.info('[library] openPagedFolder:start', {
    relativePath,
    view,
    vaultId,
    previousOpenedNotePath: store.openedNotePath || null,
    previousCurrentPath: store.currentPath || ''
  })
  store.currentPath = relativePath
  store.openedNotePath = ''
  store.activeWorkspaceView = view
  store.entries = []
  resetVisibleWindow()
  await nextTick()

  try {
    const page = await fetchDirectoryPage(relativePath, 0)
    if (!shouldApplyFolderResult(relativePath, vaultId, view) || !page) return
    store.entries = page
    if (!relativePath) store.rootEntries = page
    store.activeWorkspaceView = view
    if (view === 'notes') {
      navigationStore.push(relativePath
        ? { type: 'folder', id: relativePath, path: relativePath }
        : { type: 'all_notes' })
    }
    log.info(`[${view}] opened paged folder in library grid`, {
      path: relativePath,
      entries: entryArray(store.entries).length,
      mayHaveMore: directoryMayHaveMore.value
    })
    log.info('[library] openPagedFolder:done', {
      relativePath,
      view,
      vaultId,
      openedNotePath: store.openedNotePath || null,
      currentPath: store.currentPath || '',
      entryCount: entryArray(store.entries).length
    })
  } catch (error) {
    if (!shouldApplyFolderResult(relativePath, vaultId, view)) return
    store.entries = []
    store.activeWorkspaceView = view
    directoryMayHaveMore.value = false
    log.info(`[${view}] folder empty or unavailable in library grid`, {
      path: relativePath,
      error: error?.message || error
    })
    log.info('[library] openPagedFolder:error', {
      relativePath,
      view,
      vaultId,
      openedNotePath: store.openedNotePath || null,
      error: error?.message || String(error)
    })
  }
}

const openFolderInCurrentView = async (relativePath) => {
  await openPagedFolder(relativePath, store.activeWorkspaceView === 'wiki' ? 'wiki' : 'notes')
}

const loadMoreVisibleEntries = async() => {
  if (visibleEntryLimit.value < filteredEntries.value.length) {
    visibleEntryLimit.value = Math.min(filteredEntries.value.length, visibleEntryLimit.value + RENDER_CHUNK_SIZE)
    return
  }
  if (!directoryMayHaveMore.value || loadingMoreEntries.value) return
  const generation = directoryGeneration.value
  loadingMoreEntries.value = true
  try {
    const page = await fetchDirectoryPage(store.currentPath, entryArray(store.entries).length, generation)
    if (!page || generation !== directoryGeneration.value) return
    if (page.length) {
      const seen = new Set(entryArray(store.entries).map((entry) => entry.path))
      const appended = page.filter((entry) => !seen.has(entry.path))
      store.entries = [...entryArray(store.entries), ...appended]
      if (!store.currentPath) store.rootEntries = store.entries
      visibleEntryLimit.value = Math.min(filteredEntries.value.length + appended.length, visibleEntryLimit.value + RENDER_CHUNK_SIZE)
    }
  } finally {
    loadingMoreEntries.value = false
  }
}

const handleGridScroll = (event) => {
  const target = event.currentTarget
  if (!target) return
  const distanceFromBottom = target.scrollHeight - target.scrollTop - target.clientHeight
  if (distanceFromBottom <= SCROLL_PREFETCH_PX) {
    void loadMoreVisibleEntries()
  }
}

const openEntry = async (entry) => {
  const kind = getEntryKind(entry)
  log.info('[library] openEntry', {
    path: entry?.path || null,
    type: entry?.type || null,
    kind,
    currentPath: store.currentPath || '',
    openedNotePath: store.openedNotePath || null
  })
  if (kind === 'folder') {
    await openFolderInCurrentView(entry.path)
    return
  }
  if (isExcalidrawPath(entry?.path)) {
    const source = store.activeVault?.path
      ? window.path.join(store.activeVault.path, entry.path)
      : entry.path
    bus.emit('open-excalidraw-from-image', source)
    log.info('[library] opened Excalidraw entry', { path: entry.path, source })
    return
  }
  if (kind === 'drawing') {
    const source = getNoteCardDrawingPreview(entry)
    if (!source) {
      log.error('[library] drawing entry has no preview source', {
        path: entry?.path || '',
        title: entry?.title || ''
      })
      return
    }
    log.info('[library] opening drawing directly in Excalidraw', {
      path: entry?.path || '',
      source
    })
    bus.emit('open-excalidraw-from-image', source)
    return
  }
  if (kind === 'note' || isMarkdownNotePath(entry?.path)) {
    store.openNote(entry)
    return
  }
  log.warn('[library] ignored non-markdown entry open request', {
    path: entry?.path || '',
    type: entry?.type || '',
    kind
  })
}

const renameEntry = async ({ entry, title } = {}) => {
  if (!entry || !title) return
  await store.renameEntry(entry, title)
  resetVisibleWindow()
}

const deleteEntry = async (entry) => {
  await store.deleteEntry(entry)
  resetVisibleWindow()
}

const handleRootDragOver = (event) => {
  const entry = parseDraggedEntry(event.dataTransfer)
  isRootDropTarget.value = canDropEntryOnDirectory(entry, '')
}

const handleRootDragLeave = () => {
  isRootDropTarget.value = false
}

const handleRootDrop = async(event) => {
  const entry = parseDraggedEntry(event.dataTransfer)
  isRootDropTarget.value = false
  if (!canDropEntryOnDirectory(entry, '')) return
  await store.moveEntry(entry, '')
  resetVisibleWindow()
}
</script>

<style scoped>
.en-library-grid {
  min-height: 0;
  overflow: auto;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr);
  gap: 0;
  padding: 0;
  align-items: stretch;
  align-content: stretch;
  overscroll-behavior: contain;
}

/* The controls are intentionally floating, but their hit area must not share
 * pixels with the first card. Keep the compact toolbar visually detached from
 * the content while preserving a reliable pointer target on desktop. */
@media (min-width: 761px) {
  .en-library-grid {
    padding-top: 72px;
  }
}
.en-library-grid.list {
  display: flex;
  flex-direction: column;
  align-items: stretch;
}
.en-library-grid-surface {
  min-width: 0;
  min-height: 100%;
  align-content: start;
  padding: 0 10px 10px;
}
.en-library-grid-surface--grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(240px, 100%), 1fr));
  gap: 10px;
  align-items: start;
}
.en-library-grid-surface--list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: stretch;
}
.en-library-grid.is-empty {
  display: block;
  padding: 0;
  background: transparent;
}
.en-library-grid.is-drop-target {
  outline: 2px dashed var(--en-accent);
  outline-offset: -8px;
}
</style>
