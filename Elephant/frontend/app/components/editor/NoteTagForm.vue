<template>
  <form
    class="en-inline-tag-form"
    @click.stop
    @submit.prevent="submit"
  >
    <input
      :value="localValue"
      autofocus
      type="text"
      :placeholder="isEditing ? 'Edit tag' : 'Tag'"
      @input="updateValue($event.target.value)"
      @keydown.esc="$emit('cancel')"
      @keydown.enter.prevent="submit"
    >
    <button type="submit">
      Save
    </button>
    <button
      type="button"
      @click="$emit('cancel')"
    >
      Cancel
    </button>
  </form>
</template>

<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  modelValue: {
    type: String,
    default: ''
  },
  isEditing: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:modelValue', 'update:model-value', 'submit', 'cancel'])
const localValue = ref(props.modelValue)

watch(
  () => props.modelValue,
  (value) => {
    if (value !== localValue.value) localValue.value = value
  }
)

const updateValue = (value) => {
  localValue.value = value
  emit('update:modelValue', value)
  emit('update:model-value', value)
}

const submit = () => {
  emit('submit', localValue.value)
}
</script>

<style scoped>
.en-inline-tag-form {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.en-inline-tag-form input,
.en-inline-tag-form button {
  height: 30px;
  border: 1px solid var(--en-border);
  border-radius: 6px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
}
</style>
