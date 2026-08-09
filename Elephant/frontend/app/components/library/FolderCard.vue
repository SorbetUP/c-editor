<template>
  <article
    class="en-card en-folder-card"
    :class="{ 'is-pinned': isPinned }"
    draggable="true"
    @mouseenter="isHovering = true"
    @mouseleave="isHovering = false"
    @dragstart="handleDragStart"
    @click="$emit('open', entry)"
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
        @click="renameFolder"
      >
        <PencilLine class="en-icon" />
        Edit
      </button>
      <button
        type="button"
        @click="toggleSidebarVisibility"
      >
        <component
          :is="isSidebarVisible ? EyeOff : Eye"
          class="en-icon"
        />
        {{ isSidebarVisible ? 'Remove from sidebar' : 'Show in sidebar' }}
      </button>
      <button
        type="button"
        class="danger"
        @click="deleteFolder"
      >
        Delete
      </button>
    </div>
    <div class="en-card-topline">
      <div class="en-folder-icon" />
    </div>
    <h3
      @dblclick.stop.prevent="renameFolder"
    >
      {{ entry.title }}
    </h3>
    <p>{{ entry.noteCount }} notes</p>
    <span class="en-updated">Updated {{ updated }}</span>
  </article>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { Eye, EyeOff, MoreHorizontal, Pin, PencilLine } from '@lucide/vue'
import { useVaultStore } from '../../stores/vaultStore'
import { formatShortDate } from '../../services/markdownMetaService'

const props = defineProps({
  entry: {
    type: Object,
    required: true
  }
})
const emit = defineEmits(['open', 'rename', 'delete'])
const isMenuOpen = ref(false)
const isHovering = ref(false)
const store = useVaultStore()
const updated = computed(() => formatShortDate(props.entry.updatedAt))
const isPinned = computed(() => !!props.entry?.path && store.isEntryPinned(props.entry.path))
const isSidebarVisible = computed(() => !!props.entry?.path && store.isFolderVisibleInSidebar(props.entry.path))

const toggleMenu = () => {
  isMenuOpen.value = !isMenuOpen.value
}

const renameFolder = () => {
  isMenuOpen.value = false
  emit('rename', props.entry)
}

const togglePin = () => {
  if (!props.entry?.path) return
  store.togglePinnedEntry(props.entry.path)
  isMenuOpen.value = false
}

const toggleSidebarVisibility = async () => {
  if (!props.entry?.path) return
  await store.toggleEntrySidebarVisibility(props.entry)
  isMenuOpen.value = false
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
  if (!isMenuOpen.value) return
  if (event?.target?.closest?.('.en-folder-card')) return
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
  min-height: 168px;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 18px;
  color: var(--en-text);
  background: var(--en-bg);
  overflow: hidden;
}

.en-card:hover {
  border-color: var(--en-border-strong);
}

.en-card-actions {
  position: absolute;
  top: 18px;
  right: 18px;
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
  top: 52px;
  right: 18px;
  z-index: 5;
  min-width: 210px;
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
  display: flex;
  align-items: center;
  gap: 8px;
  border: 0;
  color: var(--en-text);
  background: transparent;
  text-align: left;
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
  margin: 0 0 8px;
  font-size: clamp(18px, 1.8vw, 28px);
  line-height: 1.1;
  overflow-wrap: anywhere;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.en-folder-card p {
  margin: 0;
  color: color-mix(in srgb, var(--en-text) 90%, transparent);
  font-size: 18px;
}

.en-updated {
  display: block;
  margin-top: auto;
  padding-top: 14px;
  color: var(--en-muted);
  font-weight: 700;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.en-icon {
  width: 20px;
  height: 20px;
}
</style>
