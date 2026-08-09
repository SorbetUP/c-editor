<template>
  <el-dialog
    v-model="store.isOpen"
    class="en-search-dialog"
    modal-class="en-search-overlay"
    :show-close="false"
    :close-on-click-modal="true"
    :close-on-press-escape="false"
    width="720px"
    @closed="handleClosed"
  >
    <div
      class="en-search-shell"
      :class="{ 'has-panel': showPanel }"
      @click.stop
    >
      <div class="en-search-bar">
        <Search class="en-search-bar-icon" />
        <input
          ref="searchInput"
          v-model="query"
          class="en-search-bar-input"
          type="text"
          placeholder="Search notes, paths, tags, or ideas…"
          autocomplete="off"
          spellcheck="false"
          @keydown="handleKeyDown"
        >
        <div
          v-if="store.busy"
          class="en-search-bar-loading"
          aria-hidden="true"
        >
          <span /><span /><span />
        </div>
        <button
          v-if="store.query"
          class="en-search-bar-clear"
          type="button"
          title="Clear"
          @click="clearQuery"
        >
          <X />
        </button>
      </div>

      <transition name="en-panel">
        <div
          v-if="showPanel"
          class="en-search-panel"
        >
          <div
            v-if="showStatus"
            class="en-search-status"
          >
            <SearchStatusBadge :status="store.status" />
          </div>

          <div
            v-if="store.error"
            class="en-search-message en-search-message-error"
          >
            {{ store.error }}
          </div>

          <div
            v-else-if="store.busy && !hasSearchContent"
            class="en-search-message en-search-message-loading"
          >
            <ScanSearch class="en-search-state-icon" />
            <span>Searching locally…</span>
          </div>

          <div
            v-else-if="!hasSearchContent && hasQuery"
            class="en-search-empty"
          >
            <Search class="en-search-empty-icon" />
            <div class="en-search-empty-title">
              No matching notes found
            </div>
            <div class="en-search-empty-subtitle">
              Try another keyword or search in all vaults.
            </div>
          </div>

          <div
            v-else-if="hasSearchContent"
            class="en-search-results"
          >
            <section
              v-if="store.conceptResults.length"
              class="en-search-concepts"
            >
              <div class="en-search-section-title">
                Wikis & concepts
                <span v-if="store.conceptRoute?.ambiguous">ambiguous query</span>
              </div>
              <button
                v-for="concept in store.conceptResults"
                :key="concept.id"
                class="en-search-concept-card"
                type="button"
                @click="openConceptEvidence(concept)"
              >
                <div class="en-search-concept-main">
                  <div class="en-search-concept-title">
                    {{ concept.title }}
                  </div>
                  <div class="en-search-concept-meta">
                    {{ formatConceptMeta(concept) }}
                  </div>
                </div>
                <div class="en-search-concept-score">
                  {{ Math.round((concept.score || 0) * 100) }}%
                </div>
              </button>
            </section>

            <section
              v-if="store.results.length"
              class="en-search-note-results"
            >
              <div
                v-if="store.conceptResults.length"
                class="en-search-section-title"
              >
                Notes & passages
              </div>
              <SearchResultItem
                v-for="(result, index) in store.results"
                :key="result.uri || result.relativePath || index"
                :result="result"
                :selected="index === selectedIndex"
                :query="hasQuery ? store.query : ''"
                @open="openResult(result)"
                @mouseenter="selectedIndex = index"
              />
            </section>
          </div>
        </div>
      </transition>
    </div>
  </el-dialog>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { debounce } from 'underscore'
import { ScanSearch, Search, X } from '@lucide/vue'
import { useSearchStore } from '../stores/searchStore'
import SearchResultItem from './SearchResultItem.vue'
import SearchStatusBadge from './SearchStatusBadge.vue'

const store = useSearchStore()
const searchInput = ref(null)
const selectedIndex = ref(0)
let pollTimer = null

const query = computed({
  get: () => store.query,
  set: (value) => store.setQuery(value)
})

const hasQuery = computed(() => store.query.trim().length > 0)
const hasSearchContent = computed(() => store.results.length > 0 || store.conceptResults.length > 0)
const showStatus = computed(() => {
  return !['ready', 'not_initialized'].includes(store.status?.status || 'not_initialized')
})
const showPanel = computed(() => {
  if (store.error) return true
  if (store.busy) return true
  if (hasSearchContent.value) return true
  if (hasQuery.value) return true
  if (showStatus.value) return true
  return false
})

const focusInput = async () => {
  await nextTick()
  searchInput.value?.focus()
  searchInput.value?.select()
}

const clearQuery = () => {
  store.setQuery('')
  focusInput()
}

const openResult = (result) => {
  store.openResult(result)
}

const openConceptEvidence = (concept) => {
  const firstEvidence = concept?.evidenceChunks?.find((chunk) => chunk.relativePath || chunk.documentPath)
  if (!firstEvidence) return
  store.openResult({
    relativePath: firstEvidence.relativePath || firstEvidence.documentPath,
    title: concept.title
  })
}

const formatConceptMeta = (concept) => {
  const evidenceCount = concept?.evidenceChunks?.length || 0
  if (evidenceCount <= 0) return 'Concept candidate'
  const first = concept.evidenceChunks[0]
  const source = first.headingPath?.length
    ? first.headingPath.join(' › ')
    : first.relativePath || first.documentPath || 'source chunk'
  return `${evidenceCount} source chunk${evidenceCount === 1 ? '' : 's'} · ${source}`
}

const moveSelection = (delta) => {
  const count = store.results.length
  if (!count) return
  let next = selectedIndex.value + delta
  if (next < 0) next = count - 1
  if (next >= count) next = 0
  selectedIndex.value = next
}

const handleKeyDown = (event) => {
  switch (event.key) {
    case 'Escape': {
      event.preventDefault()
      event.stopPropagation()
      if (hasQuery.value) {
        clearQuery()
      } else {
        store.close()
      }
      break
    }
    case 'ArrowDown': {
      event.preventDefault()
      moveSelection(1)
      break
    }
    case 'ArrowUp': {
      event.preventDefault()
      moveSelection(-1)
      break
    }
    case 'Enter': {
      event.preventDefault()
      const result = store.results[selectedIndex.value]
      if (result) openResult(result)
      else if (store.conceptResults[0]) openConceptEvidence(store.conceptResults[0])
      break
    }
  }
}

const debouncedSearch = debounce(() => {
  store.search()
}, 220)

watch(
  () => store.query,
  (value, oldValue) => {
    if (value === oldValue) return
    if (!store.isOpen) return
    selectedIndex.value = 0
    debouncedSearch()
  }
)

watch(
  () => store.mode,
  (value, oldValue) => {
    if (value !== oldValue && store.isOpen) {
      selectedIndex.value = 0
      store.search()
    }
  }
)

watch(
  () => store.isOpen,
  async (value) => {
    if (value) {
      selectedIndex.value = 0
      await focusInput()
      store.refreshStatus()
      clearInterval(pollTimer)
      pollTimer = setInterval(() => {
        store.refreshStatus()
      }, 1500)
    } else {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }
)

const handleClosed = () => {
  store.results = []
  store.conceptResults = []
  store.conceptRoute = null
  store.error = ''
  store.busy = false
  selectedIndex.value = 0
}

onMounted(() => {
  store.refreshStatus()
})

onBeforeUnmount(() => {
  clearInterval(pollTimer)
})
</script>

<style scoped>
.en-search-shell {
  display: flex;
  flex-direction: column;
  width: 100%;
  border-radius: 28px;
  background:
    linear-gradient(
      135deg,
      rgba(255, 255, 255, 0.48),
      rgba(225, 240, 255, 0.34) 58%,
      rgba(255, 255, 255, 0.4)
    );
  box-shadow: 0 30px 90px rgba(15, 23, 42, 0.24);
  backdrop-filter: blur(42px) saturate(175%);
  -webkit-backdrop-filter: blur(42px) saturate(175%);
  overflow: hidden;
  transition: border-radius 160ms ease;
}

.en-shell.en-theme-dark .en-search-shell {
  background:
    linear-gradient(
      135deg,
      rgba(30, 41, 59, 0.56),
      rgba(15, 23, 42, 0.46) 58%,
      rgba(30, 41, 59, 0.5)
    );
  box-shadow: 0 24px 70px rgba(0, 0, 0, 0.5);
}

.en-search-bar {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 14px;
  min-height: 72px;
  padding: 0 18px 0 24px;
  background: transparent;
}

.en-search-bar-icon {
  width: 22px;
  height: 22px;
  color: color-mix(in srgb, var(--en-text) 76%, var(--en-muted));
  flex: 0 0 auto;
}

.en-search-bar-input {
  min-width: 0;
  height: 100%;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--en-text);
  font: inherit;
  font-size: 22px;
  font-weight: 720;
  letter-spacing: 0;
  outline: none;
}

.en-search-bar-input::placeholder {
  color: color-mix(in srgb, var(--en-muted) 88%, var(--en-text));
  font-weight: 650;
}

.en-search-bar-loading {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 0 6px;
}

.en-search-bar-loading span {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--en-primary);
  opacity: 0.45;
  animation: en-search-dot 1s infinite ease-in-out;
}

.en-search-bar-loading span:nth-child(2) { animation-delay: 0.15s; }
.en-search-bar-loading span:nth-child(3) { animation-delay: 0.3s; }

@keyframes en-search-dot {
  0%, 80%, 100% { opacity: 0.25; transform: scale(0.85); }
  40% { opacity: 1; transform: scale(1); }
}

.en-search-bar-clear {
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 999px;
  background: color-mix(in srgb, var(--en-muted) 14%, transparent);
  color: var(--en-muted);
  cursor: pointer;
  transition: background 140ms ease, color 140ms ease;
}

.en-search-bar-clear svg {
  width: 14px;
  height: 14px;
}

.en-search-bar-clear:hover {
  background: color-mix(in srgb, var(--en-muted) 24%, transparent);
  color: var(--en-text);
}

.en-search-panel {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 0 14px 14px;
  background: transparent;
  max-height: min(60vh, 560px);
  overflow: hidden;
}

.en-search-status {
  padding: 2px 4px;
}

.en-search-message {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 18px 20px;
  border-radius: 12px;
  color: var(--en-muted);
  font-size: 13px;
}

.en-search-message-error {
  background: color-mix(in srgb, #ef4444 10%, transparent);
  color: #c2410c;
}

.en-shell.en-theme-dark .en-search-message-error { color: #ffb4b4; }

.en-search-message-loading {
  padding: 22px 20px;
}

.en-search-state-icon {
  width: 18px;
  height: 18px;
  color: var(--en-primary);
}

.en-search-empty {
  display: grid;
  place-items: center;
  gap: 6px;
  padding: 28px 24px 32px;
  text-align: center;
  color: var(--en-muted);
}

.en-search-empty-icon {
  width: 28px;
  height: 28px;
  color: var(--en-primary);
  opacity: 0.8;
  margin-bottom: 4px;
}

.en-search-empty-title {
  font-size: 15px;
  font-weight: 650;
  color: var(--en-text);
}

.en-search-empty-subtitle {
  font-size: 12.5px;
  color: var(--en-muted);
}

.en-search-results {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 0 4px 6px;
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: thin;
  scrollbar-color: color-mix(in srgb, var(--en-muted) 28%, transparent) transparent;
}

.en-search-results::-webkit-scrollbar { width: 8px; }
.en-search-results::-webkit-scrollbar-track { background: transparent; }
.en-search-results::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--en-muted) 28%, transparent);
  border-radius: 999px;
}

.en-search-concepts,
.en-search-note-results {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.en-search-section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2px 8px 4px;
  color: var(--en-muted);
  font-size: 11px;
  font-weight: 760;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.en-search-section-title span {
  font-size: 10px;
  font-weight: 720;
  color: var(--en-primary);
  text-transform: none;
  letter-spacing: 0;
}

.en-search-concept-card {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  width: 100%;
  border: 1px solid color-mix(in srgb, var(--en-primary) 18%, transparent);
  border-radius: 14px;
  padding: 10px 12px;
  background: color-mix(in srgb, var(--en-primary) 7%, transparent);
  color: var(--en-text);
  cursor: pointer;
  text-align: left;
}

.en-search-concept-card:hover {
  background: color-mix(in srgb, var(--en-primary) 12%, transparent);
}

.en-search-concept-main {
  min-width: 0;
}

.en-search-concept-title {
  font-size: 14px;
  font-weight: 760;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.en-search-concept-meta {
  margin-top: 2px;
  font-size: 12px;
  color: var(--en-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.en-search-concept-score {
  font-size: 12px;
  font-weight: 760;
  color: var(--en-primary);
}

.en-panel-enter-active,
.en-panel-leave-active {
  transition: opacity 160ms ease;
}

.en-panel-enter-from,
.en-panel-leave-to {
  opacity: 0;
}

@media (max-width: 760px) {
  .en-search-bar {
    min-height: 56px;
    padding-left: 18px;
    gap: 10px;
  }

  .en-search-bar-input {
    font-size: 17px;
  }

  .en-search-bar-icon {
    width: 18px;
    height: 18px;
  }
}

:global(.en-search-dialog .el-dialog) {
  margin-top: 12vh !important;
  border-radius: 0;
  background: transparent !important;
  background-color: transparent !important;
  border: 0 !important;
  box-shadow: none !important;
  outline: 0 !important;
  overflow: visible;
}

:global(.el-dialog.en-search-dialog),
:global(.el-dialog.en-search-dialog.ag-dialog-table) {
  background: transparent !important;
  background-color: transparent !important;
  border: 0 !important;
  box-shadow: none !important;
  outline: 0 !important;
}

:global(.en-search-dialog .el-dialog__header),
:global(.en-search-dialog .el-dialog__body) {
  padding: 0 !important;
  margin: 0 !important;
  background: transparent !important;
  background-color: transparent !important;
}

:global(.en-search-dialog .el-dialog__headerbtn) {
  display: none;
}

:global(.en-search-overlay) {
  background: rgba(15, 23, 42, 0.12);
  backdrop-filter: blur(7px);
  -webkit-backdrop-filter: blur(7px);
}

:global(.en-shell.en-theme-dark .en-search-overlay) {
  background: rgba(3, 6, 12, 0.24);
}
</style>
