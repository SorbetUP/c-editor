<template>
  <div class="en-note-toolbar">
    <button
      v-for="item in visibleToolbarItems"
      :key="item.key"
      type="button"
      class="en-note-toolbar-button"
      :title="item.title"
      :aria-label="item.title"
      @click="$emit(item.event, item.payload)"
    >
      <component
        :is="item.icon"
        v-if="item.icon"
        class="en-icon"
      />
    </button>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import {
  Bold,
  Bot,
  Code2,
  Heading2,
  Image,
  Italic,
  Link2,
  List,
  ListOrdered,
  Mic,
  Minus,
  PenLine,
  Quote,
  SquareCheckBig,
  Sparkles,
  Strikethrough,
  Table2,
  Tags,
  Volume2
} from '@lucide/vue'
import { elephantnoteClient } from '../../services/elephantnoteClient'

defineEmits([
  'format',
  'paragraph',
  'insert-image',
  'insert-excalidraw',
  'insert-horizontal-rule',
  'ask-ai',
  'open-agents',
  'speech-to-text',
  'text-to-speech',
  'auto-tag'
])

const toolbarItems = [
  { key: 'heading-2', title: 'Heading 2', icon: Heading2, event: 'paragraph', payload: 'heading 2' },
  { key: 'bold', title: 'Bold', icon: Bold, event: 'format', payload: 'strong' },
  { key: 'italic', title: 'Italic', icon: Italic, event: 'format', payload: 'em' },
  { key: 'strike', title: 'Strikethrough', icon: Strikethrough, event: 'format', payload: 'del' },
  { key: 'link', title: 'Link', icon: Link2, event: 'format', payload: 'link' },
  { key: 'bullets', title: 'Bullet list', icon: List, event: 'paragraph', payload: 'ul-bullet' },
  { key: 'numbers', title: 'Numbered list', icon: ListOrdered, event: 'paragraph', payload: 'ol-bullet' },
  { key: 'tasks', title: 'Task list', icon: SquareCheckBig, event: 'paragraph', payload: 'ul-task' },
  { key: 'code', title: 'Inline code', icon: Code2, event: 'format', payload: 'inline_code' },
  { key: 'quote', title: 'Quote', icon: Quote, event: 'paragraph', payload: 'blockquote' },
  { key: 'table', title: 'Table', icon: Table2, event: 'paragraph', payload: 'table' },
  { key: 'image', title: 'Image', icon: Image, event: 'insert-image' },
  { key: 'excalidraw', title: 'Excalidraw', icon: PenLine, event: 'insert-excalidraw' },
  { key: 'rule', title: 'Horizontal rule', icon: Minus, event: 'insert-horizontal-rule' },
  { key: 'speech-to-text', title: 'Dictate', icon: Mic, event: 'speech-to-text' },
  { key: 'text-to-speech', title: 'Read aloud', icon: Volume2, event: 'text-to-speech' },
  { key: 'auto-tag', title: 'Auto tag', icon: Tags, event: 'auto-tag' },
  { key: 'ask-ai', title: 'Ask AI', icon: Sparkles, event: 'ask-ai' },
  { key: 'agents', title: 'Agents', icon: Bot, event: 'open-agents' }
]

const featureFlags = ref({
  askAi: true,
  agents: true
})

const visibleToolbarItems = computed(() => toolbarItems.filter((item) => {
  if (item.key === 'ask-ai') return featureFlags.value.askAi
  if (item.key === 'agents') return featureFlags.value.agents
  return true
}))

onMounted(async () => {
  try {
    featureFlags.value = await elephantnoteClient.features.get()
  } catch {
    featureFlags.value = { askAi: true, agents: true }
  }
  window.addEventListener('elephantnote:feature-flags-changed', handleFeatureFlagsChanged)
})

const handleFeatureFlagsChanged = (event) => {
  featureFlags.value = {
    ...featureFlags.value,
    ...(event.detail || {})
  }
}

onBeforeUnmount(() => {
  window.removeEventListener('elephantnote:feature-flags-changed', handleFeatureFlagsChanged)
})
</script>

<style scoped>
.en-note-toolbar {
  min-height: 56px;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 0 24px;
  border-bottom: 1px solid var(--en-border);
}

.en-note-toolbar-button {
  width: 34px;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 8px;
  color: var(--en-text);
  background: transparent;
}

.en-note-toolbar-button:hover {
  background: var(--en-soft);
}

.en-icon {
  width: 20px;
  height: 20px;
}
</style>
