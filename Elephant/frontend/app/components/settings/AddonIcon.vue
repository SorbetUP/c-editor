<template>
  <component :is="iconComponent" aria-hidden="true" />
</template>

<script setup>
import { computed } from 'vue'
import {
  BookOpenText,
  Bot,
  CalendarClock,
  CalendarDays,
  Cloud,
  Database,
  Download,
  GitFork,
  Globe2,
  Layers3,
  MessageCircle,
  Package,
  ScanText,
  Search,
  Sparkles,
  Terminal
} from '@lucide/vue'
import ExcalidrawMark from './ExcalidrawMark.vue'
import OpenAiMark from './OpenAiMark.vue'

const props = defineProps({ name: { type: String, default: 'package' } })

const ICONS = Object.freeze({
  bot: Bot,
  'book-open-text': BookOpenText,
  'calendar-clock': CalendarClock,
  calendar: CalendarDays,
  'calendar-days': CalendarDays,
  cloud: Cloud,
  database: Database,
  download: Download,
  excalidraw: ExcalidrawMark,
  'git-fork': GitFork,
  globe: Globe2,
  'globe-2': Globe2,
  layers: Layers3,
  'layers-3': Layers3,
  'message-circle': MessageCircle,
  openai: OpenAiMark,
  package: Package,
  'scan-text': ScanText,
  search: Search,
  sparkles: Sparkles,
  terminal: Terminal
})

const iconComponent = computed(() => ICONS[String(props.name || '').toLowerCase()] || Package)
</script>
