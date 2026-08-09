<script setup>
import { computed } from 'vue'
import '@marktext/file-icons/build/index.css'
import fileIcons from '@marktext/file-icons'

const props = defineProps({
  name: {
    type: String,
    required: true
  }
})

const getClassByName = (name) => {
  const icon = fileIcons.matchName(name)
  return icon ? icon.getClass(0, false) : null
}

const className = computed(() => {
  let classNames = getClassByName(props.name ? props.name : 'mock.md')

  if (!classNames) {
    // Use fallback icon when the icon is unknown.
    classNames = getClassByName('mock.md')
  }
  return classNames.split(/\s/)
})
</script>

<template>
  <span
    :class="className"
    class="file-icon"
  />
</template>

<style scoped>
.file-icon {
  flex-shrink: 0;
  margin-right: 5px;
}
</style>
