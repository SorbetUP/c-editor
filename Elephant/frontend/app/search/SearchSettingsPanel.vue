<template>
  <div class="en-search-settings">
    <section class="en-search-settings-row">
      <div>
        <h3>Atomic search</h3>
        <p>{{ statusLabel }}</p>
      </div>
      <div class="en-search-settings-actions">
        <button
          type="button"
          :disabled="store.busy"
          @click="refresh"
        >
          <RefreshCw class="en-icon" />
          Refresh status
        </button>
        <button
          class="en-search-settings-toggle"
          type="button"
          :class="{ active: store.status.status !== 'disabled' }"
          @click="toggleSearch"
        >
          <Power class="en-icon" />
          {{ store.status.status === 'disabled' ? 'Off' : 'On' }}
        </button>
      </div>
    </section>

    <section class="en-search-settings-row stacked en-search-options-panel">
      <div class="en-search-settings-head">
        <div>
          <h3>Search options</h3>
          <p>Configure fast local search. The heavy visual graph is disabled to keep large vaults responsive.</p>
        </div>
        <div class="en-search-options-summary">
          <span>{{ store.defaultMode }}</span>
          <span>{{ store.queryLimit }} results</span>
        </div>
      </div>

      <div class="en-search-options-grid">
        <div class="en-search-option-card wide">
          <div class="en-search-option-title">
            <SlidersHorizontal class="en-icon" />
            <div>
              <strong>Default mode</strong>
              <small>Used when the search modal opens.</small>
            </div>
          </div>
          <div
            class="en-search-mode-switch"
            role="group"
            aria-label="Default search mode"
          >
            <button
              v-for="mode in modeOptions"
              :key="mode.id"
              type="button"
              :class="{ active: store.defaultMode === mode.id }"
              @click="store.setDefaultMode(mode.id)"
            >
              <component
                :is="mode.icon"
                class="en-icon"
              />
              <span>{{ mode.label }}</span>
              <small>{{ mode.description }}</small>
            </button>
          </div>
        </div>

        <div class="en-search-option-card">
          <div class="en-search-option-title">
            <Search class="en-icon" />
            <div>
              <strong>Result limit</strong>
              <small>Maximum notes returned per query.</small>
            </div>
            <output>{{ store.queryLimit }}</output>
          </div>
          <input
            class="en-search-range"
            type="range"
            min="1"
            max="50"
            :style="queryLimitRangeStyle"
            :value="store.queryLimit"
            @input="store.setQueryLimit($event.target.value)"
          >
        </div>

        <div class="en-search-performance-card">
          <div class="en-search-option-title">
            <Zap class="en-icon" />
            <div>
              <strong>Performance mode</strong>
              <small>No graph, no map polling, no thousands of SVG nodes. Search stays usable on large vaults.</small>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup>
import { computed, onMounted } from 'vue'
import { GitBranch, Power, RefreshCw, Search, SlidersHorizontal, Zap } from '@lucide/vue'
import { useSearchStore } from '../stores/searchStore'

const store = useSearchStore()

const modeOptions = [
  { id: 'exact', label: 'Exact', description: 'Fast filename and text match', icon: Search },
  { id: 'smart', label: 'Smart', description: 'Balanced local ranking', icon: GitBranch },
  { id: 'semantic', label: 'Semantic', description: 'Meaning-first discovery', icon: Zap }
]

const statusLabel = computed(() => {
  const status = store.status.status || 'not_initialized'
  if (status === 'ready') return 'Atomic local search ready.'
  if (status === 'indexing') return `${store.status.indexedDocuments}/${store.status.totalDocuments} indexed.`
  if (status === 'disabled') return 'Atomic local search disabled.'
  if (status === 'error') return store.status.error || 'Atomic search error.'
  return store.status.message || 'Local search not initialized.'
})

const queryLimitRangeStyle = computed(() => ({
  '--range-progress': `${Math.round(((Number(store.queryLimit) - 1) / 49) * 100)}%`
}))

const refresh = async () => {
  await store.refreshStatus()
}

const toggleSearch = async () => {
  if (store.status.status === 'disabled') await store.enable()
  else await store.disable()
  await refresh()
}

onMounted(async () => {
  await store.ensureActiveVault()
  await refresh()
})
</script>

<style scoped>
.en-search-settings {
  display: grid;
  gap: 12px;
}

.en-search-settings-row {
  border: 1px solid var(--en-border);
  border-radius: 12px;
  padding: 16px;
  background: var(--en-bg);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.en-search-settings-row.stacked {
  display: block;
}

.en-search-settings-row h3 {
  margin: 0;
  color: var(--en-text);
  font-size: 17px;
}

.en-search-settings-row p {
  margin: 5px 0 0;
  color: var(--en-muted);
}

.en-search-settings-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.en-search-settings-toggle,
.en-search-settings-actions button {
  border: 1px solid var(--en-border);
  border-radius: 8px;
  min-height: 34px;
  padding: 0 10px;
  color: var(--en-text);
  background: var(--en-surface);
  cursor: pointer;
}

.en-search-settings-toggle.active {
  border-color: var(--en-border-strong);
  background: var(--en-soft-strong);
}

.en-search-settings-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}

.en-search-options-panel {
  background:
    radial-gradient(circle at top right, color-mix(in srgb, var(--en-primary) 14%, transparent), transparent 32%),
    var(--en-bg);
}

.en-search-options-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.en-search-options-summary span {
  border: 1px solid var(--en-border);
  border-radius: 999px;
  padding: 5px 9px;
  background: var(--en-surface);
  color: var(--en-muted);
  font-size: 12px;
  text-transform: capitalize;
}

.en-search-options-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-top: 14px;
}

.en-search-option-card,
.en-search-performance-card {
  border: 1px solid var(--en-border);
  border-radius: 14px;
  padding: 14px;
  background: color-mix(in srgb, var(--en-surface) 92%, var(--en-bg));
  box-shadow: 0 12px 34px color-mix(in srgb, black 6%, transparent);
}

.en-search-option-card.wide {
  grid-column: 1 / -1;
}

.en-search-performance-card {
  display: flex;
  align-items: center;
}

.en-search-option-title {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: start;
}

.en-search-option-title strong {
  display: block;
  color: var(--en-text);
  font-size: 13px;
}

.en-search-option-title small {
  display: block;
  margin-top: 3px;
  color: var(--en-muted);
  font-size: 12px;
  line-height: 1.35;
}

.en-search-option-title output {
  border-radius: 999px;
  min-width: 34px;
  padding: 4px 8px;
  background: var(--en-soft-strong);
  color: var(--en-text);
  font-size: 12px;
  text-align: center;
}

.en-search-mode-switch {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-top: 12px;
}

.en-search-mode-switch button {
  border: 1px solid var(--en-border);
  border-radius: 12px;
  padding: 12px;
  background: var(--en-surface);
  color: var(--en-muted);
  text-align: left;
  cursor: pointer;
  transition: border-color 120ms ease, background 120ms ease, transform 120ms ease;
}

.en-search-mode-switch button:hover {
  transform: translateY(-1px);
}

.en-search-mode-switch button.active {
  border-color: var(--en-border-strong);
  background: var(--en-soft-strong);
  color: var(--en-text);
}

.en-search-mode-switch button span {
  display: block;
  margin-top: 8px;
  color: var(--en-text);
  font-weight: 600;
}

.en-search-mode-switch button small {
  display: block;
  margin-top: 3px;
  line-height: 1.35;
}

.en-search-range {
  width: 100%;
  min-height: 34px;
  margin-top: 12px;
  border-radius: 999px;
  accent-color: var(--en-primary);
  background: linear-gradient(
    to right,
    var(--en-primary) var(--range-progress),
    var(--en-soft) var(--range-progress)
  );
}

.en-icon {
  width: 16px;
  height: 16px;
  vertical-align: middle;
}

@media (max-width: 780px) {
  .en-search-options-grid,
  .en-search-mode-switch {
    grid-template-columns: 1fr;
  }

  .en-search-settings-row {
    align-items: stretch;
  }
}
</style>
