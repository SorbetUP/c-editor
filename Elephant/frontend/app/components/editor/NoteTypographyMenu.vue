<template>
  <div class="en-note-menu-wrap">
    <button
      type="button"
      title="Text display"
      @click="$emit('toggle')"
    >
      Aa
    </button>
    <div
      v-if="isOpen"
      class="en-note-popover en-typography-menu"
    >
      <button
        type="button"
        @click="$emit('set-text-scale', 'compact')"
      >
        Compact
      </button>
      <button
        type="button"
        @click="$emit('set-text-scale', 'normal')"
      >
        Normal
      </button>
      <button
        type="button"
        @click="$emit('set-text-scale', 'large')"
      >
        Large
      </button>
    </div>
  </div>
</template>

<script setup>
defineProps({
  isOpen: {
    type: Boolean,
    default: false
  }
})

defineEmits(['toggle', 'set-text-scale'])
</script>

<style scoped>
.en-note-menu-wrap {
  position: relative;
}

.en-note-menu-wrap > button {
  width: 36px;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 0;
  color: var(--en-text);
  background: transparent;
  font: inherit;
}

.en-note-menu-wrap > button:hover {
  background: var(--en-soft);
}

.en-note-popover {
  position: absolute;
  right: 0;
  bottom: 42px;
  z-index: 30;
  min-width: 150px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 8px;
  background: var(--en-surface);
}

.en-note-popover button {
  min-height: 32px;
  border: 0;
  border-radius: 6px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  text-align: left;
}
</style>
