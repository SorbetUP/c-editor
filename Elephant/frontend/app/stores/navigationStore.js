import { defineStore } from 'pinia'
import log from '@/platform/runtimeLogShim'
import { irohSyncClient } from '../services/irohSyncClient'

const isSameEntry = (a, b) => {
  if (!a || !b) return false
  if (a.type !== b.type) return false
  if (a.id || b.id) return a.id === b.id
  if (a.path || b.path) return a.path === b.path
  return a.type === b.type
}

const peersFromStatus = (status) => Array.isArray(status?.peers) ? status.peers : []

export const useNavigationStore = defineStore('elephantnoteNavigation', {
  state: () => ({
    history: [],
    index: -1,
    syncStatus: 'idle',
    syncError: '',
    syncDetails: null,
    syncChecked: false
  }),

  getters: {
    canGoBack: (state) => state.index > 0,
    canGoForward: (state) => state.index < state.history.length - 1,
    current: (state) => state.history[state.index] || null,
    syncPeers: (state) => peersFromStatus(state.syncDetails),
    hasPairedSyncDevice() {
      return this.syncPeers.length > 0
    }
  },

  actions: {
    push(entry) {
      if (!entry) return
      log.info('[navigation] push:start', {
        entry,
        current: this.current,
        index: this.index,
        historyLength: this.history.length
      })
      const normalized = { ...entry }
      if (this.history.length > 0 && this.index >= 0) {
        if (isSameEntry(this.history[this.index], normalized)) {
          log.info('[navigation] push:skip-same-entry', { entry: normalized })
          return
        }
        this.history = this.history.slice(0, this.index + 1)
      }
      this.history = [...this.history, normalized]
      this.index = this.history.length - 1
      if (this.history.length > 100) {
        this.history = this.history.slice(-80)
        this.index = this.history.length - 1
      }
      log.info('[navigation] push:done', {
        current: this.current,
        index: this.index,
        historyLength: this.history.length
      })
    },

    reset(entry) {
      log.info('[navigation] reset', { entry, previousLength: this.history.length })
      this.history = []
      this.index = -1
      this.push(entry)
    },

    back() {
      if (this.index > 0) {
        this.index -= 1
        log.info('[navigation] back', { entry: this.history[this.index], index: this.index })
        return this.history[this.index]
      }
      log.info('[navigation] back:empty', { index: this.index })
      return null
    },

    forward() {
      if (this.index < this.history.length - 1) {
        this.index += 1
        log.info('[navigation] forward', { entry: this.history[this.index], index: this.index })
        return this.history[this.index]
      }
      log.info('[navigation] forward:empty', { index: this.index })
      return null
    },

    applySyncStatus(status, { preserveRunning = false } = {}) {
      this.syncDetails = status && typeof status === 'object' ? status : null
      this.syncChecked = true
      const backendRunning = Boolean(status?.running)
      const hasExplicitRunning = typeof status?.running === 'boolean'
      const preserveLegacyRunning = preserveRunning &&
        !hasExplicitRunning &&
        this.syncStatus === 'syncing'
      if (backendRunning || preserveLegacyRunning) {
        this.syncStatus = 'syncing'
        this.syncError = ''
        return this.syncDetails
      }
      const lastError = String(status?.lastError || '').trim()
      if (lastError) {
        this.syncStatus = 'error'
        this.syncError = lastError
      } else if (Number(status?.lastRunAt || 0) > 0) {
        this.syncStatus = 'synced'
        this.syncError = ''
      } else {
        this.syncStatus = 'idle'
        this.syncError = ''
      }
      return this.syncDetails
    },

    clearSyncStatus() {
      this.syncStatus = 'idle'
      this.syncError = ''
      this.syncDetails = null
      this.syncChecked = false
    },

    async refreshSyncStatus({ preserveRunning = false } = {}) {
      try {
        const status = await irohSyncClient.status()
        return this.applySyncStatus(status, { preserveRunning })
      } catch (error) {
        this.syncChecked = true
        this.syncStatus = 'error'
        this.syncError = error?.message || 'Unable to read Iroh synchronization status.'
        throw error
      }
    },

    async syncWorkspace(vaultPath = '') {
      if (this.syncStatus === 'syncing') return this.syncDetails
      if (!this.syncChecked) await this.refreshSyncStatus()
      if (!this.hasPairedSyncDevice) {
        const error = new Error('No paired Iroh device is available for this vault.')
        this.syncStatus = 'error'
        this.syncError = error.message
        throw error
      }

      this.syncStatus = 'syncing'
      this.syncError = ''
      try {
        const result = await irohSyncClient.run()
        const lastError = String(result?.lastError || '').trim()
        if (lastError) throw new Error(lastError)
        if (!Array.isArray(result?.peers) || result.peers.length === 0) {
          throw new Error('Synchronization completed without a paired Iroh device.')
        }
        this.applySyncStatus(result)
        this.syncStatus = 'synced'
        if (vaultPath && Number(result?.transferredFiles || 0) > 0) {
          const { elephantnoteClient } = await import('../services/elephantnoteClient')
          await elephantnoteClient.search.initVault(vaultPath)
          await elephantnoteClient.search.rebuild()
        }
        return result
      } catch (error) {
        this.syncStatus = 'error'
        this.syncError = error?.message || 'Iroh synchronization failed.'
        throw error
      }
    }
  }
})
