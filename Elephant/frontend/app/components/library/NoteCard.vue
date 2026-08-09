<template>
  <article
    class="en-card en-note-card"
    :class="{
      'is-featured': featured,
      'is-pinned': isPinned,
      'is-folder': isFolder,
      'is-dragging': isDragging,
      'is-drop-target': isDropTarget,
      'is-drop-disabled': isDropDisabled
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
    @click="$emit('open', entry)"
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
        @click="renameEntry"
      >
        Rename
      </button>
      <button
        type="button"
        class="danger"
        @click="deleteEntry"
      >
        Delete
      </button>
    </div>
    <div class="en-note-card-head">
      <div class="en-note-card-title-row">
        <component
          :is="isFolder ? Folder : FileText"
          class="en-note-document-icon"
        />
        <h3
          @dblclick.stop.prevent="renameEntry"
        >
          {{ title }}
        </h3>
      </div>
    </div>
    <p>{{ excerpt }}</p>
    <div class="en-tags">
      <span
        v-for="tag in entry.tags"
        :key="tag"
      >
        #{{ tag }}
      </span>
    </div>
    <footer>
      <span class="en-dot" />
      <span>{{ updated }}</span>
    </footer>
  </article>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { FileText, Folder, MoreHorizontal, Pin } from '@lucide/vue'
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
  getNoteCardTitle,
  getNoteCardUpdatedLabel
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

const title = computed(() => getNoteCardTitle(props.entry))
const updated = computed(() => getNoteCardUpdatedLabel(props.entry))
const isFolder = computed(() => getEntryKind(props.entry) === 'folder')
const excerpt = computed(() => isFolder.value
  ? `${props.entry.noteCount || 0} note${props.entry.noteCount === 1 ? '' : 's'}`
  : getNoteCardExcerpt(props.entry))
const isPinned = computed(() => !!props.entry?.path && store.isEntryPinned(props.entry.path))

const toggleMenu = () => {
  isMenuOpen.value = !isMenuOpen.value
}

const openContextMenu = () => {
  isMenuOpen.value = true
}

const closeMenu = (event) => {
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
    title: title.value
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

const renameEntry = () => {
  isMenuOpen.value = false
  emit('rename', props.entry)
}

const deleteEntry = () => {
  isMenuOpen.value = false
  emit('delete', props.entry)
}

onMounted(() => {
  window.addEventListener('click', closeMenu)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeMenu)
})
</script>

<style scoped>
.en-card {
  position: relative;
  min-height: 208px;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--en-border);
  border-radius: 10px;
  padding: 22px;
  color: var(--en-text);
  background: color-mix(in srgb, var(--en-surface) 34%, var(--en-bg));
  overflow: hidden;
  contain: layout paint style;
  content-visibility: auto;
  contain-intrinsic-size: 208px 360px;
}

.en-card.is-featured {
  min-height: 286px;
  contain-intrinsic-size: 286px 720px;
}

.en-card:hover {
  border-color: var(--en-border-strong);
}

.en-card.is-dragging {
  opacity: 0.42;
}

.en-card.is-folder {
  cursor: pointer;
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
  top: 22px;
  right: 22px;
  display: flex;
  gap: 10px;
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
  top: 56px;
  right: 22px;
  z-index: 5;
  min-width: 190px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 8px;
  background: var(--en-surface);
}

.en-card-popover button {
  min-height: 34px;
  border: 0;
  color: var(--en-text);
  background: transparent;
  text-align: left;
}

.en-card-popover button.danger {
  color: var(--en-danger, #ef4444);
}

.en-note-card h3 {
  min-width: 0;
  max-width: calc(100% - 42px);
  margin: 0;
  font-size: clamp(22px, 1.8vw, 30px);
  line-height: 1.12;
  overflow-wrap: anywhere;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.en-note-card-title-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.en-note-document-icon {
  width: 26px;
  height: 26px;
  flex: 0 0 auto;
  margin-top: 4px;
}
</style>
