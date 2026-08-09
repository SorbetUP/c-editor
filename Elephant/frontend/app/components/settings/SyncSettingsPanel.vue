<template>
  <div class="en-sync-panel">
    <div class="en-sync-toolbar">
      <nav class="en-sync-tabs" aria-label="Synchronization settings pages">
        <button type="button" :class="{ active: activeSyncPage === 'overview' }" @click="activeSyncPage = 'overview'">
          <Gauge aria-hidden="true" /> Overview
        </button>
        <button type="button" :class="{ active: activeSyncPage === 'devices' }" @click="activeSyncPage = 'devices'">
          <Laptop aria-hidden="true" /> Devices
          <span v-if="pairedDevices.length" class="en-sync-tab-count">{{ pairedDevices.length }}</span>
        </button>
        <button type="button" :class="{ active: activeSyncPage === 'conflicts' }" @click="activeSyncPage = 'conflicts'">
          <Archive aria-hidden="true" /> Conflicts
          <span v-if="archiveEntries.length || reportedConflicts.length" class="en-sync-tab-count warning">{{ archiveEntries.length + reportedConflicts.length }}</span>
        </button>
      </nav>

      <div class="en-sync-toolbar-actions">
        <span class="en-sync-status" :class="{ active: pairedDevices.length > 0, error: hasError }">
          <span class="en-sync-status-dot" />{{ connectionLabel }}
        </span>
        <button class="secondary compact icon-only" type="button" title="Refresh status" :disabled="loading || !hasVault" @click="refreshAll()"><RotateCw aria-hidden="true" /></button>
        <button class="primary compact" type="button" :disabled="loading || !hasVault || !pairedDevices.length" @click="syncNow"><RefreshCw aria-hidden="true" :class="{ spinning: syncing }" />{{ syncing ? 'Syncing…' : 'Sync now' }}</button>
      </div>
    </div>

    <p v-if="statusMessage" class="en-sync-message" :class="{ error: hasError }">{{ statusMessage }}</p>

    <template v-if="activeSyncPage === 'overview'">
      <section class="en-sync-card">
        <div class="en-sync-summary">
          <article>
            <span class="en-sync-summary-icon"><FolderSync aria-hidden="true" /></span>
            <div><small>Active vault</small><strong>{{ activeVaultName }}</strong><p>{{ activeVaultPath || 'Open a vault to configure sync.' }}</p></div>
          </article>
          <article>
            <span class="en-sync-summary-icon"><Fingerprint aria-hidden="true" /></span>
            <div><small>Device identity</small><strong>{{ shortDeviceId }}</strong><p>Iroh EndpointId</p></div>
          </article>
          <article>
            <span class="en-sync-summary-icon"><Clock3 aria-hidden="true" /></span>
            <div><small>Last synchronization</small><strong>{{ lastRunLabel }}</strong><p>{{ transferLabel }}</p></div>
          </article>
        </div>
      </section>

      <section class="en-sync-card">
        <header class="en-sync-card-header">
          <h4>Devices</h4>
          <button class="secondary compact" type="button" @click="activeSyncPage = 'devices'"><Link2 aria-hidden="true" /> Pair a device</button>
        </header>
        <div class="en-sync-list flush">
          <article v-if="!pairedDevices.length" class="en-sync-empty">
            <Laptop aria-hidden="true" />
            <div><strong>No paired device</strong><p>Pair another ElephantNote installation to begin synchronizing.</p></div>
            <button type="button" class="primary compact" @click="activeSyncPage = 'devices'">Start pairing</button>
          </article>
          <article v-for="device in pairedDevices" :key="device.endpointId" class="en-sync-device-row">
            <span class="en-device-avatar"><Laptop aria-hidden="true" /></span>
            <div><strong>{{ device.name }}</strong><p>{{ shortId(device.endpointId) }} · last seen {{ formatEpochSeconds(device.lastSeenAt) }}</p></div>
            <span class="en-verified-badge"><ShieldCheck aria-hidden="true" /> Verified</span>
          </article>
        </div>
      </section>

      <section class="en-sync-card">
        <header class="en-sync-card-header"><h4>Conflict protection</h4><button class="secondary compact" type="button" @click="activeSyncPage = 'conflicts'"><Archive aria-hidden="true" /> View copies</button></header>
        <div class="en-sync-setting-row">
          <div><strong>Temporary copies</strong><p>Older conflicting versions stay in <code>.conflit/</code> on this device.</p></div>
          <div class="en-retention-control">
            <label>
              <input v-model.number="retentionDays" type="number" :min="conflictSettings.minimumRetentionDays || 1" :max="conflictSettings.maximumRetentionDays || 365" step="1" aria-label="Conflict retention days">
              <span>days</span>
            </label>
            <button class="primary compact" type="button" :disabled="loading || !hasVault || !validRetention" @click="saveRetention">Save</button>
          </div>
        </div>
        <p v-if="conflictMessage" class="en-sync-inline-message">{{ conflictMessage }}</p>
      </section>
    </template>

    <template v-else-if="activeSyncPage === 'devices'">
      <section class="en-sync-card">
        <header class="en-sync-card-header"><h4>Pair a device</h4><span>Invitations expire after ten minutes.</span></header>
        <div class="en-pair-flow">
          <article class="en-pair-step">
            <div class="en-pair-step-heading"><span>1</span><div><strong>Create invitation</strong><p>Keep ElephantNote open until the second device accepts it.</p></div></div>
            <button class="primary" type="button" :disabled="loading || !hasVault" @click="createInvite"><Link2 aria-hidden="true" /> Create invitation</button>
            <div v-if="inviteCode" class="en-invite-box">
              <textarea :value="inviteCode" readonly rows="5" aria-label="Iroh pairing invitation"></textarea>
              <button class="secondary" type="button" @click="copyInvite"><Copy aria-hidden="true" />{{ copied ? 'Copied' : 'Copy invitation' }}</button>
            </div>
          </article>

          <span class="en-pair-connector"><ArrowRight aria-hidden="true" /></span>

          <article class="en-pair-step">
            <div class="en-pair-step-heading"><span>2</span><div><strong>Accept invitation</strong><p>Paste the complete invitation generated by the first device.</p></div></div>
            <textarea v-model.trim="incomingInvite" rows="5" placeholder="Paste the ElephantNote Iroh invitation here"></textarea>
            <button class="primary" type="button" :disabled="loading || !hasVault || !incomingInvite" @click="acceptInvite"><ShieldCheck aria-hidden="true" /> Pair this device</button>
          </article>
        </div>
      </section>

      <section class="en-sync-card">
        <header class="en-sync-card-header"><h4>Paired devices</h4><span>{{ pairedDevices.length }}</span></header>
        <div class="en-sync-list flush">
          <article v-if="!pairedDevices.length" class="en-sync-empty compact-empty"><Laptop aria-hidden="true" /><div><strong>No paired device</strong><p>Create or accept an invitation above.</p></div></article>
          <article v-for="device in pairedDevices" :key="device.endpointId" class="en-sync-device-row">
            <span class="en-device-avatar"><Laptop aria-hidden="true" /></span>
            <div><strong>{{ device.name }}</strong><p>{{ shortId(device.endpointId) }} · last seen {{ formatEpochSeconds(device.lastSeenAt) }}</p></div>
            <span class="en-verified-badge"><ShieldCheck aria-hidden="true" /> Verified</span>
          </article>
        </div>
      </section>
    </template>

    <template v-else>
      <section class="en-sync-card">
        <header class="en-sync-card-header"><h4>Retention</h4><span>{{ archiveEntries.length }} temporary cop{{ archiveEntries.length === 1 ? 'y' : 'ies' }}</span></header>
        <div class="en-sync-setting-row">
          <div><strong>Keep conflict copies for</strong><p>Cleanup is local and never removes a note or a copy from another device.</p></div>
          <div class="en-retention-control">
            <label><input v-model.number="retentionDays" type="number" :min="conflictSettings.minimumRetentionDays || 1" :max="conflictSettings.maximumRetentionDays || 365" step="1"><span>days</span></label>
            <button class="primary compact" type="button" :disabled="loading || !hasVault || !validRetention" @click="saveRetention">Save retention</button>
          </div>
        </div>
        <div class="en-security-note"><ShieldCheck aria-hidden="true" /><p>Restoring a copy never overwrites the current note. ElephantNote creates a separate restored file when the original path already exists.</p></div>
        <p v-if="conflictMessage" class="en-sync-inline-message">{{ conflictMessage }}</p>
      </section>

      <section class="en-sync-card">
        <header class="en-sync-card-header"><h4>Archived versions</h4><span>{{ archiveEntries.length }}</span></header>
        <div class="en-sync-list flush">
          <article v-if="!archiveEntries.length" class="en-sync-empty compact-empty"><Archive aria-hidden="true" /><div><strong>No temporary conflict copy</strong><p>Archived versions appear here and expire after {{ retentionDays }} day(s).</p></div></article>
          <article v-for="entry in archiveEntries" :key="entry.path" class="en-conflict-row">
            <span class="en-conflict-icon"><FileClock aria-hidden="true" /></span>
            <div><strong>{{ entry.path }}</strong><p>{{ formatBytes(entry.size) }} · archived {{ formatTimestamp(entry.modifiedMs) }}</p></div>
            <div class="en-conflict-actions">
              <button class="secondary compact" type="button" :disabled="loading || conflictActionPath === entry.path" @click="restoreConflict(entry)"><Undo2 aria-hidden="true" />{{ conflictActionPath === entry.path ? 'Working…' : 'Restore' }}</button>
              <button class="danger compact" type="button" :disabled="loading || conflictActionPath === entry.path" @click="deleteConflict(entry)"><Trash2 aria-hidden="true" />Delete</button>
            </div>
          </article>
        </div>
      </section>

      <section v-if="reportedConflicts.length" class="en-sync-card warning-card">
        <header class="en-sync-card-header"><h4>Last synchronization</h4><AlertTriangle aria-hidden="true" /></header>
        <div class="en-sync-list flush">
          <article v-for="conflict in reportedConflicts" :key="conflict.path" class="en-conflict-row">
            <span class="en-conflict-icon warning"><AlertTriangle aria-hidden="true" /></span>
            <div><strong>{{ conflict.path }}</strong><p>Both devices modified this path.</p></div>
            <span class="en-preserved-badge">Preserved</span>
          </article>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  AlertTriangle,
  Archive,
  ArrowRight,
  Clock3,
  Copy,
  FileClock,
  Fingerprint,
  FolderSync,
  Gauge,
  Laptop,
  Link2,
  RefreshCw,
  RotateCw,
  ShieldCheck,
  Trash2,
  Undo2
} from '@lucide/vue'
import { irohSyncClient } from '../../services/irohSyncClient'

const props = defineProps({
  vaults: { type: Array, default: () => [] },
  activeVaultPath: { type: String, default: '' },
  initialPage: { type: String, default: 'overview' }
})

const validPages = new Set(['overview', 'devices', 'conflicts'])
const activeSyncPage = ref(validPages.has(props.initialPage) ? props.initialPage : 'overview')
const status = ref({})
const conflictSettings = ref({
  retentionDays: 3,
  minimumRetentionDays: 1,
  maximumRetentionDays: 365,
  entries: []
})
const retentionDays = ref(3)
const inviteCode = ref('')
const incomingInvite = ref('')
const statusMessage = ref('')
const conflictMessage = ref('')
const loading = ref(false)
const syncing = ref(false)
const copied = ref(false)
const conflictActionPath = ref('')
let refreshTimer = null

const hasVault = computed(() => Boolean(props.activeVaultPath))
const activeVaultName = computed(() => {
  const active = props.vaults.find((vault) => vault?.path === props.activeVaultPath)
  return active?.name || status.value?.activeVault?.name || 'No active vault'
})
const pairedDevices = computed(() => Array.isArray(status.value?.peers) ? status.value.peers : [])
const archiveEntries = computed(() => Array.isArray(conflictSettings.value?.entries) ? conflictSettings.value.entries : [])
const reportedConflicts = computed(() => Array.isArray(status.value?.conflicts) ? status.value.conflicts : [])
const connectionLabel = computed(() => pairedDevices.value.length
  ? `${pairedDevices.value.length} paired device${pairedDevices.value.length === 1 ? '' : 's'}`
  : 'Not paired')
const shortDeviceId = computed(() => shortId(status.value?.deviceId || 'Unavailable'))
const lastRunLabel = computed(() => formatEpochSeconds(status.value?.lastRunAt, 'Never'))
const transferLabel = computed(() => {
  const files = Number(status.value?.transferredFiles || 0)
  const bytes = Number(status.value?.transferredBytes || 0)
  return files ? `${files} file${files === 1 ? '' : 's'} · ${formatBytes(bytes)}` : 'No transfer recorded'
})
const validRetention = computed(() => {
  const value = Number(retentionDays.value)
  const minimum = Number(conflictSettings.value?.minimumRetentionDays || 1)
  const maximum = Number(conflictSettings.value?.maximumRetentionDays || 365)
  return Number.isInteger(value) && value >= minimum && value <= maximum
})
const hasError = computed(() => Boolean(status.value?.lastError))

const shortId = (value) => {
  const text = String(value || '')
  if (text.length <= 20) return text
  return `${text.slice(0, 10)}…${text.slice(-8)}`
}

const formatEpochSeconds = (value, fallback = 'Unknown') => {
  const seconds = Number(value)
  if (!Number.isFinite(seconds) || seconds <= 0) return fallback
  return new Date(seconds * 1000).toLocaleString()
}

const formatTimestamp = (value) => {
  const milliseconds = Number(value)
  if (!Number.isFinite(milliseconds) || milliseconds <= 0) return 'unknown time'
  return new Date(milliseconds).toLocaleString()
}

const formatBytes = (value) => {
  const bytes = Number(value || 0)
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  const units = ['B', 'KiB', 'MiB', 'GiB']
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const amount = bytes / (1024 ** index)
  return `${amount >= 10 || index === 0 ? amount.toFixed(0) : amount.toFixed(1)} ${units[index]}`
}

const errorMessage = (error, fallback) => error instanceof Error ? error.message : fallback

const loadStatus = async () => {
  status.value = await irohSyncClient.status()
  statusMessage.value = status.value?.lastError || ''
}

const loadConflictSettings = async () => {
  const result = await irohSyncClient.conflictSettings()
  conflictSettings.value = result || conflictSettings.value
  retentionDays.value = Number(result?.retentionDays || 3)
  if (Number(result?.deletedFiles || 0) > 0) {
    conflictMessage.value = `${result.deletedFiles} expired conflict file(s) removed.`
  }
}

const refreshAll = async (silent = false) => {
  if (!hasVault.value || (loading.value && !silent)) return
  if (!silent) loading.value = true
  try {
    await Promise.all([loadStatus(), loadConflictSettings()])
  } catch (error) {
    statusMessage.value = errorMessage(error, 'Unable to load Iroh synchronization status.')
  } finally {
    if (!silent) loading.value = false
  }
}

const createInvite = async () => {
  if (!hasVault.value || loading.value) return
  loading.value = true
  statusMessage.value = 'Creating a secure one-time invitation…'
  try {
    const result = await irohSyncClient.createInvite({ deviceName: activeVaultName.value })
    inviteCode.value = String(result?.manualCode || result?.qrPayload || '')
    copied.value = false
    statusMessage.value = inviteCode.value
      ? 'Invitation created. Paste it on the second device within ten minutes.'
      : 'Invitation created.'
  } catch (error) {
    statusMessage.value = errorMessage(error, 'Unable to create an Iroh invitation.')
  } finally {
    loading.value = false
  }
}

const copyInvite = async () => {
  if (!inviteCode.value) return
  try {
    await navigator.clipboard.writeText(inviteCode.value)
    copied.value = true
  } catch {
    copied.value = false
    statusMessage.value = 'Clipboard access failed. Select and copy the invitation manually.'
  }
}

const acceptInvite = async () => {
  if (!incomingInvite.value || !hasVault.value || loading.value) return
  loading.value = true
  statusMessage.value = 'Pairing with the remote Iroh device…'
  try {
    const result = await irohSyncClient.acceptInvite(incomingInvite.value)
    status.value = result?.status || await irohSyncClient.status()
    incomingInvite.value = ''
    statusMessage.value = 'Device paired. You can synchronize now.'
    await loadConflictSettings()
  } catch (error) {
    statusMessage.value = errorMessage(error, 'Unable to accept this invitation.')
  } finally {
    loading.value = false
  }
}

const syncNow = async () => {
  if (!pairedDevices.value.length || !hasVault.value || loading.value) return
  loading.value = true
  syncing.value = true
  statusMessage.value = 'Comparing manifests and transferring changed files…'
  try {
    status.value = await irohSyncClient.run()
    statusMessage.value = status.value?.lastError || 'Synchronization finished.'
    await loadConflictSettings()
  } catch (error) {
    statusMessage.value = errorMessage(error, 'Synchronization failed.')
  } finally {
    syncing.value = false
    loading.value = false
  }
}

const saveRetention = async () => {
  if (!validRetention.value || !hasVault.value || loading.value) return
  loading.value = true
  conflictMessage.value = 'Saving local conflict retention…'
  try {
    const result = await irohSyncClient.setConflictRetentionDays(retentionDays.value)
    conflictSettings.value = result
    retentionDays.value = Number(result?.retentionDays || retentionDays.value)
    conflictMessage.value = Number(result?.deletedFiles || 0) > 0
      ? `Saved. ${result.deletedFiles} expired conflict file(s) removed.`
      : 'Retention saved for this device.'
  } catch (error) {
    conflictMessage.value = errorMessage(error, 'Unable to save conflict retention.')
  } finally {
    loading.value = false
  }
}

const restoreConflict = async (entry) => {
  if (!entry?.path || loading.value) return
  loading.value = true
  conflictActionPath.value = entry.path
  conflictMessage.value = `Restoring ${entry.path}…`
  try {
    const result = await irohSyncClient.restoreConflict(entry.path)
    conflictSettings.value = result
    conflictMessage.value = `Restored as ${result?.restoredPath || 'a separate vault file'}.`
  } catch (error) {
    conflictMessage.value = errorMessage(error, 'Unable to restore this conflict copy.')
  } finally {
    conflictActionPath.value = ''
    loading.value = false
  }
}

const deleteConflict = async (entry) => {
  if (!entry?.path || loading.value) return
  if (!window.confirm(`Delete the temporary conflict copy "${entry.path}"?`)) return
  loading.value = true
  conflictActionPath.value = entry.path
  conflictMessage.value = `Deleting ${entry.path}…`
  try {
    conflictSettings.value = await irohSyncClient.deleteConflict(entry.path)
    conflictMessage.value = 'Temporary conflict copy deleted from this device.'
  } catch (error) {
    conflictMessage.value = errorMessage(error, 'Unable to delete this conflict copy.')
  } finally {
    conflictActionPath.value = ''
    loading.value = false
  }
}

watch(() => props.initialPage, (page) => {
  if (validPages.has(page)) activeSyncPage.value = page
})

watch(() => props.activeVaultPath, () => {
  inviteCode.value = ''
  incomingInvite.value = ''
  refreshAll()
})

onMounted(() => {
  refreshAll()
  refreshTimer = window.setInterval(() => {
    if (!loading.value && hasVault.value) refreshAll(true)
  }, 5000)
})

onBeforeUnmount(() => {
  if (refreshTimer) window.clearInterval(refreshTimer)
})
</script>

<style scoped>
.en-sync-panel { display: grid; gap: 14px; color: var(--en-text, #101828); }
.en-sync-toolbar { position: sticky; top: -28px; z-index: 3; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 5px; border: 1px solid var(--en-border, #c5cfdd); border-radius: 12px; background: color-mix(in srgb, var(--en-surface, #fff) 94%, transparent); backdrop-filter: blur(14px); }
.en-sync-tabs { display: flex; align-items: center; gap: 4px; min-width: 0; }
.en-sync-tabs button { min-height: 32px; display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 0 10px; border: 1px solid transparent; border-radius: 8px; background: transparent; color: var(--en-muted, #667085); cursor: pointer; }
.en-sync-tabs button.active { border-color: var(--en-border, #c5cfdd); background: var(--en-surface, #fff); color: var(--en-text, #101828); box-shadow: 0 1px 4px rgba(2, 6, 23, 0.08); }
.en-sync-tabs svg, button svg { width: 14px; height: 14px; }
.en-sync-tab-count { min-width: 17px; height: 17px; display: inline-grid; place-items: center; padding: 0 4px; border-radius: 99px; background: color-mix(in srgb, var(--en-primary, #2563eb) 12%, transparent); color: var(--en-primary, #2563eb); font-size: 9px; }
.en-sync-tab-count.warning { background: rgba(245, 158, 11, 0.13); color: #b45309; }
.en-sync-toolbar-actions, .en-conflict-actions, .en-retention-control { display: flex; align-items: center; gap: 7px; }
.en-sync-status { display: inline-flex; align-items: center; gap: 6px; min-height: 27px; padding: 0 8px; border: 1px solid var(--en-border, #c5cfdd); border-radius: 99px; color: var(--en-muted, #667085); font-size: 9.5px; font-weight: 650; }
.en-sync-status-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--en-muted, #667085); }
.en-sync-status.active { border-color: color-mix(in srgb, #16a34a 30%, var(--en-border, #c5cfdd)); color: #15803d; }
.en-sync-status.active .en-sync-status-dot { background: #22c55e; box-shadow: 0 0 0 3px rgba(34, 197, 94, 0.13); }
.en-sync-status.error { color: #b42318; }
.en-sync-status.error .en-sync-status-dot { background: #ef4444; }
.en-sync-card { overflow: hidden; border: 1px solid var(--en-border, #c5cfdd); border-radius: 14px; background: var(--en-surface, #fff); box-shadow: 0 1px 2px rgba(2, 6, 23, 0.03); }
.en-sync-card-header { min-height: 52px; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 10px 16px; border-bottom: 1px solid var(--en-border, #c5cfdd); background: color-mix(in srgb, var(--en-surface, #fff) 94%, var(--en-soft, #e9eff7)); }
h4, p { margin: 0; }
h4 { font-size: 13px; }
.en-sync-card-header > span, .en-sync-summary p, .en-sync-device-row p, .en-conflict-row p, .en-pair-step p, .en-sync-empty p, .en-sync-setting-row p { color: var(--en-muted, #667085); font-size: 10.5px; line-height: 1.42; }
button { min-height: 34px; display: inline-flex; align-items: center; justify-content: center; gap: 7px; padding: 0 11px; border: 1px solid var(--en-border, #c5cfdd); border-radius: 9px; background: var(--en-surface, #fff); color: var(--en-text, #101828); cursor: pointer; transition: 140ms ease; }
button:hover:not(:disabled) { border-color: var(--en-primary, #2563eb); }
button:disabled { opacity: 0.48; cursor: not-allowed; }
button.primary { border-color: var(--en-primary, #2563eb); background: var(--en-primary, #2563eb); color: #fff; }
button.secondary { background: var(--en-bg, #f7f9fc); }
button.danger { border-color: color-mix(in srgb, var(--en-danger, #dc2626) 35%, var(--en-border, #c5cfdd)); color: var(--en-danger, #dc2626); }
button.compact { min-height: 29px; padding: 0 8px; font-size: 10.5px; }
button.icon-only { width: 29px; padding: 0; }
.spinning { animation: spin 0.9s linear infinite; }
.en-sync-message, .en-sync-inline-message { padding: 9px 12px; border: 1px solid var(--en-border, #c5cfdd); border-radius: 9px; background: color-mix(in srgb, var(--en-soft, #e9eff7) 42%, transparent); color: var(--en-muted, #667085); font-size: 10.5px; }
.en-sync-message.error { color: #b42318; }
.en-sync-inline-message { margin: 0 16px 14px; }
.en-sync-summary { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); }
.en-sync-summary article { min-width: 0; display: flex; align-items: flex-start; gap: 10px; padding: 15px 16px; }
.en-sync-summary article + article { border-left: 1px solid var(--en-border, #c5cfdd); }
.en-sync-summary-icon, .en-device-avatar, .en-conflict-icon { width: 30px; height: 30px; display: grid; place-items: center; flex: 0 0 auto; border-radius: 8px; background: var(--en-soft, #e9eff7); color: var(--en-primary, #2563eb); }
.en-sync-summary-icon svg, .en-device-avatar svg, .en-conflict-icon svg { width: 15px; height: 15px; }
.en-sync-summary div, .en-sync-device-row div, .en-conflict-row div { min-width: 0; }
.en-sync-summary small { display: block; margin-bottom: 3px; color: var(--en-muted, #667085); font-size: 9.5px; }
.en-sync-summary strong, .en-sync-summary p { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.en-sync-list { display: grid; }
.en-sync-device-row, .en-conflict-row, .en-sync-empty { min-height: 62px; display: grid; grid-template-columns: 32px minmax(0, 1fr) auto; align-items: center; gap: 11px; padding: 10px 16px; }
.en-sync-device-row + .en-sync-device-row, .en-conflict-row + .en-conflict-row { border-top: 1px solid var(--en-border, #c5cfdd); }
.en-verified-badge, .en-preserved-badge { display: inline-flex; align-items: center; gap: 5px; min-height: 25px; padding: 0 7px; border: 1px solid color-mix(in srgb, #16a34a 28%, var(--en-border, #c5cfdd)); border-radius: 99px; color: #15803d; font-size: 9.5px; }
.en-verified-badge svg { width: 12px; height: 12px; }
.en-preserved-badge { border-color: color-mix(in srgb, #d97706 28%, var(--en-border, #c5cfdd)); color: #b45309; }
.en-sync-empty { color: var(--en-muted, #667085); }
.en-sync-empty > svg { width: 19px; height: 19px; }
.compact-empty { grid-template-columns: 30px minmax(0, 1fr); }
.en-sync-setting-row { min-height: 70px; display: flex; align-items: center; justify-content: space-between; gap: 20px; padding: 13px 16px; }
.en-retention-control label { display: flex; align-items: center; gap: 6px; }
.en-retention-control input { width: 66px; height: 33px; padding: 0 8px; border: 1px solid var(--en-border, #c5cfdd); border-radius: 8px; background: var(--en-bg, #f7f9fc); color: var(--en-text, #101828); }
.en-retention-control span { color: var(--en-muted, #667085); font-size: 10.5px; }
.en-pair-flow { display: grid; grid-template-columns: minmax(0, 1fr) 28px minmax(0, 1fr); align-items: stretch; gap: 9px; padding: 16px; }
.en-pair-step { display: flex; flex-direction: column; gap: 11px; padding: 13px; border: 1px solid var(--en-border, #c5cfdd); border-radius: 10px; background: var(--en-bg, #f7f9fc); }
.en-pair-step-heading { display: flex; align-items: flex-start; gap: 9px; }
.en-pair-step-heading > span { width: 23px; height: 23px; display: grid; place-items: center; flex: 0 0 auto; border-radius: 50%; background: var(--en-primary, #2563eb); color: #fff; font-size: 10px; font-weight: 700; }
.en-pair-step textarea, .en-invite-box textarea { width: 100%; min-height: 98px; padding: 9px; resize: vertical; box-sizing: border-box; border: 1px solid var(--en-border, #c5cfdd); border-radius: 8px; background: var(--en-surface, #fff); color: var(--en-text, #101828); font: 10.5px/1.45 ui-monospace, SFMono-Regular, Menlo, monospace; }
.en-invite-box { display: grid; gap: 7px; }
.en-pair-connector { display: grid; place-items: center; color: var(--en-muted, #667085); }
.en-pair-connector svg { width: 17px; height: 17px; }
.en-security-note { display: flex; align-items: flex-start; gap: 8px; margin: 0 16px 14px; padding: 10px 11px; border: 1px solid color-mix(in srgb, var(--en-primary, #2563eb) 18%, var(--en-border, #c5cfdd)); border-radius: 9px; background: color-mix(in srgb, var(--en-primary, #2563eb) 5%, var(--en-bg, #f7f9fc)); color: var(--en-muted, #667085); }
.en-security-note svg { width: 15px; height: 15px; flex: 0 0 auto; color: var(--en-primary, #2563eb); }
.en-security-note p { font-size: 10.5px; line-height: 1.45; }
.en-conflict-icon.warning { color: #b45309; background: rgba(245, 158, 11, 0.12); }
.warning-card { border-color: color-mix(in srgb, #f59e0b 32%, var(--en-border, #c5cfdd)); }
code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 900px) {
  .en-sync-toolbar { align-items: stretch; flex-direction: column; }
  .en-sync-toolbar-actions { justify-content: flex-end; }
  .en-sync-summary { grid-template-columns: 1fr; }
  .en-sync-summary article + article { border-top: 1px solid var(--en-border, #c5cfdd); border-left: 0; }
  .en-pair-flow { grid-template-columns: 1fr; }
  .en-pair-connector { transform: rotate(90deg); }
}
@media (max-width: 620px) {
  .en-sync-tabs { width: 100%; }
  .en-sync-tabs button { flex: 1; font-size: 0; }
  .en-sync-tabs button svg, .en-sync-tab-count { font-size: initial; }
  .en-sync-toolbar-actions { flex-wrap: wrap; }
  .en-sync-status { margin-right: auto; }
  .en-sync-setting-row { align-items: flex-start; flex-direction: column; }
  .en-retention-control { width: 100%; flex-wrap: wrap; }
  .en-sync-device-row, .en-conflict-row { grid-template-columns: 32px minmax(0, 1fr); }
  .en-verified-badge, .en-preserved-badge, .en-conflict-actions { grid-column: 2; justify-self: start; }
}
</style>
