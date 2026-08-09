<template>
  <div class="en-note-meta">
    <span>{{ noteDate }}</span>
    <template
      v-for="(tag, index) in tags"
      :key="`${tag}-${index}`"
    >
      <div
        v-if="localAddingTag && localEditingTag && localEditingIndex === index"
        class="en-inline-tag-editor"
      >
        <input
          ref="tagInput"
          name="tag"
          type="text"
          autofocus
          :value="localTagDraft"
          placeholder="Tag"
          @click.stop
          @input="updateTagDraft($event.target.value)"
          @keydown.enter.prevent="completeTagEdit"
          @keydown.esc.prevent="cancelTag"
          @blur="completeTagEdit"
        >
      </div>
      <note-tag-chip
        v-else
        :tag="tag"
        :show-hash="showTagHash"
        @edit="startTagEdit(index, tag)"
        @delete="$emit('delete-tag', index)"
      />
    </template>
    <button
      v-if="!localAddingTag"
      class="en-add-tag-button"
      type="button"
      @pointerdown.stop
      @mousedown.stop
      @click.stop.prevent="startTagCreation"
    >
      + Add tag
    </button>
    <div
      v-else-if="!localEditingTag"
      class="en-inline-tag-editor"
    >
      <input
        ref="tagInput"
        name="tag"
        type="text"
        autofocus
        :value="localTagDraft"
        placeholder="Tag"
        @click.stop
        @input="updateTagDraft($event.target.value)"
        @keydown.enter.prevent="completeTagEdit"
        @keydown.esc.prevent="cancelTag"
        @blur="completeTagEdit"
      >
    </div>
  </div>
</template>

<script setup>
import { nextTick, ref, watch } from 'vue'
import NoteTagChip from './NoteTagChip.vue'

const props = defineProps({
  noteDate: {
    type: String,
    required: true
  },
  tags: {
    type: Array,
    default: () => []
  },
  showTagHash: {
    type: Boolean,
    default: true
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
  'start-tag-creation',
  'edit-tag',
  'delete-tag',
  'submit-tag',
  'cancel-tag',
  'update-tag-draft'
])

const localAddingTag = ref(props.isAddingTag)
const localEditingTag = ref(props.isEditingTag)
const localEditingIndex = ref(-1)
const localTagDraft = ref(props.tagDraft)
const tagInput = ref(null)

const focusTagInput = async () => {
  await nextTick()
  tagInput.value?.focus()
  tagInput.value?.select()
}

const startTagCreation = () => {
  localAddingTag.value = true
  localEditingTag.value = false
  localEditingIndex.value = -1
  localTagDraft.value = ''
  emit('start-tag-creation')
  emit('update-tag-draft', '')
  focusTagInput()
}

const startTagEdit = (index, tag) => {
  localAddingTag.value = true
  localEditingTag.value = true
  localEditingIndex.value = index
  localTagDraft.value = tag
  emit('edit-tag', index)
  emit('update-tag-draft', tag)
  focusTagInput()
}

const updateTagDraft = (value) => {
  localTagDraft.value = value
  emit('update-tag-draft', value)
}

const cancelTag = () => {
  localAddingTag.value = false
  localEditingTag.value = false
  localEditingIndex.value = -1
  localTagDraft.value = ''
  emit('cancel-tag')
}

const submitTag = (value = localTagDraft.value) => {
  localAddingTag.value = false
  localEditingTag.value = false
  localEditingIndex.value = -1
  emit('submit-tag', value)
}

const completeTagEdit = () => {
  if (!localAddingTag.value) return

  const value = String(localTagDraft.value || '').trim()
  if (value) {
    submitTag(value)
    return
  }

  cancelTag()
}

watch(
  () => props.isAddingTag,
  (value) => {
    localAddingTag.value = value
    if (value) focusTagInput()
    if (!value) localEditingIndex.value = -1
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
.en-note-meta {
  position: relative;
  z-index: 20;
  min-height: 48px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 24px;
  border-bottom: 1px solid var(--en-border);
  color: var(--en-muted);
  pointer-events: auto;
  -webkit-app-region: no-drag;
}

.en-add-tag-button {
  position: relative;
  z-index: 21;
  height: 30px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 0 10px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  cursor: pointer;
  pointer-events: auto;
  user-select: none;
  -webkit-app-region: no-drag;
}

.en-add-tag-button:hover {
  background: var(--en-soft);
}

.en-inline-tag-editor {
  position: relative;
  z-index: 21;
  display: inline-flex;
  align-items: center;
  pointer-events: auto;
  -webkit-app-region: no-drag;
}

.en-inline-tag-editor input {
  height: 30px;
  width: 112px;
  min-width: 72px;
  max-width: 180px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 0 10px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  pointer-events: auto;
  -webkit-app-region: no-drag;
}
</style>
