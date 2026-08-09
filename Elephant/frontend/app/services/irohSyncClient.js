import { invoke } from '@tauri-apps/api/core'

export const IROH_SYNC_STATUS_EVENT = 'elephantnote:iroh-sync-status'

let lastPublishedStatus = null
let addonActive = false

const normalizeObject = (value) => (value && typeof value === 'object' ? value : {})

const requireActive = () => {
  if (!addonActive) throw new Error('The Sync addon is disabled.')
}

const publishStatus = (status) => {
  if (status && typeof status === 'object') lastPublishedStatus = status
  if (typeof window !== 'undefined' && status && typeof status === 'object') {
    window.dispatchEvent(new CustomEvent(IROH_SYNC_STATUS_EVENT, { detail: status }))
  }
  return status
}

const readStatus = async () => {
  requireActive()
  return publishStatus(await invoke('tauri_sync_status'))
}

const acceptInvite = async (manualCode) => {
  requireActive()
  const result = await invoke('tauri_sync_accept_invite', {
    invite: { manualCode: String(manualCode || '').trim() }
  })
  if (result?.status) publishStatus(result.status)
  return result
}

const run = async () => {
  requireActive()
  if (lastPublishedStatus) {
    publishStatus({ ...lastPublishedStatus, running: true, lastError: '' })
  }
  try {
    return publishStatus(await invoke('tauri_sync_run', {
      payloadByOperation: { sync: {} }
    }))
  } catch (error) {
    if (lastPublishedStatus) {
      publishStatus({
        ...lastPublishedStatus,
        running: false,
        lastError: error?.message || 'Iroh synchronization failed.'
      })
    }
    throw error
  }
}

const activate = () => {
  addonActive = true
}

const shutdown = async () => {
  addonActive = false
  const result = await invoke('tauri_sync_shutdown')
  lastPublishedStatus = null
  return publishStatus({ ...(result || {}), running: false, started: false })
}

export const irohSyncClient = {
  activate,
  status: readStatus,
  shutdown,
  createInvite: (payload = {}) => {
    requireActive()
    return invoke('tauri_sync_create_invite', { payload: normalizeObject(payload) })
  },
  acceptInvite,
  run,
  conflictSettings: () => {
    requireActive()
    return invoke('tauri_sync_conflict_settings_get')
  },
  setConflictRetentionDays: (days) => {
    requireActive()
    return invoke('tauri_sync_conflict_settings_set', {
      conflictRetentionDays: Number(days)
    })
  },
  restoreConflict: (relativePath) => {
    requireActive()
    return invoke('tauri_sync_conflict_restore', { relativePath: String(relativePath || '') })
  },
  deleteConflict: (relativePath) => {
    requireActive()
    return invoke('tauri_sync_conflict_delete', { relativePath: String(relativePath || '') })
  }
}
