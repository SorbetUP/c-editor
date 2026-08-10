<template>
  <div
    ref="root"
    class="en-create-menu"
    :class="{ 'en-create-menu-mobile': mobile }"
  >
    <slot
      name="trigger"
      :toggle="toggle"
      :open="isOpen"
      :disabled="disabled"
    />
    <div
      v-if="isOpen"
      class="en-create-menu-popover"
      role="menu"
      aria-label="Create"
    >
      <div class="en-create-menu-heading">
        Create
      </div>
      <button
        v-for="entry in entries"
        :key="entry.key"
        class="en-create-menu-option"
        type="button"
        role="menuitem"
        @click="select(entry.key)"
      >
        <img
          v-if="entry.key === 'drawing'"
          class="en-create-menu-icon en-excalidraw-logo"
          data-testid="excalidraw-logo"
          data-excalidraw-asset="shared-muya-icon"
          :src="excalidrawLogo"
          alt=""
          aria-hidden="true"
        >
        <component
          :is="entry.icon"
          v-else
          class="en-create-menu-icon"
          aria-hidden="true"
        />
        <span class="en-create-menu-copy">
          <strong>{{ entry.label }}</strong>
          <small>{{ entry.description }}</small>
        </span>
      </button>
    </div>
  </div>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { FilePlus2, FolderPlus, PenLine } from '@lucide/vue'
import excalidrawLogo from '../../../src/muya/lib/assets/icons/excalidraw.svg?url'

const props = defineProps({
  disabled: { type: Boolean, default: false },
  mobile: { type: Boolean, default: false }
})

const emit = defineEmits(['select'])
const root = ref(null)
const isOpen = ref(false)
const entries = [
  { key: 'note', label: 'Note', description: 'Create a new note', icon: FilePlus2 },
  { key: 'drawing', label: 'Drawing', description: 'Open a new Excalidraw canvas', icon: PenLine },
  { key: 'folder', label: 'Folder', description: 'Organize notes in a folder', icon: FolderPlus }
]

const toggle = () => {
  if (props.disabled) return
  if (!isOpen.value) {
    isOpen.value = true
    return
  }
  isOpen.value = false
}

const select = (key) => {
  isOpen.value = false
  emit('select', key)
}

const closeOnOutsidePointer = (event) => {
  if (isOpen.value && !root.value?.contains(event.target)) isOpen.value = false
}

const closeOnEscape = (event) => {
  if (event.key === 'Escape') isOpen.value = false
}

onMounted(() => {
  document.addEventListener('pointerdown', closeOnOutsidePointer)
  document.addEventListener('keydown', closeOnEscape)
})

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', closeOnOutsidePointer)
  document.removeEventListener('keydown', closeOnEscape)
})
</script>

<style scoped>
.en-create-menu {
  position: relative;
  display: inline-flex;
}

.en-create-menu-popover {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  z-index: 1200;
  width: 280px;
  padding: 8px;
  border: 1px solid var(--en-border-strong, var(--en-border));
  border-radius: 14px;
  background: var(--en-surface, #242424);
  box-shadow: 0 16px 36px rgb(0 0 0 / 28%);
}

.en-create-menu-heading {
  padding: 8px 10px 6px;
  color: var(--en-muted);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.en-create-menu-option {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px;
  border: 0;
  border-radius: 10px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.en-create-menu-option:hover,
.en-create-menu-option:focus-visible {
  background: var(--en-soft);
  outline: none;
}

.en-create-menu-icon {
  width: 20px;
  height: 20px;
  flex: 0 0 auto;
  color: var(--en-primary);
}

.en-create-menu-copy {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.en-create-menu-copy strong {
  font-size: 14px;
  font-weight: 700;
}

.en-create-menu-copy small {
  color: var(--en-muted);
  font-size: 12px;
}

.en-create-menu-mobile {
  position: fixed;
  right: max(20px, env(safe-area-inset-right));
  bottom: max(20px, env(safe-area-inset-bottom));
  z-index: 1200;
}

.en-excalidraw-logo {
  border-radius: 6px;
  object-fit: contain;
}

.en-create-menu-mobile .en-create-menu-popover {
  top: auto;
  right: 0;
  bottom: calc(100% + 10px);
  left: auto;
}
</style>
