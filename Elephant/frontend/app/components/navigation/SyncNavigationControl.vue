<template>
  <button
    type="button"
    class="en-nav-btn en-nav-sync"
    :class="syncClass"
    :disabled="syncDisabled"
    :aria-disabled="syncDisabled"
    :aria-label="syncTitle"
    :title="syncTitle"
    @mousedown.stop.prevent
    @pointerdown.stop
    @mouseup.stop
    @click.stop="syncWorkspace"
  >
    <RefreshCw
      class="en-nav-sync-icon"
      :class="{ spinning: nav.syncStatus === 'syncing' }"
    />
    <span
      v-if="nav.syncStatus === 'error' || nav.syncStatus === 'synced'"
      class="en-sync-dot"
      :class="nav.syncStatus === 'error' ? 'error' : 'success'"
      aria-hidden="true"
    />
  </button>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, watch } from 'vue'
import { RefreshCw } from '@lucide/vue'
import { IROH_SYNC_STATUS_EVENT } from '../../services/irohSyncClient'
import { useNavigationStore } from '../../stores/navigationStore'
import { useVaultStore } from '../../stores/vaultStore'

const nav = useNavigationStore()
const vaultStore = useVaultStore()
const activeVaultPath = computed(() => vaultStore.activeVault?.path || '')
const syncDisabled = computed(() => (
  nav.syncStatus === 'syncing' ||
  !activeVaultPath.value ||
  !nav.hasPairedSyncDevice
))

const syncClass = computed(() => ({
  'is-unpaired': nav.syncChecked && !nav.hasPairedSyncDevice,
  'is-syncing': nav.syncStatus === 'syncing',
  'is-error': nav.syncStatus === 'error',
  'is-synced': nav.syncStatus === 'synced'
}))

const syncTitle = computed(() => {
  if (!activeVaultPath.value) return 'Ouvrez une vault pour synchroniser'
  if (!nav.syncChecked) return 'Vérification de la synchronisation Iroh…'
  if (!nav.hasPairedSyncDevice) return 'Aucun appareil Iroh appairé'
  if (nav.syncStatus === 'syncing') return 'Synchronisation Iroh en cours…'
  if (nav.syncStatus === 'error') return `Erreur de synchronisation : ${nav.syncError}`
  if (nav.syncStatus === 'synced') return 'Synchronisé — cliquer pour relancer'
  return 'Synchroniser la vault active'
})

const syncWorkspace = async () => {
  if (syncDisabled.value) return
  try {
    await nav.syncWorkspace(activeVaultPath.value)
  } catch {
    // The navigation store keeps the durable error state and tooltip.
  }
}

const refreshSyncStatus = async () => {
  if (!activeVaultPath.value || nav.syncStatus === 'syncing') return
  try {
    await nav.refreshSyncStatus()
  } catch {
    // The icon displays the real backend error returned by the store.
  }
}

const handleWindowFocus = () => {
  refreshSyncStatus()
}

const handleSyncStatus = (event) => {
  if (event?.detail) nav.applySyncStatus(event.detail, { preserveRunning: true })
}

watch(activeVaultPath, (nextPath, previousPath) => {
  if (nextPath === previousPath) return
  nav.clearSyncStatus()
  if (nextPath) refreshSyncStatus()
})

onMounted(() => {
  refreshSyncStatus()
  window.addEventListener('focus', handleWindowFocus)
  window.addEventListener(IROH_SYNC_STATUS_EVENT, handleSyncStatus)
})

onBeforeUnmount(() => {
  window.removeEventListener('focus', handleWindowFocus)
  window.removeEventListener(IROH_SYNC_STATUS_EVENT, handleSyncStatus)
  nav.clearSyncStatus()
})
</script>

<style scoped>
.en-nav-btn {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  color: var(--en-muted);
  background: transparent;
  cursor: pointer;
  transition: background 0.12s ease, color 0.12s ease, opacity 0.12s ease;
  -webkit-app-region: no-drag !important;
  pointer-events: auto;
}

.en-nav-btn:hover:not(:disabled) {
  background: var(--en-soft);
  color: var(--en-text);
}

.en-nav-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.en-nav-sync {
  position: relative;
  margin-left: 0;
}

.en-nav-sync-icon {
  width: 18px;
  height: 18px;
}

.en-nav-sync-icon.spinning {
  animation: en-spin 0.8s linear infinite;
}

.en-nav-sync.is-error {
  color: var(--en-danger, #dc2626);
}

.en-nav-sync.is-synced {
  color: #16a34a;
}

.en-nav-sync.is-unpaired {
  color: var(--en-muted);
}

.en-sync-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  position: absolute;
  bottom: 2px;
  right: 2px;
  box-shadow: 0 0 0 1px var(--en-bg);
}

.en-sync-dot.error {
  background: var(--en-danger, #dc2626);
}

.en-sync-dot.success {
  background: #16a34a;
}

@keyframes en-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
