<template>
  <div class="en-site-preview-toolbar">
    <div>
      <strong>{{ title }}</strong>
      <span v-if="url">{{ url }}</span>
    </div>
    <button
      v-if="url"
      type="button"
      class="en-ghost-button"
      @click="$emit('open-external')"
    >
      <ExternalLink class="en-icon" />
      Open in browser
    </button>
    <button
      type="button"
      class="en-note-exit-zone"
      aria-label="Close website preview"
      @click="$emit('close')"
    >
      <X class="en-icon" />
    </button>
  </div>
</template>

<script setup>
import { ExternalLink, X } from '@lucide/vue'

defineProps({
  title: {
    type: String,
    default: 'Website preview'
  },
  url: {
    type: String,
    default: ''
  }
})

defineEmits(['open-external', 'close'])
</script>

<style scoped>
.en-site-preview-toolbar {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 10px;
}

.en-site-preview-toolbar > div {
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.en-site-preview-toolbar strong {
  min-width: 0;
  color: var(--en-text);
  font-size: 15px;
  line-height: 1.2;
  white-space: nowrap;
}

.en-site-preview-toolbar span {
  min-width: 0;
  color: var(--en-muted);
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.en-ghost-button,
.en-note-exit-zone {
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
}

.en-ghost-button {
  gap: 8px;
  padding: 0 12px;
}

.en-note-exit-zone {
  width: 34px;
  padding: 0;
}

.en-ghost-button:hover,
.en-note-exit-zone:hover {
  background: var(--en-soft);
}

.en-icon {
  width: 18px;
  height: 18px;
  flex: 0 0 auto;
}

@media (max-width: 760px) {
  .en-site-preview-toolbar {
    grid-template-columns: minmax(0, 1fr) auto;
  }

  .en-ghost-button {
    display: none;
  }

  .en-site-preview-toolbar > div {
    display: grid;
    gap: 2px;
  }
}
</style>
