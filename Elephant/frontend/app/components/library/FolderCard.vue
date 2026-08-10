<template>
  <article
    class="en-card en-folder-card"
    :class="{ 'is-pinned': isPinned }"
    draggable="true"
    @mouseenter="isHovering = true"
    @mouseleave="isHovering = false"
    @dragstart="handleDragStart"
    @click="handleCardClick"
  >
    <div class="en-card-actions">
      <button
        class="en-card-pin-button"
        type="button"
        :class="{ visible: isPinned || isHovering }"
        :title="isPinned ? 'Unpin folder' : 'Pin folder'"
        :aria-label="isPinned ? 'Unpin folder' : 'Pin folder'"
        @click.stop.prevent="togglePin"
      >
        <Pin class="en-icon" />
      </button>
      <button
        class="en-card-menu"
        :class="{ visible: isMenuOpen || isHovering }"
        type="button"
        :title="isMenuOpen ? 'Close folder actions' : 'Folder actions'"
        :aria-label="isMenuOpen ? 'Close folder actions' : 'Folder actions'"
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
        title="Rename folder"
        aria-label="Rename folder"
        @click.stop.prevent="beginRename"
      >
        <PencilLine class="en-icon" />
      </button>
      <button
        type="button"
        :title="isSidebarVisible ? 'Hide from sidebar' : 'Show in sidebar'"
        :aria-label="isSidebarVisible ? 'Hide from sidebar' : 'Show in sidebar'"
        @click.stop.prevent="toggleSidebarVisibility"
      >
        <component
          :is="isSidebarVisible ? EyeOff : Eye"
          class="en-icon"
        />
      </button>
      <button
        type="button"
        class="danger"
        title="Delete folder"
        aria-label="Delete folder"
        @click.stop.prevent="deleteFolder"
      >
        <Trash2 class="en-icon" />
      </button>
    </div>
    <div class="en-card-topline">
      <div class="en-folder-icon" />
    </div>
    <input
      v-if="isRenaming"
      ref="renameInput"
      v-model="renameDraft"
      class="en-folder-title-input"
      data-entry-rename-input
      type="text"
      aria-label="Rename folder"
      @click.stop
      @keydown.enter.stop.prevent="commitRename"
      @keydown.esc.stop.prevent="cancelRename"
    >
    <h3
      v-else
      @dblclick.stop.prevent="beginRename"
    >
      {{ entry.title }}
    </h3>
    <p>{{ entry.noteCount }} notes</p>
    <div
      v-if="previewItems.length"
      class="en-folder-preview"
      aria-label="Folder contents preview"
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
    </div>
    <div
      v-else
      class="en-folder-preview is-empty"
      aria-label="Empty folder"
    >
      <span>No items yet</span>
    </div>
  </article>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { Eye, EyeOff, FileText, Folder, MoreHorizontal, PenLine, Pin, PencilLine, Trash2 } from '@lucide/vue'
import { useVaultStore } from '../../stores/vaultStore'

const props = defineProps({
  entry: {
    type: Object,
    required: true
  }
})
const emit = defineEmits(['open', 'rename', 'delete'])
const isMenuOpen = ref(false)
const isHovering = ref(false)
const isRenaming = ref(false)
const renameDraft = ref('')
const renameInput = ref(null)
const store = useVaultStore()
const isPinned = computed(() => !!props.entry?.path && store.isEntryPinned(props.entry.path))
const isSidebarVisible = computed(() => !!props.entry?.path && store.isFolderVisibleInSidebar(props.entry.path))
const previewItems = computed(() => Array.isArray(props.entry?.childrenPreview)
  ? props.entry.childrenPreview.slice(0, 3)
  : [])

const previewItemTitle = (item) => String(item?.title || 'Untitled')
  .replace(/\.(?:md|excalidraw)$/i, '')

const previewItemIcon = (item) => {
  if (item?.type === 'folder') return Folder
  if (item?.type === 'drawing' || /\.excalidraw(?:\.png)?$/i.test(String(item?.title || ''))) return PenLine
  return FileText
}

const toggleMenu = () => {
  isMenuOpen.value = !isMenuOpen.value
}

const beginRename = async () => {
  isMenuOpen.value = false
  isRenaming.value = true
  renameDraft.value = props.entry?.title || ''
  await nextTick()
  renameInput.value?.focus?.()
  renameInput.value?.select?.()
}

const cancelRename = () => {
  isRenaming.value = false
  renameDraft.value = ''
}

const commitRename = () => {
  const title = renameDraft.value.trim()
  const previousTitle = props.entry?.title || ''
  cancelRename()
  if (!title || title === previousTitle) return
  emit('rename', { entry: props.entry, title })
}

const handleCardClick = () => {
  if (isRenaming.value) {
    cancelRename()
    return
  }
  emit('open', props.entry)
}

const togglePin = () => {
  if (!props.entry?.path) return
  store.togglePinnedEntry(props.entry.path)
  isMenuOpen.value = false
}

const toggleSidebarVisibility = async () => {
  if (!props.entry?.path) return
  try {
    await store.toggleEntrySidebarVisibility(props.entry)
  } finally {
    isMenuOpen.value = false
  }
}

const handleDragStart = (event) => {
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'copy'
  }
  event.dataTransfer?.setData('application/x-elephantnote-entry', JSON.stringify({
    kind: 'folder',
    path: props.entry.path,
    title: props.entry.title
  }))
}

const deleteFolder = () => {
  isMenuOpen.value = false
  emit('delete', props.entry)
}

const closeMenu = (event) => {
  const target = event?.target
  if (isRenaming.value && !target?.closest?.('[data-entry-rename-input]')) {
    cancelRename()
  }
  if (!isMenuOpen.value) return
  if (target?.closest?.('.en-folder-card')) return
  isMenuOpen.value = false
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
  min-height: 176px;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 10px;
  color: var(--en-text);
  background: var(--en-bg);
  overflow: hidden;
}

.en-card:hover {
  border-color: var(--en-border-strong);
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
}

.en-card-pin-button.visible,
.en-card-menu.visible {
  opacity: 1;
}

.en-card-pin-button:not(.visible),
.en-card-menu:not(.visible) {
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

.en-card-popover .danger {
  color: #ff6b6b;
}

.en-folder-icon {
  position: relative;
  width: 42px;
  height: 32px;
  margin-bottom: 10px;
  border-radius: 6px;
  background: linear-gradient(180deg, #3d95ff 0%, #1d63f0 100%);
}

.en-folder-icon::before {
  content: "";
  position: absolute;
  left: 3px;
  top: -7px;
  width: 20px;
  height: 11px;
  border-radius: 5px 5px 0 0;
  background: #65adff;
}

.en-folder-card h3 {
  margin: 0 0 6px;
  font-size: clamp(17px, 1.6vw, 24px);
  line-height: 1.1;
  overflow-wrap: anywhere;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.en-folder-card p {
  margin: 0 0 8px;
  color: color-mix(in srgb, var(--en-text) 90%, transparent);
  font-size: 15px;
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

.en-folder-title-input {
  min-width: 0;
  width: 100%;
  margin: 0 0 6px;
  border: 1px solid var(--en-primary);
  border-radius: 7px;
  padding: 4px 6px;
  color: var(--en-text);
  background: var(--en-input-bg, var(--en-surface));
  font: inherit;
  font-size: clamp(17px, 1.6vw, 24px);
  font-weight: 800;
  outline: none;
}

.en-icon {
  width: 20px;
  height: 20px;
}
</style>
