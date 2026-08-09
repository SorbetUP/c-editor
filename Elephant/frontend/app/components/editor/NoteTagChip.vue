<template>
  <div
    ref="tagRoot"
    class="en-note-tag"
  >
    <button
      type="button"
      class="en-note-tag-label"
      :title="`Double-click to edit ${displayTag}`"
      :aria-label="`Edit tag ${tag}`"
      @dblclick="$emit('edit')"
      @contextmenu.prevent.stop="openMenu"
    >
      {{ displayTag }}
    </button>
    <div
      v-if="isMenuOpen"
      class="en-note-tag-menu"
      @click.stop
      @contextmenu.prevent.stop
    >
      <p>Delete {{ displayTag }}?</p>
      <div class="en-note-tag-menu-actions">
        <button
          type="button"
          @click="closeMenu"
        >
          Cancel
        </button>
        <button
          type="button"
          class="danger"
          @click="confirmDelete"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

const props = defineProps({
  tag: {
    type: String,
    required: true
  },
  showHash: {
    type: Boolean,
    default: true
  }
})

const emit = defineEmits(['edit', 'delete'])
const isMenuOpen = ref(false)
const tagRoot = ref(null)
const displayTag = computed(() => props.showHash ? `#${props.tag}` : props.tag)

const openMenu = () => {
  isMenuOpen.value = true
}

const closeMenu = () => {
  isMenuOpen.value = false
}

const confirmDelete = () => {
  closeMenu()
  emit('delete')
}

const closeOnOutsideClick = (event) => {
  if (!isMenuOpen.value) return
  if (tagRoot.value?.contains?.(event.target)) return
  closeMenu()
}

onMounted(() => {
  window.addEventListener('click', closeOnOutsideClick)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeOnOutsideClick)
})
</script>

<style scoped>
.en-note-tag {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.en-note-tag-label {
  height: 30px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 0 10px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  cursor: pointer;
  -webkit-app-region: no-drag;
}

.en-note-tag-label:hover {
  background: var(--en-soft);
}

.en-note-tag-menu {
  position: absolute;
  left: 0;
  top: calc(100% + 6px);
  z-index: 80;
  min-width: 188px;
  display: grid;
  gap: 10px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 10px;
  color: var(--en-text);
  background: var(--en-surface);
  box-shadow: var(--en-card-shadow, 0 18px 44px rgba(0, 0, 0, 0.28));
}

.en-note-tag-menu p {
  margin: 0;
  color: var(--en-muted);
  font-size: 13px;
  font-weight: 700;
}

.en-note-tag-menu-actions {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}

.en-note-tag-menu-actions button {
  height: 30px;
  border: 1px solid var(--en-border);
  border-radius: 6px;
  padding: 0 9px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  cursor: pointer;
}

.en-note-tag-menu-actions button:hover {
  background: var(--en-soft);
}

.en-note-tag-menu-actions .danger {
  border-color: color-mix(in srgb, var(--en-danger, #ff6b7a) 50%, var(--en-border));
  color: var(--en-danger, #ff6b7a);
}
</style>
