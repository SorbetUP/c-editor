<template>
  <article
    class="en-card en-note-card"
    :class="{
      'is-featured': featured,
      'is-pinned': isPinned,
      'is-folder': isFolder,
      'is-dragging': isDragging,
      'is-drop-target': isDropTarget,
      'is-drop-disabled': isDropDisabled,
      'is-renaming': isRenaming
    }"
    draggable="true"
    @mouseenter="isHovering = true"
    @mouseleave="isHovering = false"
    @dragstart="handleDragStart"
    @dragend="handleDragEnd"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
    @contextmenu.stop.prevent="openContextMenu"
    @click="handleCardClick"
  >
    <div class="en-card-actions">
      <button
        class="en-card-pin-button"
        type="button"
        :class="{ visible: isPinned || isHovering }"
        :title="isPinned ? 'Unpin entry' : 'Pin entry'"
        :aria-label="isPinned ? 'Unpin entry' : 'Pin entry'"
        @click.stop.prevent="togglePin"
      >
        <Pin
          class="en-icon"
        />
      </button>
      <button
        class="en-card-menu"
        type="button"
        :title="isFolder ? 'Folder actions' : 'Note actions'"
        :aria-label="isFolder ? 'Folder actions' : 'Note actions'"
        @click.stop.prevent="toggleMenu"
      >
        <MoreHorizontal class="en-icon" />
      </button>
    </div>
    <div
      v-if="isMenuOpen"
      class="en-card-popover"
      @click.stop
    >
      <button
        type="button"
        data-entry-action="rename"
        title="Rename"
        aria-label="Rename"
        @click.stop="beginRename"
      >
        <Pencil
          class="en-icon"
          aria-hidden="true"
        />
      </button>
      <button
        v-if="isFolder"
        type="button"
        data-entry-action="sidebar"
        :title="isSidebarVisible ? 'Hide from sidebar' : 'Show in sidebar'"
        :aria-label="isSidebarVisible ? 'Hide from sidebar' : 'Show in sidebar'"
        @click.stop.prevent="toggleSidebarVisibility"
      >
        <component
          :is="isSidebarVisible ? EyeOff : Eye"
          class="en-icon"
          aria-hidden="true"
        />
      </button>
      <button
        type="button"
        class="danger"
        data-entry-action="delete"
        title="Delete"
        aria-label="Delete"
        @click.stop.prevent="deleteEntry"
      >
        <Trash2
          class="en-icon"
          aria-hidden="true"
        />
      </button>
    </div>
    <div class="en-note-card-head">
      <div class="en-note-card-title-row">
        <component
          :is="isFolder ? Folder : isDrawing ? PenLine : FileText"
          class="en-note-document-icon"
        />
        <input
          v-if="isRenaming"
          ref="renameInput"
          v-model="renameDraft"
          class="en-note-card-title-input"
          data-entry-rename-input
          type="text"
          :aria-label="`Rename ${title}`"
          @click.stop
          @keydown.enter.stop.prevent="commitRename"
          @keydown.esc.stop.prevent="cancelRename"
        >
        <h3
          v-else
          @dblclick.stop.prevent="beginRename"
        >
          {{ title }}
        </h3>
      </div>
    </div>
    <div
      v-if="isFolder"
      class="en-folder-preview"
      :class="{ 'is-empty': !previewItems.length }"
      :aria-label="previewItems.length ? 'Folder contents preview' : 'Empty folder'"
    >
      <span
        v-for="item in previewItems"
        :key="`${item.type}:${item.title}`"
        class="en-folder-preview-item"
        :title="previewItemTitle(item)"
      >
        <component
          :is="previewItemIcon(item)"
          class="en-folder-preview-icon"
          aria-hidden="true"
        />
        <span>{{ previewItemTitle(item) }}</span>
      </span>
      <span v-if="!previewItems.length">No items yet</span>
    </div>
    <div
      v-else-if="drawingPreview"
      class="en-note-card-drawing-preview"
    >
      <img
        :src="drawingPreviewSrc"
        :alt="`${title} preview`"
        loading="lazy"
        data-elephant-excalidraw-preview="true"
      >
    </div>
    <p v-else>
      {{ excerpt }}
    </p>
    <div class="en-tags">
      <span
        v-for="tag in entry.tags"
        :key="tag"
      >
        #{{ tag }}
      </span>
    </div>
  </article>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Eye, EyeOff, FileText, Folder, MoreHorizontal, PenLine, Pencil, Pin, Trash2 } from '@lucide/vue'
import { useVaultStore } from '../../stores/vaultStore'
import {
  canDropEntryOnDirectory,
  clearDraggedEntry,
  getEntryKind,
  parseDraggedEntry,
  writeDraggedEntry
} from '../../utils/entryDragDrop'
import {
  getNoteCardExcerpt,
  getNoteCardDrawingPreview,
  getNoteCardTitle
} from '../../utils/noteCardView'

const props = defineProps({
  entry: {
    type: Object,
    required: true
  },
  featured: {
    type: Boolean,
    default: false
  }
})
const emit = defineEmits(['open', 'rename', 'delete'])

const store = useVaultStore()
const isMenuOpen = ref(false)
const isHovering = ref(false)
const isDragging = ref(false)
const isDropTarget = ref(false)
const isDropDisabled = ref(false)
const isRenaming = ref(false)
const renameDraft = ref('')
const renameInput = ref(null)
const CLICK_OPEN_DELAY_MS = 220
let openClickTimer = null

const title = computed(() => getNoteCardTitle(props.entry))
const isFolder = computed(() => getEntryKind(props.entry) === 'folder')
const excerpt = computed(() => isFolder.value
  ? `${props.entry.noteCount || 0} note${props.entry.noteCount === 1 ? '' : 's'}`
  : getNoteCardExcerpt(props.entry))
const previewItems = computed(() => Array.isArray(props.entry?.childrenPreview)
  ? props.entry.childrenPreview.slice(0, 3)
  : [])
const drawingPreview = computed(() => isFolder.value ? '' : getNoteCardDrawingPreview(props.entry))
const isDrawing = computed(() => !isFolder.value && (!!drawingPreview.value || /\.excalidraw(?:\.png)?$/i.test(
  String(props.entry?.path || props.entry?.name || props.entry?.filename || '')
)))

const previewItemTitle = (item) => String(item?.title || 'Untitled')
  .replace(/\.(?:md|excalidraw)$/i, '')

const previewItemIcon = (item) => {
  if (item?.type === 'folder') return Folder
  if (item?.type === 'drawing' || /\.excalidraw(?:\.png)?$/i.test(String(item?.title || ''))) return PenLine
  return FileText
}

const drawingPreviewSrc = ref('')
let drawingPreviewObjectUrl = ''
let drawingPreviewLoadId = 0

const resolveDrawingPreviewPath = (source) => {
  const vaultRoot = store.activeVault?.path || ''
  if (!vaultRoot) return source
  const joined = window.path?.join
    ? window.path.join(vaultRoot, source)
    : `${vaultRoot.replace(/[\\/]$/, '')}/${source.replace(/^\.?[\\/]/, '')}`
  return String(joined).replace(/\\/g, '/')
}

const drawingMimeType = (pathname) => /[.]jpe?g$/i.test(pathname) ? 'image/jpeg' : 'image/png'

const loadDrawingPreview = async () => {
  const source = drawingPreview.value
  const loadId = ++drawingPreviewLoadId
  if (drawingPreviewObjectUrl) URL.revokeObjectURL(drawingPreviewObjectUrl)
  drawingPreviewObjectUrl = ''
  drawingPreviewSrc.value = ''
  if (!source) return
  if (/^(?:asset|data):/i.test(source)) {
    drawingPreviewSrc.value = source
    return
  }

  const pathname = resolveDrawingPreviewPath(source)
  const readFile = window.fileUtils?.readFile
  if (typeof readFile !== 'function') {
    drawingPreviewSrc.value = source
    return
  }

  try {
    const data = await readFile.call(window.fileUtils, pathname)
    const blob = data instanceof Blob ? data : new Blob([data], { type: drawingMimeType(pathname) })
    const objectUrl = URL.createObjectURL(blob)
    if (loadId !== drawingPreviewLoadId) {
      URL.revokeObjectURL(objectUrl)
      return
    }
    drawingPreviewObjectUrl = objectUrl
    drawingPreviewSrc.value = objectUrl
  } catch (error) {
    if (loadId !== drawingPreviewLoadId) return
    console.warn('[library] drawing preview load failed', {
      path: pathname,
      error: error?.message || String(error)
    })
    drawingPreviewSrc.value = source
  }
}

watch(
  () => [drawingPreview.value, store.activeVault?.path],
  loadDrawingPreview,
  { immediate: true }
)
const isPinned = computed(() => !!props.entry?.path && store.isEntryPinned(props.entry.path))
const isSidebarVisible = computed(() => isFolder.value && store.isFolderVisibleInSidebar(props.entry.path))

const toggleMenu = () => {
  isMenuOpen.value = !isMenuOpen.value
}

const openContextMenu = () => {
  isMenuOpen.value = true
}

const closeMenu = (event) => {
  const target = event?.target
  const isRenameAction = target?.closest?.('[data-entry-action="rename"]')
  if (isRenaming.value && !target?.closest?.('[data-entry-rename-input]') && !isRenameAction) {
    cancelRename()
  }

  if (!isMenuOpen.value) return
  if (event?.target?.closest?.('.en-note-card')) return
  isMenuOpen.value = false
}

const togglePin = () => {
  if (!props.entry?.path) return
  store.togglePinnedEntry(props.entry.path)
  isMenuOpen.value = false
}

const handleDragStart = (event) => {
  isDragging.value = true
  writeDraggedEntry(event, {
    ...props.entry,
    kind: getEntryKind(props.entry),
    title: title.value,
    preview: drawingPreview.value
  })
}

const handleDragEnd = () => {
  clearDraggedEntry()
  isDragging.value = false
  isDropTarget.value = false
  isDropDisabled.value = false
}

const handleDragOver = (event) => {
  if (!isFolder.value) return
  event.preventDefault()
  event.stopPropagation()
  const draggedEntry = parseDraggedEntry(event)
  const canDrop = canDropEntryOnDirectory(draggedEntry, props.entry.path)
  isDropTarget.value = canDrop
  isDropDisabled.value = !!draggedEntry && !canDrop
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = canDrop ? 'move' : 'none'
  }
}

const handleDragLeave = (event) => {
  if (!isFolder.value) return
  event.stopPropagation()
  isDropTarget.value = false
  isDropDisabled.value = false
}

const handleDrop = async (event) => {
  if (!isFolder.value) return
  event.preventDefault()
  event.stopPropagation()
  const draggedEntry = parseDraggedEntry(event)
  const canDrop = canDropEntryOnDirectory(draggedEntry, props.entry.path)
  isDropTarget.value = false
  isDropDisabled.value = false
  console.info('[library:dnd] folder drop', { draggedEntry, target: props.entry.path, canDrop })
  if (!canDrop) return
  await store.moveEntry(draggedEntry, props.entry.path)
}

const beginRename = async () => {
  if (openClickTimer) {
    window.clearTimeout(openClickTimer)
    openClickTimer = null
  }
  isMenuOpen.value = false
  isRenaming.value = true
  renameDraft.value = title.value
  await nextTick()
  renameInput.value?.focus?.()
  renameInput.value?.select?.()
}

const cancelRename = () => {
  isRenaming.value = false
  renameDraft.value = ''
}

const commitRename = () => {
  const nextTitle = renameDraft.value.trim()
  const previousTitle = title.value
  cancelRename()
  if (!nextTitle || nextTitle === previousTitle) return
  emit('rename', { entry: props.entry, title: nextTitle })
}

const handleCardClick = () => {
  if (isRenaming.value) {
    cancelRename()
    return
  }
  if (openClickTimer) window.clearTimeout(openClickTimer)
  openClickTimer = window.setTimeout(() => {
    openClickTimer = null
    emit('open', props.entry)
  }, CLICK_OPEN_DELAY_MS)
}

const deleteEntry = () => {
  isMenuOpen.value = false
  emit('delete', props.entry)
}

const toggleSidebarVisibility = async () => {
  if (!isFolder.value || !props.entry?.path) return
  try {
    await store.toggleEntrySidebarVisibility(props.entry)
  } finally {
    isMenuOpen.value = false
  }
}

onMounted(() => {
  window.addEventListener('click', closeMenu)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeMenu)
  drawingPreviewLoadId += 1
  if (drawingPreviewObjectUrl) URL.revokeObjectURL(drawingPreviewObjectUrl)
  drawingPreviewObjectUrl = ''
  if (openClickTimer) window.clearTimeout(openClickTimer)
})
</script>

<style scoped>
.en-card {
  position: relative;
  min-height: 176px;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--en-border);
  border-radius: 10px;
  padding: 10px;
  color: var(--en-text);
  background: color-mix(in srgb, var(--en-surface) 34%, var(--en-bg));
  overflow: hidden;
  contain: layout paint style;
  content-visibility: auto;
  contain-intrinsic-size: 176px 320px;
}

.en-card.is-featured {
  min-height: 220px;
  contain-intrinsic-size: 220px 560px;
}

/* Folders use the same compact footprint as other library entries even when
 * the first grid item receives the featured-card modifier. */
.en-card.is-featured.is-folder {
  min-height: 176px;
  contain-intrinsic-size: 176px 320px;
}

.en-card:hover {
  border-color: var(--en-border-strong);
}

.en-card.is-dragging {
  opacity: 0.42;
}

.en-card.is-folder {
  cursor: pointer;
  min-height: 176px;
}

.en-card.is-drop-target {
  border-color: var(--en-primary);
  background: color-mix(in srgb, var(--en-primary) 10%, var(--en-bg));
}

.en-card.is-drop-disabled {
  border-color: var(--en-danger);
}

.en-card-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 6px;
}

.en-card-pin-button,
.en-card-menu {
  width: 30px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  color: var(--en-muted);
  background: transparent;
  cursor: pointer;
}

.en-card-pin-button.visible,
.en-card-menu {
  opacity: 1;
}

.en-card-pin-button:not(.visible) {
  opacity: 0;
}

.en-card.is-pinned .en-card-pin-button {
  color: #facc15;
}

.en-card.is-pinned .en-card-pin-button :deep(svg) {
  fill: currentColor;
}

.en-card-popover {
  position: absolute;
  top: 42px;
  right: 8px;
  z-index: 5;
  min-width: 0;
  display: flex;
  flex-direction: row;
  gap: 4px;
  border: 1px solid var(--en-border);
  border-radius: 10px;
  padding: 5px;
  background: var(--en-surface);
  box-shadow: 0 10px 24px rgb(0 0 0 / 24%);
}

.en-card-popover button {
  width: 32px;
  height: 32px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 8px;
  color: var(--en-text);
  background: transparent;
  cursor: pointer;
}

.en-card-popover button:hover,
.en-card-popover button:focus-visible {
  background: var(--en-soft);
  outline: none;
}

.en-card-popover button.danger {
  color: var(--en-danger, #ef4444);
}

.en-note-card h3 {
  min-width: 0;
  max-width: calc(100% - 42px);
  margin: 0;
  font-size: clamp(18px, 1.6vw, 26px);
  line-height: 1.12;
  overflow-wrap: anywhere;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.en-note-card-title-input {
  min-width: 0;
  width: 100%;
  margin: 0;
  border: 1px solid var(--en-primary);
  border-radius: 7px;
  padding: 4px 6px;
  color: var(--en-text);
  background: var(--en-input-bg, var(--en-surface));
  font: inherit;
  font-size: clamp(18px, 1.6vw, 26px);
  font-weight: 800;
  line-height: 1.12;
  outline: none;
}

.en-note-card-title-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.en-note-document-icon {
  width: 22px;
  height: 22px;
  flex: 0 0 auto;
  margin-top: 2px;
}

.en-note-card-drawing-preview {
  width: 100%;
  height: 112px;
  margin: 10px 0 8px;
  overflow: hidden;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  background: #ffffff;
}

.en-note-card-drawing-preview img {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: contain;
}

.en-card.is-featured .en-note-card-drawing-preview {
  height: 156px;
}

.en-card.is-folder p {
  margin: 6px 0 0;
}

.en-folder-preview {
  min-height: 42px;
  display: grid;
  gap: 4px;
  margin-top: auto;
  padding: 7px 8px;
  border: 1px solid color-mix(in srgb, var(--en-border) 70%, transparent);
  border-radius: 8px;
  background: color-mix(in srgb, var(--en-surface) 55%, transparent);
}

.en-folder-preview.is-empty {
  display: flex;
  align-items: center;
  color: var(--en-muted);
  font-size: 13px;
}

.en-folder-preview-item {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--en-muted);
  font-size: 13px;
  line-height: 1.2;
}

.en-folder-preview-item span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.en-folder-preview-icon {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
}

.en-library-grid.list .en-note-card {
  min-height: 58px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  padding: 8px 10px;
}

.en-library-grid.list .en-note-card h3 {
  font-size: clamp(16px, 1.4vw, 21px);
  -webkit-line-clamp: 1;
}

.en-library-grid.list .en-note-document-icon {
  width: 20px;
  height: 20px;
}

.en-library-grid.list .en-note-card p,
.en-library-grid.list .en-note-card .en-tags,
.en-library-grid.list .en-note-card-drawing-preview,
.en-library-grid.list .en-note-card .en-folder-preview {
  display: none;
}

</style>
