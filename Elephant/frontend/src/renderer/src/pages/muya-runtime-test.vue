<template>
  <main class="muya-runtime-test-page">
    <header class="muya-runtime-test-header">
      <div>
        <p class="eyebrow">Muya Runtime</p>
        <h1>Runtime integration test</h1>
        <p class="description">Live Muya-style rendering test page for the replacement runtime.</p>
      </div>
      <div class="runtime-controls">
        <label>
          Mode
          <select v-model="mode" @change="applyMode">
            <option value="disabled">disabled</option>
            <option value="shadow">shadow</option>
            <option value="active">active</option>
          </select>
        </label>
        <button type="button" @click="resetSample">Reset sample</button>
      </div>
    </header>

    <section class="muya-runtime-test-grid">
      <article class="panel">
        <h2>Source Markdown</h2>
        <textarea v-model="markdown" spellcheck="false" />
      </article>

      <article class="panel editor-panel">
        <h2>Live MuyaRuntimeEditor</h2>
        <MuyaRuntimeEditor
          v-model="markdown"
          :mode="mode"
          @ready="handleReady"
          @change="handleChange"
        />
      </article>

      <article class="panel preview-panel">
        <h2>Live HTML preview</h2>
        <div class="live-preview" v-html="liveHtml" />
      </article>

      <article class="panel">
        <h2>Runtime state</h2>
        <pre>{{ statePreview }}</pre>
      </article>
    </section>
  </main>
</template>

<script setup>
import { computed, ref } from 'vue'

import { MuyaRuntimeEditor } from '@/muya'

const sample = `# Muya runtime test

Paste Word/Notion/web HTML here.

- [x] Task item

| A | B |
| :- | -: |
| 1 | 2 |

![Alt](pic.png)

[^note]

[^note]: Footnote text

\`\`\`mermaid
graph TD;
\`\`\`

$$
x + 1
$$`

const markdown = ref(sample)
const mode = ref(window.__ELEPHANT_MUYA_RUNTIME__?.mode?.() || 'active')
const runtime = ref(null)
const lastChange = ref('')

const applyMode = () => {
  window.__ELEPHANT_MUYA_RUNTIME__?.setMode?.(mode.value)
}

const resetSample = () => {
  markdown.value = sample
}

const handleReady = (instance) => {
  runtime.value = instance
}

const handleChange = (value) => {
  lastChange.value = value
}

const liveHtml = computed(() => runtime.value?.html || '')

const statePreview = computed(() => JSON.stringify({
  mode: mode.value,
  globalMode: window.__ELEPHANT_MUYA_RUNTIME__?.mode?.() || null,
  ready: Boolean(runtime.value),
  markdownLength: markdown.value.length,
  lastChangeLength: lastChange.value.length,
  htmlLength: liveHtml.value.length,
  blockTypes: runtime.value?.state?.blocks?.map((block) => block.type) || []
}, null, 2))
</script>

<style scoped>
.muya-runtime-test-page {
  min-height: 100vh;
  padding: 24px;
  background: var(--editorColor, #f6f6f6);
  color: var(--textColor, #1f2328);
}

.muya-runtime-test-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 20px;
}

.eyebrow {
  margin: 0 0 4px;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  opacity: 0.65;
}

h1,
h2,
p {
  margin: 0;
}

.description {
  margin-top: 6px;
  opacity: 0.72;
}

.runtime-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.runtime-controls label {
  display: grid;
  gap: 4px;
  font-size: 12px;
}

.runtime-controls select,
.runtime-controls button {
  border: 1px solid rgba(0, 0, 0, 0.14);
  border-radius: 8px;
  padding: 8px 10px;
  background: rgba(255, 255, 255, 0.8);
}

.muya-runtime-test-grid {
  display: grid;
  grid-template-columns: minmax(240px, 0.8fr) minmax(320px, 1.1fr) minmax(320px, 1.1fr) minmax(240px, 0.8fr);
  gap: 16px;
}

.panel {
  min-height: 68vh;
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 18px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.84);
  box-shadow: 0 16px 44px rgba(0, 0, 0, 0.08);
  overflow: auto;
}

.panel h2 {
  margin-bottom: 12px;
  font-size: 15px;
}

textarea {
  width: 100%;
  min-height: calc(68vh - 52px);
  border: 0;
  outline: none;
  resize: none;
  background: transparent;
  font: 14px/1.55 ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.editor-panel :deep(.muya-runtime-shell) {
  min-height: calc(68vh - 52px);
}

.live-preview {
  line-height: 1.6;
}

.live-preview :deep(pre),
.live-preview :deep(code) {
  white-space: pre-wrap;
}

pre {
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 12px;
}
</style>
