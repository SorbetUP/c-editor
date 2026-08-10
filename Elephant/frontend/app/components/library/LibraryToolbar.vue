<template>
  <div class="en-library-toolbar">
    <div class="en-library-toolbar-left">
      <CreateEntryMenu
        :mobile="true"
        :disabled="isBusy || !store.hasVault"
        @select="handleCreateSelection"
      >
        <template #trigger="{ toggle, open, disabled }">
          <button
            class="en-create-button en-create-button-primary"
            type="button"
            :disabled="disabled"
            :aria-expanded="open"
            aria-label="Create"
            aria-haspopup="menu"
            title="Create"
            :aria-busy="isBusy"
            @click="toggle"
          >
            <Plus
              class="en-create-icon"
              aria-hidden="true"
            />
          </button>
        </template>
      </CreateEntryMenu>
      <span
        v-if="actionError"
        class="en-library-action-error"
        role="alert"
      >
        {{ actionError }}
      </span>
    </div>

    <div class="en-library-actions">
      <button
        class="en-sort-cycle"
        type="button"
        :title="`Sort: ${sortOption.label}`"
        :aria-label="`Sort: ${sortOption.label}`"
        :data-sort="currentSort"
        @click="cycleSort"
      >
        <component
          :is="sortOption.icon"
          class="en-icon"
          aria-hidden="true"
        />
      </button>
      <button
        class="en-view-cycle"
        type="button"
        :title="viewModeLabel"
        :aria-label="viewModeLabel"
        :data-view-mode="store.viewMode"
        @click="cycleView"
      >
        <Grid3x3
          v-if="store.viewMode === 'list'"
          class="en-icon"
          aria-hidden="true"
        />
        <List
          v-else
          class="en-icon"
          aria-hidden="true"
        />
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { ArrowDownAZ, ArrowDownNarrowWide, ArrowDownZA, ArrowUpNarrowWide, Grid3x3, List, Plus } from '@lucide/vue'
import { useVaultStore } from '../../stores/vaultStore'
import CreateEntryMenu from './CreateEntryMenu.vue'
import { openNewDrawing } from './createEntryActions'

const store = useVaultStore()
const busyAction = ref('')
const actionError = ref('')
const isBusy = computed(() => !!busyAction.value)
const sortOptions = [
  { value: 'updated-newest', label: 'Updated newest', icon: ArrowDownNarrowWide },
  { value: 'updated-oldest', label: 'Updated oldest', icon: ArrowUpNarrowWide },
  { value: 'title-az', label: 'Title A-Z', icon: ArrowDownAZ },
  { value: 'title-za', label: 'Title Z-A', icon: ArrowDownZA }
]
const currentSort = computed(() => store.sort === 'title' ? 'title-az' : store.sort)
const sortOption = computed(() => sortOptions.find((option) => option.value === currentSort.value) || sortOptions[0])
const viewModeLabel = computed(() => store.viewMode === 'grid' ? 'Show notes as list' : 'Show notes as grid')

const cycleSort = () => {
  const index = sortOptions.findIndex((option) => option.value === currentSort.value)
  store.sort = sortOptions[(index + 1) % sortOptions.length].value
}

const cycleView = () => {
  store.viewMode = store.viewMode === 'grid' ? 'list' : 'grid'
}

const runCreateAction = async (action, callback) => {
  if (isBusy.value || !store.hasVault) return
  busyAction.value = action
  actionError.value = ''
  try {
    await callback()
  } catch (error) {
    actionError.value = error?.message || `Unable to create ${action}.`
    console.error(`[library] create ${action} failed`, error)
  } finally {
    busyAction.value = ''
  }
}

const createNote = () => runCreateAction('note', () => store.createNote())
const createFolder = () => runCreateAction('folder', () => store.createFolder())
const createDrawing = () => runCreateAction('drawing', openNewDrawing)

const handleCreateSelection = (key) => {
  if (key === 'note') return createNote()
  if (key === 'folder') return createFolder()
  if (key === 'drawing') return createDrawing()
}
</script>

<style scoped>
.en-library-toolbar {
  position: absolute;
  inset: 0 0 auto;
  z-index: 10;
  isolation: isolate;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 10px 12px;
  pointer-events: none;
}

.en-library-toolbar-left {
  min-width: 0;
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  pointer-events: auto;
}

.en-create-button {
  width: 56px;
  height: 56px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 1px solid var(--en-border);
  border-radius: 11px;
  color: var(--en-text);
  background: color-mix(in srgb, var(--en-surface) 68%, transparent);
  font: inherit;
  font-size: 15px;
  font-weight: 650;
  cursor: pointer;
}

.en-create-button:hover:not(:disabled) {
  border-color: var(--en-border-strong);
  background: var(--en-soft);
}

.en-create-button-primary {
  border-color: color-mix(in srgb, var(--en-primary) 64%, var(--en-border));
  color: #ffffff;
  background: var(--en-primary);
}

.en-create-button-primary:hover:not(:disabled) {
  border-color: var(--en-primary);
  background: color-mix(in srgb, var(--en-primary) 88%, #000000);
}

.en-create-button:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.en-create-button:focus-visible,
.en-sort-cycle:focus-visible,
.en-view-cycle:focus-visible {
  outline: 2px solid var(--en-primary);
  outline-offset: 2px;
}

.en-create-icon {
  width: 27px;
  height: 27px;
  flex-shrink: 0;
}

.en-library-action-error {
  min-width: 0;
  max-width: 320px;
  overflow: hidden;
  color: var(--en-danger, #dc2626);
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.en-library-actions {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  gap: 14px;
  margin-left: auto;
  pointer-events: auto;
}

.en-sort-cycle,
.en-view-cycle {
  position: relative;
  z-index: 1;
  width: 52px;
  height: 52px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--en-border);
  border-radius: 12px;
  color: var(--en-text);
  background: color-mix(in srgb, var(--en-surface) 52%, transparent);
  box-shadow: 0 8px 22px rgb(0 0 0 / 22%);
  backdrop-filter: blur(12px);
  font: inherit;
  font-size: 18px;
  cursor: pointer;
  pointer-events: auto;
  touch-action: manipulation;
}

.en-sort-cycle:hover,
.en-view-cycle:hover {
  color: var(--en-text);
  background: var(--en-soft);
}

.en-icon {
  width: 22px;
  height: 22px;
}

@media (max-width: 980px) {
  .en-library-toolbar {
    min-height: 0;
    align-items: center;
    flex-wrap: nowrap;
    padding-top: 8px;
    padding-bottom: 8px;
  }

  .en-library-toolbar-left,
  .en-library-actions {
    width: auto;
  }

  .en-library-actions {
    justify-content: flex-end;
  }
}
</style>
