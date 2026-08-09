<template>
  <div
    class="en-search-status"
    :class="`is-${statusValue}`"
  >
    <span class="en-search-status-dot" />
    <span class="en-search-status-label">{{ label }}</span>
    <span
      v-if="counts"
      class="en-search-status-counts"
    >
      {{ counts }}
    </span>
    <span
      v-if="message"
      class="en-search-status-message"
    >
      {{ message }}
    </span>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  status: {
    type: Object,
    default: () => ({})
  }
})

const statusValue = computed(() => props.status?.status || 'not_initialized')
const labelMap = {
  disabled: 'Disabled',
  not_initialized: 'Not initialized',
  model_missing: 'Model missing',
  model_loading: 'Loading model',
  indexing: 'Indexing',
  ready: 'Ready',
  error: 'Error'
}

const label = computed(() => labelMap[statusValue.value] || 'Unknown')
const counts = computed(() => {
  const indexed = Number(props.status?.indexedDocuments || 0)
  const total = Number(props.status?.totalDocuments || 0)
  if (!indexed && !total) return ''
  return `${indexed}/${total}`
})
const message = computed(() => props.status?.message || props.status?.error || '')
</script>

<style scoped>
.en-search-status {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 56px;
  padding: 10px 14px;
  border-radius: 14px;
  border: 1px solid var(--en-border);
  background: var(--en-surface);
  color: var(--en-muted);
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
}

.en-search-status-dot {
  width: 8px;
  height: 8px;
  flex: 0 0 auto;
  border-radius: 999px;
  background: var(--en-subtle);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--en-subtle) 13%, transparent);
}

.en-search-status.is-ready {
  color: var(--en-primary);
}

.en-search-status.is-ready .en-search-status-dot {
  background: #16a34a;
  box-shadow: 0 0 0 4px color-mix(in srgb, #16a34a 15%, transparent);
}

.en-search-status.is-indexing .en-search-status-dot,
.en-search-status.is-model_loading .en-search-status-dot {
  background: #d97706;
  box-shadow: 0 0 0 4px color-mix(in srgb, #d97706 16%, transparent);
}

.en-search-status.is-error {
  color: #ef4444;
}

.en-search-status.is-error .en-search-status-dot {
  background: #ef4444;
  box-shadow: 0 0 0 4px color-mix(in srgb, #ef4444 14%, transparent);
}

.en-search-status-counts,
.en-search-status-message {
  font-weight: 600;
  color: var(--en-subtle);
}

.en-search-status-message {
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
