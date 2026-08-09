<template>
  <form
    v-if="image"
    class="rust-image-toolbar"
    :style="position"
    @mousedown.stop
    @submit.prevent="apply"
  >
    <div class="rust-image-toolbar__header">
      <strong>Image</strong>
      <button
        type="button"
        class="rust-image-toolbar__icon"
        aria-label="Close image toolbar"
        @click="$emit('close')"
      >
        ×
      </button>
    </div>
    <label>
      <span>Source</span>
      <input
        v-model="form.source"
        type="text"
        autocomplete="off"
      >
    </label>
    <label>
      <span>Alt text</span>
      <input
        v-model="form.alt"
        type="text"
        autocomplete="off"
      >
    </label>
    <label>
      <span>Title</span>
      <input
        v-model="form.title"
        type="text"
        autocomplete="off"
      >
    </label>
    <div class="rust-image-toolbar__actions">
      <button
        type="button"
        @click="$emit('choose-file', image)"
      >
        Choose file
      </button>
      <button
        type="button"
        class="danger"
        @click="$emit('delete', image)"
      >
        Delete
      </button>
      <button type="submit">
        Apply
      </button>
    </div>
  </form>
</template>

<script setup>
import { computed, reactive, watch } from 'vue'

const props = defineProps({
  image: { type: Object, default: null }
})

const emit = defineEmits(['apply', 'choose-file', 'delete', 'close'])
const form = reactive({ source: '', alt: '', title: '' })

watch(
  () => props.image,
  (image) => {
    form.source = image?.source || ''
    form.alt = image?.alt || ''
    form.title = image?.title || ''
  },
  { immediate: true }
)

const position = computed(() => {
  const rect = props.image?.rect
  if (!rect) return {}
  const width = 360
  const left = Math.max(12, Math.min(rect.left, window.innerWidth - width - 12))
  const top = Math.max(12, rect.top - 196)
  return { left: `${left}px`, top: `${top}px`, width: `${width}px` }
})

const apply = () => {
  if (!props.image || !form.source.trim()) return
  emit('apply', {
    image: props.image.image,
    source: form.source.trim(),
    alt: form.alt,
    title: form.title
  })
}
</script>

<style scoped>
.rust-image-toolbar {
  position: fixed;
  z-index: 2200;
  display: grid;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--floatBorderColor, rgba(127, 127, 127, 0.35));
  border-radius: 8px;
  background: var(--floatBgColor, var(--editorBgColor));
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.2);
}

.rust-image-toolbar__header,
.rust-image-toolbar__actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.rust-image-toolbar__header {
  justify-content: space-between;
}

.rust-image-toolbar label {
  display: grid;
  gap: 4px;
  font-size: 12px;
}

.rust-image-toolbar input {
  min-width: 0;
  padding: 7px 8px;
  border: 1px solid var(--inputBorderColor, rgba(127, 127, 127, 0.35));
  border-radius: 5px;
  background: var(--inputBgColor, transparent);
  color: inherit;
}

.rust-image-toolbar button {
  padding: 6px 9px;
  border: 1px solid var(--buttonBorderColor, rgba(127, 127, 127, 0.35));
  border-radius: 5px;
  background: var(--buttonBgColor, transparent);
  color: inherit;
  cursor: pointer;
}

.rust-image-toolbar__actions {
  justify-content: flex-end;
}

.rust-image-toolbar__icon {
  border: 0;
  font-size: 18px;
  line-height: 1;
}

.rust-image-toolbar .danger {
  color: var(--errorColor, #c33);
}
</style>
