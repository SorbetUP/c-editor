import { defineStore } from 'pinia'
import { useVaultStore } from '../stores/vaultStore'
import { elephantnoteClient } from '../services/elephantnoteClient'

const resolveFolderPath = (vaultRoot, relativePath) => {
  const cleanRoot = String(vaultRoot || '').replace(/[\\/]+$/, '')
  const cleanRelative = String(relativePath || '').replace(/^[\\/]+/, '')
  return cleanRelative ? `${cleanRoot}/${cleanRelative}` : cleanRoot
}

export const useSitePreviewStore = defineStore('elephantnoteSitePreview', {
  state: () => ({
    status: 'idle',
    info: null,
    error: '',
    lastBuild: null
  }),

  getters: {
    isBusy: (state) => state.status === 'preparing' || state.status === 'building' || state.status === 'serving',
    previewUrl: (state) => state.info?.url || ''
  },

  actions: {
    createPayload(entry) {
      const vaultStore = useVaultStore()
      if (!vaultStore.activeVault?.path || !entry?.path) {
        throw new Error('A folder inside the active vault is required.')
      }
      return {
        vaultRoot: vaultStore.activeVault.path,
        folderPath: resolveFolderPath(vaultStore.activeVault.path, entry.path)
      }
    },

    async previewFolder(entry) {
      this.status = 'preparing'
      this.error = ''
      try {
        const info = await elephantnoteClient.sitePreview.previewFolder(this.createPayload(entry))
        this.info = info
        this.status = info.status
        this.error = info.error || ''
        return info
      } catch (err) {
        this.status = 'error'
        this.error = err.message || 'Website preview failed. See logs for details.'
        throw err
      }
    },

    async buildFolder(entry) {
      this.status = 'building'
      this.error = ''
      try {
        const info = await elephantnoteClient.sitePreview.buildFolder(this.createPayload(entry))
        this.lastBuild = info
        this.status = info.status
        this.error = info.error || ''
        return info
      } catch (err) {
        this.status = 'error'
        this.error = err.message || 'Website build failed.'
        throw err
      }
    },

    async stopPreview() {
      if (!this.info?.id) return
      const info = await elephantnoteClient.sitePreview.stop(this.info.id)
      this.info = info
      this.status = info?.status || 'stopped'
    },

    async openPreviewExternal() {
      if (!this.info?.url) return
      await elephantnoteClient.sitePreview.openExternal(this.info.url)
    },

    async openBuildExternal() {
      if (!this.lastBuild?.outputDir) return
      await window.tauri.shell.openPath(this.lastBuild.outputDir)
    },

    clear() {
      this.status = 'idle'
      this.info = null
      this.error = ''
    }
  }
})
