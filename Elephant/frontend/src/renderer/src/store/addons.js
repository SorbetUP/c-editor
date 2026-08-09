import { defineStore } from 'pinia'
import { markRaw } from 'vue'
import { isTrustedAddonManifest } from '@/addons/manifest'

const COMMUNITY_ADDONS_PREF_KEY = 'addons.communityEnabled'
const ADDON_STORE_EVENTS = [
  'registered',
  'unregistered',
  'changed',
  'enabled',
  'disabled',
  'error',
  'contribution:changed'
]

const cloneContributionMap = (map = {}) => Object.fromEntries(
  Object.entries(map).map(([area, entries]) => [area, Array.isArray(entries) ? [...entries] : []])
)

const invokeTauri = (command, payload = {}) => {
  const invoke = globalThis?.__TAURI__?.core?.invoke
  if (typeof invoke !== 'function') throw new Error(`Tauri command API is unavailable for ${command}`)
  return invoke(command, payload)
}

const setExternalRegistryEnabled = (id, enabled) => invokeTauri('tauri_addons_set_enabled', { addonId: id, enabled })
const readCommunityAddonsEnabled = async () => await invokeTauri('tauri_prefs_get', { key: COMMUNITY_ADDONS_PREF_KEY }) === true
const persistCommunityAddonsEnabled = (enabled) => invokeTauri('tauri_prefs_set', {
  key: COMMUNITY_ADDONS_PREF_KEY,
  value: enabled === true
})
const isOfficialManifest = (manifest = {}) => manifest.official === true || manifest.source === 'official'
const isOfficialCatalogEntry = (entry = {}) => entry.official === true || entry.source === 'official'
const isCommunityExternal = (addon = {}) => addon?.manifest?.source === 'external' && !isOfficialManifest(addon.manifest)

const entryArray = (value) => Array.isArray(value) ? value : []

const parentDirectory = (relativePath = '') => {
  const parts = String(relativePath || '').split('/').filter(Boolean)
  return parts.length > 1 ? parts.slice(0, -1).join('/') : ''
}

const refreshVaultAfterAddonAction = async (result, logger) => {
  const relativePath = typeof result?.path === 'string' ? result.path : ''
  if (!relativePath) return
  const [{ useVaultStore }, { elephantnoteClient }] = await Promise.all([
    import('elephant-front/stores/vaultStore'),
    import('elephant-front/services/elephantnoteClient')
  ])
  const vaultStore = useVaultStore()
  if (!vaultStore.activeVault?.path) return
  const targetDirectory = parentDirectory(relativePath)
  logger?.info?.('[addons] vault-refresh:start', { path: relativePath, targetDirectory })
  const rootEntries = entryArray(await elephantnoteClient.directory.list(''))
  vaultStore.rootEntries = rootEntries
  vaultStore.entries = vaultStore.currentPath
    ? entryArray(await elephantnoteClient.directory.list(vaultStore.currentPath))
    : rootEntries
  const targetEntries = targetDirectory === vaultStore.currentPath
    ? vaultStore.entries
    : targetDirectory
      ? entryArray(await elephantnoteClient.directory.list(targetDirectory))
      : rootEntries
  const createdEntry = targetEntries.find((entry) => entry?.path === relativePath)
  if (createdEntry) vaultStore.openNote(createdEntry)
  logger?.info?.('[addons] vault-refresh:done', { path: relativePath, found: Boolean(createdEntry) })
}

export const useAddonsStore = defineStore('addons', {
  state: () => ({
    installed: false,
    items: [],
    contributions: {},
    catalog: [],
    catalogLoading: false,
    catalogError: null,
    lastError: null,
    operationInProgress: false,
    communityAddonsEnabled: false,
    communityConsentLoaded: false,
    trustedApprovals: {},
    trustedSafeMode: false,
    trustedStateLoaded: false,
    manager: null,
    disposeListeners: []
  }),

  getters: {
    enabledAddons: (state) => state.items.filter((item) => item.enabled),
    externalAddons: (state) => state.items.filter((item) => item.manifest.source === 'external'),
    communityExternalAddons: (state) => state.items.filter(isCommunityExternal),
    officialAddons: (state) => state.items.filter((item) => isOfficialManifest(item.manifest)),
    trustedAddons: (state) => state.items.filter((item) => isTrustedAddonManifest(item.manifest)),
    failedAddons: (state) => state.items.filter((item) => item.status === 'error'),
    contributionCount: (state) => Object.values(state.contributions).reduce(
      (total, entries) => total + (Array.isArray(entries) ? entries.length : 0),
      0
    ),
    getContributions: (state) => (area) => [...(state.contributions[area] || [])],
    isTrustedApproved: (state) => (id) => state.trustedApprovals[id]?.approved === true
  },

  actions: {
    install(manager) {
      if (!manager) throw new TypeError('Addon manager is required')
      this.uninstall()
      this.manager = markRaw(manager)
      this.installed = true
      this.refresh()
      void this.loadCommunityAddonsConsent()
      const refresh = () => this.refresh()
      this.disposeListeners = ADDON_STORE_EVENTS.map((eventName) => manager.on(eventName, refresh))
    },

    uninstall() {
      for (const dispose of this.disposeListeners) {
        try { dispose() } catch { /* UI mirror cleanup must not block shutdown. */ }
      }
      this.disposeListeners = []
      this.manager = null
      this.installed = false
      this.items = []
      this.contributions = {}
      this.catalog = []
      this.catalogLoading = false
      this.catalogError = null
      this.lastError = null
      this.operationInProgress = false
      this.communityAddonsEnabled = false
      this.communityConsentLoaded = false
      this.trustedApprovals = {}
      this.trustedSafeMode = false
      this.trustedStateLoaded = false
    },

    refresh() {
      if (!this.manager) return
      this.items = this.manager.list()
      this.contributions = cloneContributionMap(this.manager.getContributionMap())
    },

    async loadAddonCatalog() {
      this.catalogLoading = true
      try {
        const entries = await invokeTauri('tauri_addons_catalog_list')
        this.catalog = Array.isArray(entries) ? entries : []
        this.catalogError = null
        return this.catalog
      } catch (error) {
        this.catalog = []
        this.catalogError = error?.message || String(error)
        throw error
      } finally {
        this.catalogLoading = false
      }
    },

    async loadCommunityAddonsConsent() {
      try {
        this.communityAddonsEnabled = await readCommunityAddonsEnabled()
        this.lastError = null
      } catch (error) {
        this.communityAddonsEnabled = false
        this.lastError = error?.message || String(error)
      } finally {
        this.communityConsentLoaded = true
      }
      return this.communityAddonsEnabled
    },

    async loadTrustedState() {
      if (!this.manager?.external) return false
      this.trustedSafeMode = await this.manager.external.getSafeMode()
      const approvals = {}
      for (const addon of this.manager.list()) {
        if (addon.manifest.source !== 'external' || !isTrustedAddonManifest(addon.manifest)) continue
        approvals[addon.manifest.id] = await this.manager.external.getTrustState(addon.manifest.id)
      }
      this.trustedApprovals = approvals
      this.trustedStateLoaded = true
      return true
    },

    async approveTrustedAddon(id) {
      if (!this.manager?.external) throw new Error('External addon runtime is not available')
      const approval = await this.manager.external.approveTrusted(id)
      this.trustedApprovals = { ...this.trustedApprovals, [id]: approval }
      return approval
    },

    async revokeTrustedAddon(id) {
      if (!this.manager?.external) throw new Error('External addon runtime is not available')
      const addon = this.manager.get(id)
      if (isOfficialManifest(addon?.manifest)) throw new Error('Verified official package trust cannot be revoked independently from its package')
      if (addon?.enabled) await this.disableAddon(id)
      const approval = await this.manager.external.revokeTrusted(id)
      this.trustedApprovals = { ...this.trustedApprovals, [id]: approval }
      return approval
    },

    async setTrustedSafeMode(enabled) {
      if (!this.manager?.external) throw new Error('External addon runtime is not available')
      const nextEnabled = await this.manager.external.setSafeMode(enabled)
      this.trustedSafeMode = nextEnabled
      if (nextEnabled) {
        const running = this.manager.list().filter(
          (addon) => addon.enabled && isCommunityExternal(addon) && isTrustedAddonManifest(addon.manifest)
        )
        for (const addon of running) await this.disableAddon(addon.manifest.id)
      }
      return nextEnabled
    },

    async setCommunityAddonsEnabled(enabled) {
      if (!this.manager) throw new Error('Addon manager is not installed')
      this.operationInProgress = true
      try {
        const nextEnabled = enabled === true
        await persistCommunityAddonsEnabled(nextEnabled)
        this.communityAddonsEnabled = nextEnabled
        this.communityConsentLoaded = true
        if (!nextEnabled) {
          const running = this.manager.list().filter((addon) => addon.enabled && isCommunityExternal(addon))
          for (const addon of running) await this.manager.disable(addon.manifest.id)
        }
        this.lastError = null
      } catch (error) {
        this.lastError = error?.message || String(error)
        throw error
      } finally {
        this.operationInProgress = false
        this.refresh()
      }
    },

    async enableAddon(id) {
      if (!this.manager) throw new Error('Addon manager is not installed')
      const addon = this.manager.get(id)
      const external = addon?.manifest?.source === 'external'
      const official = isOfficialManifest(addon?.manifest)
      const trusted = external && isTrustedAddonManifest(addon.manifest)
      this.operationInProgress = true
      try {
        if (external) {
          if (!official) {
            if (!this.communityConsentLoaded) await this.loadCommunityAddonsConsent()
            if (!this.communityAddonsEnabled) throw new Error('Community addons are disabled.')
            if (trusted) {
              if (!this.trustedStateLoaded) await this.loadTrustedState()
              if (this.trustedSafeMode) throw new Error('Trusted addon safe mode is enabled.')
              if (!this.trustedApprovals[id]?.approved) {
                const error = new Error('Full app access approval is required.')
                error.code = 'TRUST_REQUIRED'
                error.addonId = id
                throw error
              }
            }
          }
          await setExternalRegistryEnabled(id, true)
        }
        const result = await this.manager.enable(id)
        this.lastError = null
        return result
      } catch (error) {
        if (external) await setExternalRegistryEnabled(id, false).catch(() => {})
        this.lastError = error?.message || String(error)
        throw error
      } finally {
        this.operationInProgress = false
        this.refresh()
      }
    },

    async disableAddon(id) {
      if (!this.manager) throw new Error('Addon manager is not installed')
      this.operationInProgress = true
      try {
        const result = await this.manager.disable(id)
        this.lastError = null
        return result
      } catch (error) {
        this.lastError = error?.message || String(error)
        throw error
      } finally {
        this.operationInProgress = false
        this.refresh()
      }
    },

    async setAddonEnabled(id, enabled) {
      return enabled ? this.enableAddon(id) : this.disableAddon(id)
    },

    async installExternalAddon(packagePath) {
      if (!this.manager?.external) throw new Error('External addon runtime is not available')
      this.operationInProgress = true
      try {
        const result = await this.manager.external.installFromPath(packagePath)
        await this.loadTrustedState()
        this.lastError = null
        return result
      } finally {
        this.operationInProgress = false
        this.refresh()
      }
    },

    async installCatalogAddon(id) {
      if (!this.manager?.external) throw new Error('External addon runtime is not available')
      const entry = this.catalog.find((candidate) => candidate?.id === id)
      const official = isOfficialCatalogEntry(entry)
      if (!official && !this.communityAddonsEnabled) {
        throw new Error('Community addons must be enabled before installing a third-party catalogue package')
      }
      this.operationInProgress = true
      try {
        const record = await invokeTauri('tauri_addons_catalog_install', { addonId: id })
        const existing = this.manager.get(record.manifest.id)
        if (existing) {
          await this.manager.disable(record.manifest.id).catch(() => {})
          this.manager.unregister(record.manifest.id)
        }
        this.manager.external.register(record)
        await this.loadTrustedState()
        this.lastError = null
        return record
      } finally {
        this.operationInProgress = false
        this.refresh()
      }
    },

    async uninstallExternalAddon(id) {
      if (!this.manager?.external) throw new Error('External addon runtime is not available')
      this.operationInProgress = true
      try {
        await this.manager.external.uninstall(id)
        const approvals = { ...this.trustedApprovals }
        delete approvals[id]
        this.trustedApprovals = approvals
        this.lastError = null
      } finally {
        this.operationInProgress = false
        this.refresh()
      }
    },

    async runAction(id, payload = undefined) {
      if (!this.manager) throw new Error('Addon manager is not installed')
      this.operationInProgress = true
      this.manager.logger?.info?.('[addons] action:start', { id })
      try {
        const result = await this.manager.runAction(id, payload)
        await refreshVaultAfterAddonAction(result, this.manager.logger)
        this.lastError = null
        this.manager.logger?.info?.('[addons] action:done', { id, result })
        return result
      } catch (error) {
        this.lastError = error?.message || String(error)
        this.manager.logger?.error?.('[addons] action:failed', { id, error: this.lastError })
        throw error
      } finally {
        this.operationInProgress = false
        this.refresh()
      }
    }
  }
})
