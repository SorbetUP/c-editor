<template>
  <header
    ref="rootRef"
    class="en-note-topbar"
    @click.stop
  >
    <input
      class="en-note-title-input"
      type="text"
      :value="title"
      aria-label="Note title"
      @change="$emit('update-title', $event.target.value)"
    >

    <div class="en-note-chip-rail">
      <button
        v-if="noteDate"
        class="en-note-date-chip"
        type="button"
        :title="noteDate"
      >
        {{ noteDate }}
      </button>

      <template
        v-for="(tag, index) in visibleTags"
        :key="`${tag}-${index}`"
      >
        <div
          v-if="localAddingTag && localEditingTag && localEditingIndex === index"
          class="en-note-chip-wrap"
        >
          <note-tag-form
            :model-value="localTagDraft"
            :is-editing="true"
            @update:model-value="updateTagDraft"
            @submit="submitTag"
            @cancel="cancelTag"
          />
        </div>
        <button
          v-else
          class="en-note-chip"
          type="button"
          :title="`Click to edit, right-click to delete ${displayTag(tag)}`"
          @click="startTagEdit(index, tag)"
          @contextmenu.prevent.stop="$emit('delete-tag', index)"
        >
          {{ displayTag(tag) }}
        </button>
      </template>

      <div
        v-if="hiddenTags.length"
        class="en-note-chip-wrap"
      >
        <button
          class="en-note-chip en-note-chip-muted"
          type="button"
          :title="`${hiddenTags.length} hidden tags`"
          @click="isHiddenTagsOpen = !isHiddenTagsOpen"
        >
          +{{ hiddenTags.length }}
        </button>
        <div
          v-if="isHiddenTagsOpen"
          class="en-note-chip-popover"
        >
          <button
            v-for="(tag, index) in hiddenTags"
            :key="`${tag}-${index}`"
            type="button"
            @click="startHiddenTagEdit(index, tag)"
            @contextmenu.prevent.stop="$emit('delete-tag', index + visibleTags.length)"
          >
            {{ displayTag(tag) }}
          </button>
        </div>
      </div>

      <div class="en-note-chip-wrap">
        <button
          v-if="!localAddingTag"
          class="en-note-chip en-note-chip-add"
          type="button"
          title="Add tag"
          aria-label="Add tag"
          @click="startTagCreation"
        >
          +
        </button>
        <note-tag-form
          v-else-if="!localEditingTag || localEditingIndex >= visibleTags.length"
          :model-value="localTagDraft"
          :is-editing="localEditingTag"
          @update:model-value="updateTagDraft"
          @submit="submitTag"
          @cancel="cancelTag"
        />
      </div>
    </div>

    <div class="en-note-topbar-actions">
      <button
        class="en-note-action-button"
        :class="{ active: isPinned }"
        type="button"
        :title="isPinned ? 'Unpin note' : 'Pin note'"
        :aria-label="isPinned ? 'Unpin note' : 'Pin note'"
        @click="$emit('toggle-pin')"
      >
        <Pin class="en-icon" />
      </button>
      <button
        class="en-note-action-button"
        type="button"
        title="Close note"
        aria-label="Close note"
        @click="$emit('close')"
      >
        <X class="en-icon" />
      </button>
    </div>
  </header>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Pin, X } from '@lucide/vue'
import NoteTagForm from './NoteTagForm.vue'

const props = defineProps({
  title: {
    type: String,
    required: true
  },
  noteDate: {
    type: String,
    default: ''
  },
  tags: {
    type: Array,
    default: () => []
  },
  showTagHash: {
    type: Boolean,
    default: true
  },
  isPinned: {
    type: Boolean,
    default: false
  },
  isAddingTag: {
    type: Boolean,
    default: false
  },
  isEditingTag: {
    type: Boolean,
    default: false
  },
  tagDraft: {
    type: String,
    default: ''
  }
})

const emit = defineEmits([
  'update-title',
  'toggle-pin',
  'close',
  'start-tag-creation',
  'edit-tag',
  'delete-tag',
  'submit-tag',
  'cancel-tag',
  'update-tag-draft'
])

const rootRef = ref(null)
const localAddingTag = ref(props.isAddingTag)
const localEditingTag = ref(props.isEditingTag)
const localEditingIndex = ref(-1)
const localTagDraft = ref(props.tagDraft)
const isHiddenTagsOpen = ref(false)

const visibleTags = computed(() => props.tags.slice(0, 2))
const hiddenTags = computed(() => props.tags.slice(2))
const displayTag = (tag) => props.showTagHash ? `#${tag}` : tag

const focusFirstInput = async () => {
  await nextTick()
  rootRef.value?.querySelector?.('input[name="tag"]')?.focus?.()
}

const startTagCreation = () => {
  isHiddenTagsOpen.value = false
  localAddingTag.value = true
  localEditingTag.value = false
  localEditingIndex.value = -1
  localTagDraft.value = ''
  emit('start-tag-creation')
  emit('update-tag-draft', '')
  focusFirstInput()
}

const startTagEdit = (index, tag) => {
  isHiddenTagsOpen.value = false
  localAddingTag.value = true
  localEditingTag.value = true
  localEditingIndex.value = index
  localTagDraft.value = tag
  emit('edit-tag', index)
  emit('update-tag-draft', tag)
  focusFirstInput()
}

const startHiddenTagEdit = (index, tag) => {
  startTagEdit(index + visibleTags.value.length, tag)
}

const updateTagDraft = (value) => {
  localTagDraft.value = value
  emit('update-tag-draft', value)
}

const submitTag = (value = localTagDraft.value) => {
  localAddingTag.value = false
  localEditingTag.value = false
  localEditingIndex.value = -1
  emit('submit-tag', value)
}

const cancelTag = () => {
  localAddingTag.value = false
  localEditingTag.value = false
  localEditingIndex.value = -1
  localTagDraft.value = ''
  emit('cancel-tag')
}

const closeOnOutsideClick = (event) => {
  if (rootRef.value?.contains?.(event.target)) return
  isHiddenTagsOpen.value = false
}

onMounted(() => {
  window.addEventListener('click', closeOnOutsideClick)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeOnOutsideClick)
})

watch(
  () => props.isAddingTag,
  (value) => {
    localAddingTag.value = value
  }
)

watch(
  () => props.isEditingTag,
  (value) => {
    localEditingTag.value = value
    if (!value) localEditingIndex.value = -1
  }
)

watch(
  () => props.tagDraft,
  (value) => {
    localTagDraft.value = value
  }
)
</script>

<style scoped>
.en-note-topbar {
  position: relative;
  min-height: 58px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 var(--en-note-editor-gutter-right, var(--en-note-editor-gutter, 24px)) 0 var(--en-note-editor-gutter-left, var(--en-note-editor-gutter, 24px));
  color: var(--en-text);
  background: var(--en-bg);
  box-sizing: border-box;
  overflow: visible;
}

.en-note-topbar::after {
  content: '';
  position: absolute;
  left: var(--en-note-editor-gutter-left, var(--en-note-editor-gutter, 24px));
  right: var(--en-note-editor-gutter-right, var(--en-note-editor-gutter, 24px));
  bottom: 0;
  height: 1px;
  background: color-mix(in srgb, var(--en-border) 42%, transparent);
  pointer-events: none;
}

.en-note-title-input {
  min-width: 120px;
  flex: 1 1 auto;
  border: 0;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  font-size: 28px;
  font-weight: 800;
  outline: none;
}

.en-note-chip-rail,
.en-note-topbar-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 0 0 auto;
}

.en-note-chip-wrap {
  position: relative;
  display: inline-flex;
  align-items: center;
}

.en-note-date-chip,
.en-note-chip,
.en-note-action-button {
  height: 30px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  font-size: 14px;
  cursor: pointer;
  -webkit-app-region: no-drag;
}

.en-note-date-chip,
.en-note-chip {
  padding: 0 8px;
  white-space: nowrap;
}

.en-note-date-chip,
.en-note-chip-muted {
  color: var(--en-muted);
}

.en-note-chip-add,
.en-note-action-button {
  width: 30px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.en-note-date-chip:hover,
.en-note-chip:hover,
.en-note-action-button:hover {
  background: var(--en-soft);
}

.en-note-action-button.active {
  border-color: color-mix(in srgb, #facc15 54%, var(--en-border));
  color: #facc15;
}

.en-note-chip-popover {
  position: absolute;
  right: 0;
  top: calc(100% + 4px);
  z-index: 90;
  width: min(220px, calc(100vw - 24px));
  display: grid;
  gap: 3px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 4px;
  background: var(--en-surface);
  box-shadow: var(--en-card-shadow, 0 18px 44px rgba(0, 0, 0, 0.28));
}

.en-note-chip-popover button {
  min-height: 24px;
  border: 1px solid var(--en-border);
  border-radius: 6px;
  padding: 0 6px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  text-align: left;
}

.en-note-chip-popover button:hover {
  background: var(--en-soft);
}

:deep(.en-inline-tag-form) {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

:deep(.en-inline-tag-form input),
:deep(.en-inline-tag-form button) {
  height: 30px;
  min-width: 0;
  border: 1px solid var(--en-border);
  border-radius: 6px;
  padding: 0 6px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  font-size: 13px;
}

:deep(.en-inline-tag-form input) {
  width: 96px;
}

.en-icon {
  width: 18px;
  height: 18px;
}

@media (max-width: 900px) {
  .en-note-topbar {
    flex-wrap: wrap;
    align-content: center;
    padding: 6px var(--en-note-editor-gutter-right, 24px) 6px var(--en-note-editor-gutter-left, 32px);
  }

  .en-note-title-input {
    flex-basis: 100%;
  }
}
</style>
