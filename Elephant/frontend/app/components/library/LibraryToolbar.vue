<template>
  <div class="en-library-toolbar">
    <div class="en-library-toolbar-left">
      <CreateEntryMenu
        :disabled="isBusy || !store.hasVault"
        @select="handleCreateSelection"
      >
        <template #trigger="{ toggle, open, disabled }">
          <button
            class="en-create-button en-create-button-primary"
            type="button"
            :disabled="disabled"
            :aria-expanded="open"
            aria-haspopup="menu"
            @click="toggle"
          >
            <Plus class="en-create-icon" />
            <span>{{ isBusy ? 'Creating…' : 'Create' }}</span>
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
      <select
        v-model="store.sort"
        class="en-select"
      >
        <option value="updated-newest">
          Sort: Updated newest
        </option>
        <option value="updated-oldest">
          Sort: Updated oldest
        </option>
        <option value="title">
          Sort: Title A-Z
        </option>
      </select>
      <div class="en-view-toggle">
        <button
          type="button"
          :class="{ active: store.viewMode === 'grid' }"
          title="Grid"
          aria-label="Grid view"
          @click="store.viewMode = 'grid'"
        >
          <Grid3x3 class="en-icon" />
        </button>
        <button
          type="button"
          :class="{ active: store.viewMode === 'list' }"
          title="List"
          aria-label="List view"
          @click="store.viewMode = 'list'"
        >
          <List class="en-icon" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { Grid3x3, List, Plus } from '@lucide/vue'
import { useVaultStore } from '../../stores/vaultStore'
import CreateEntryMenu from './CreateEntryMenu.vue'
import { openNewDrawing } from './createEntryActions'

const store = useVaultStore()
const busyAction = ref('')
const actionError = ref('')
const isBusy = computed(() => !!busyAction.value)

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
  min-height: 112px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 0 34px;
}

.en-library-toolbar-left {
  min-width: 0;
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
}

.en-create-button {
  min-height: 44px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 9px;
  padding: 0 16px;
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
.en-view-toggle button:focus-visible,
.en-select:focus-visible {
  outline: 2px solid var(--en-primary);
  outline-offset: 2px;
}

.en-create-icon {
  width: 18px;
  height: 18px;
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
  display: flex;
  align-items: center;
  gap: 14px;
  margin-left: auto;
}

.en-select,
.en-view-toggle {
  height: 52px;
  border: 1px solid var(--en-border);
  border-radius: 12px;
  color: var(--en-text);
  background: color-mix(in srgb, var(--en-surface) 52%, transparent);
  font: inherit;
  font-size: 18px;
}

.en-select {
  min-width: 278px;
  padding: 0 18px;
}

.en-view-toggle {
  display: inline-flex;
  overflow: hidden;
}

.en-view-toggle button {
  width: 56px;
  border: 0;
  color: var(--en-muted);
  background: transparent;
  cursor: pointer;
}

.en-view-toggle button.active {
  color: var(--en-text);
  background: var(--en-soft);
}

.en-icon {
  width: 22px;
  height: 22px;
}

@media (max-width: 980px) {
  .en-library-toolbar {
    min-height: 132px;
    align-items: flex-start;
    flex-wrap: wrap;
    padding-top: 20px;
    padding-bottom: 20px;
  }

  .en-library-toolbar-left,
  .en-library-actions {
    width: 100%;
  }

  .en-library-actions {
    justify-content: flex-end;
  }
}
</style>
