<template>
  <article
    class="en-search-result"
    :class="{ selected }"
    @click="$emit('open', result)"
  >
    <div class="en-search-result-icon">
      <FileText />
    </div>

    <div class="en-search-result-body">
      <div class="en-search-result-headline">
        <h3 class="en-search-result-title">
          <template v-if="query && titleSegments.length">
            <span
              v-for="(segment, index) in titleSegments"
              :key="`title-${index}`"
              :class="{ 'en-search-result-match': segment.match }"
            >{{ segment.text }}</span>
          </template>
          <template v-else>
            {{ result.title }}
          </template>
        </h3>
        <span class="en-search-result-badge">{{ matchLabel }}</span>
      </div>

      <div
        v-if="result.relativePath"
        class="en-search-result-path"
      >
        {{ result.relativePath }}
      </div>

      <p
        v-for="(snippet, index) in displaySnippets"
        :key="`snippet-${index}`"
        class="en-search-result-snippet"
      >
        <template v-if="query && snippetSegments(snippet).length">
          <span
            v-for="(segment, sIndex) in snippetSegments(snippet)"
            :key="`sn-${index}-${sIndex}`"
            :class="{ 'en-search-result-match': segment.match }"
          >{{ segment.text }}</span>
        </template>
        <template v-else>
          {{ snippet }}
        </template>
      </p>
    </div>

    <button
      class="en-search-result-open"
      type="button"
      title="Open note"
      @click.stop="$emit('open', result)"
    >
      <ArrowUpRight />
    </button>
  </article>
</template>

<script setup>
import { computed } from 'vue'
import { ArrowUpRight, FileText } from '@lucide/vue'

defineEmits(['open'])

const props = defineProps({
  result: {
    type: Object,
    required: true
  },
  selected: {
    type: Boolean,
    default: false
  },
  query: {
    type: String,
    default: ''
  }
})

const matchLabel = computed(() => {
  const matchType = String(props.result?.matchType || '').trim()
  if (matchType === 'hybrid') return 'Semantic + keyword'
  if (matchType === 'semantic') return 'Semantic'
  if (matchType === 'keyword') return 'Keyword'
  return 'Local match'
})

const displaySnippets = computed(() => {
  return (props.result?.snippets || [])
    .map((snippet) => String(snippet?.text || '').trim())
    .filter(Boolean)
    .slice(0, 1)
})

const escapedTokens = computed(() => {
  const query = String(props.query || '').trim()
  if (!query) return []
  const tokens = query.split(/\s+/).filter(Boolean)
  const seen = new Set()
  const escaped = []
  for (const token of tokens) {
    if (token.length < 2) continue
    if (seen.has(token.toLowerCase())) continue
    seen.add(token.toLowerCase())
    escaped.push(token.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
  }
  return escaped
})

const buildSegments = (text) => {
  if (!text) return [{ text: '', match: false }]
  if (!escapedTokens.value.length) return [{ text, match: false }]
  try {
    const re = new RegExp(`(${escapedTokens.value.join('|')})`, 'ig')
    const parts = String(text).split(re)
    return parts
      .filter((part) => part !== undefined && part !== '')
      .map((part) => ({ text: part, match: re.test(part) && part.length > 0 }))
  } catch (error) {
    return [{ text, match: false }]
  }
}

const titleSegments = computed(() => buildSegments(props.result?.title || ''))
const snippetSegments = (snippet) => buildSegments(snippet)
</script>

<style scoped>
.en-search-result {
  display: grid;
  grid-template-columns: 38px minmax(0, 1fr) 34px;
  align-items: start;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 16px;
  background: transparent;
  cursor: pointer;
  transition: background 120ms ease;
}

.en-search-result:hover {
  background: rgba(255, 255, 255, 0.28);
}

.en-search-result.selected {
  background: rgba(37, 99, 235, 0.2);
}

.en-shell.en-theme-dark .en-search-result.selected {
  background: rgba(94, 161, 255, 0.24);
}

.en-shell.en-theme-dark .en-search-result:hover {
  background: rgba(255, 255, 255, 0.14);
}

.en-search-result-icon {
  width: 38px;
  height: 38px;
  display: grid;
  place-items: center;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.34);
  color: var(--en-primary);
  flex: 0 0 auto;
}

.en-shell.en-theme-dark .en-search-result-icon {
  background: color-mix(in srgb, white 8%, transparent);
}

.en-search-result-icon svg {
  width: 18px;
  height: 18px;
}

.en-search-result-body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.en-search-result-headline {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  min-width: 0;
}

.en-search-result-title {
  margin: 0;
  min-width: 0;
  font-size: 15px;
  line-height: 1.3;
  font-weight: 700;
  color: var(--en-text);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.en-search-result-badge {
  flex: 0 0 auto;
  height: 22px;
  padding: 0 9px;
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  background: color-mix(in srgb, var(--en-muted) 20%, transparent);
  color: color-mix(in srgb, var(--en-text) 74%, var(--en-muted));
  font-size: 11.5px;
  font-weight: 700;
  letter-spacing: 0;
}

.en-search-result.selected .en-search-result-badge {
  background: rgba(37, 99, 235, 0.24);
  color: var(--en-primary);
}

.en-search-result-path {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--en-primary);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.en-search-result-snippet {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.4;
  color: color-mix(in srgb, var(--en-muted) 72%, var(--en-text));
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.en-search-result-match {
  background: color-mix(in srgb, var(--en-primary) 26%, transparent);
  color: color-mix(in srgb, var(--en-primary) 74%, var(--en-text));
  border-radius: 4px;
  padding: 0 2px;
  font-weight: 700;
}

.en-shell.en-theme-dark .en-search-result-match {
  background: color-mix(in srgb, var(--en-primary) 28%, transparent);
  color: color-mix(in srgb, var(--en-primary) 60%, white);
}

.en-search-result-open {
  align-self: center;
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 9px;
  background: transparent;
  color: var(--en-muted);
  cursor: pointer;
  opacity: 0;
  transition: opacity 140ms ease, background 140ms ease, color 140ms ease;
}

.en-search-result-open svg {
  width: 14px;
  height: 14px;
}

.en-search-result:hover .en-search-result-open,
.en-search-result.selected .en-search-result-open {
  opacity: 1;
}

.en-search-result-open:hover {
  background: color-mix(in srgb, var(--en-primary) 18%, transparent);
  color: var(--en-primary);
}

@media (max-width: 760px) {
  .en-search-result {
    grid-template-columns: 32px minmax(0, 1fr) 28px;
    gap: 10px;
    padding: 10px 12px;
  }

  .en-search-result-icon {
    width: 32px;
    height: 32px;
  }

  .en-search-result-open {
    width: 28px;
    height: 28px;
  }
}
</style>
