<template>
  <footer class="en-note-footer">
    <div class="en-note-counts">
      <span>{{ wordCount }} words</span>
      <span>{{ characterCount }} characters</span>
    </div>
    <div class="en-note-footer-actions">
      <component
        :is="entry.contribution.component"
        v-for="entry in footerItems"
        :key="entry.contribution.id"
      />
      <note-typography-menu
        :is-open="isTypographyOpen"
        @toggle="$emit('toggle-typography')"
        @set-text-scale="$emit('set-text-scale', $event)"
      />
      <button
        type="button"
        title="Toggle theme"
        @click="$emit('toggle-theme')"
      >
        <component
          :is="themeIcon"
          class="en-icon"
        />
      </button>
    </div>
  </footer>
</template>

<script setup>
import { computed } from 'vue'
import { useAddonsStore } from '@/store/addons'
import NoteTypographyMenu from './NoteTypographyMenu.vue'

const addonsStore = useAddonsStore()
const footerItems = computed(() => addonsStore.getContributions('editor.footer-items')
  .filter((entry) => entry?.contribution?.id && entry?.contribution?.component)
  .sort((left, right) => Number(left.contribution.order || 0) - Number(right.contribution.order || 0)))

defineProps({
  wordCount: {
    type: Number,
    default: 0
  },
  characterCount: {
    type: Number,
    default: 0
  },
  isTypographyOpen: {
    type: Boolean,
    default: false
  },
  themeIcon: {
    type: [Object, Function, String],
    required: true
  }
})

defineEmits(['toggle-typography', 'set-text-scale', 'toggle-theme'])
</script>

<style scoped>
.en-note-footer {
  min-height: 50px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 24px;
  border-top: 1px solid var(--en-border);
  color: var(--en-muted);
}

.en-note-footer-actions,
.en-note-counts {
  display: flex;
  align-items: center;
}

.en-note-footer-actions {
  gap: 8px;
}

.en-note-counts {
  gap: 12px;
}

.en-note-footer-actions :deep(button) {
  min-width: 36px;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 0 10px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  font-size: 13px;
}

.en-note-footer-actions :deep(button:hover) {
  background: var(--en-soft);
}

.en-icon {
  width: 20px;
  height: 20px;
}
</style>
