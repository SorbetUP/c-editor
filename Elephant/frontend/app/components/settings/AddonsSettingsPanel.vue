<template>
  <div class="en-addons-panel">
    <Teleport defer to="#en-addons-title-actions">
      <button
        class="en-community-title-check"
        :class="{ active: communityAddonsEnabled }"
        type="button"
        role="checkbox"
        aria-label="Community addons"
        :aria-checked="communityAddonsEnabled"
        :title="communityAddonsEnabled ? 'Disable community addons' : 'Enable community addons'"
        :disabled="!communityConsentLoaded || operationInProgress"
        @click="toggleCommunityAddons"
      >
        <Check v-if="communityAddonsEnabled" aria-hidden="true" />
      </button>
    </Teleport>

    <nav class="en-addons-tabs" aria-label="Addon settings pages">
      <button type="button" :class="{ active: activePage === 'addons' }" @click="activePage = 'addons'">
        <Package aria-hidden="true" />
        <span>Addons</span>
      </button>
      <button type="button" :class="{ active: activePage === 'packs' }" @click="activePage = 'packs'">
        <Layers3 aria-hidden="true" />
        <span>Addon packs</span>
      </button>
    </nav>

    <div class="en-addons-toolbar">
      <label class="en-addons-search">
        <Search aria-hidden="true" />
        <input
          v-if="activePage === 'addons'"
          v-model.trim="query"
          type="search"
          placeholder="Search addons"
          aria-label="Search addons"
        >
        <input
          v-else
          v-model.trim="packQuery"
          type="search"
          placeholder="Search addon packs"
          aria-label="Search addon packs"
        >
      </label>

      <label v-if="activePage === 'addons'" class="en-installed-only-control">
        <button
          class="en-addon-installed-filter"
          type="button"
          role="switch"
          aria-label="Installed only"
          :aria-checked="installedOnly"
          :class="{ active: installedOnly }"
          @click="installedOnly = !installedOnly"
        ><span /></button>
        <span>Installed only</span>
      </label>

      <button
        class="en-addons-toolbar-icon"
        type="button"
        aria-label="Refresh"
        title="Refresh"
        :disabled="operationInProgress || (activePage === 'addons' && catalogLoading)"
        @click="refreshActivePage"
      >
        <RefreshCw aria-hidden="true" />
      </button>
      <button
        class="en-addons-toolbar-icon primary"
        type="button"
        :aria-label="activePage === 'addons' ? 'Install addon from file' : 'Add addon pack from file'"
        :title="activePage === 'addons' ? 'Install addon from file' : 'Add addon pack from file'"
        :disabled="operationInProgress || (activePage === 'addons' && !communityAddonsEnabled)"
        @click="addFromFile"
      >
        <Plus aria-hidden="true" />
      </button>
    </div>

    <template v-if="activePage === 'addons'">
      <p v-if="message" class="en-addons-feedback" :class="{ error: messageIsError }">{{ message }}</p>
      <p v-if="lastError" class="en-addons-feedback error">{{ lastError }}</p>

      <section v-if="!selectedEntry" class="en-addon-catalogue" aria-label="Addon catalogue">
        <button
          v-for="entry in browserEntries"
          :key="entry.id"
          class="en-addon-tile"
          :class="{ installed: entry.installed }"
          :data-addon-id="entry.id"
          type="button"
          @click="openAddon(entry)"
        >
          <span class="en-addon-tile-icon"><AddonIcon :name="entry.manifest.icon" /></span>
          <span class="en-addon-tile-copy">
            <span class="en-addon-tile-title-row">
              <strong>{{ entry.manifest.name }}</strong>
              <small>v{{ entry.manifest.version }}</small>
            </span>
            <span class="en-addon-tile-description">{{ entry.manifest.description || 'No description.' }}</span>
            <span class="en-addon-tile-status">{{ entryStatusLabel(entry) }}</span>
          </span>
          <ChevronRight aria-hidden="true" />
        </button>

        <div v-if="communityAddonsEnabled && catalogLoading" class="en-addons-empty">Loading the addon catalogue…</div>
        <div v-else-if="communityAddonsEnabled && catalogError" class="en-addons-empty error"><strong>Catalogue unavailable</strong><span>{{ catalogError }}</span></div>
        <div v-else-if="!browserEntries.length" class="en-addons-empty">{{ query ? 'No addon matches this search.' : 'No addon is available.' }}</div>
      </section>

      <section v-else class="en-addon-browser" :class="{ 'en-addon-browser-detail-mode': true }">
        <aside class="en-addon-browser-sidebar">
          <header class="en-addon-browser-sidebar-header">
            <button class="en-addon-browser-back" type="button" @click="closeAddonDetails">
              <ArrowLeft aria-hidden="true" />
              <span>Catalogue</span>
            </button>
            <small>{{ browserEntries.length }} addon{{ browserEntries.length === 1 ? '' : 's' }}</small>
          </header>
          <div class="en-addon-browser-list" role="listbox" aria-label="Addon catalogue">
            <button
              v-for="entry in browserEntries"
              :key="entry.id"
              class="en-addon-browser-item"
              :class="{ active: selectedAddonId === entry.id, installed: entry.installed }"
              :data-addon-id="entry.id"
              type="button"
              role="option"
              :aria-selected="selectedAddonId === entry.id"
              @click="openAddon(entry)"
            >
              <span class="en-addon-browser-item-icon"><AddonIcon :name="entry.manifest.icon" /></span>
              <span class="en-addon-browser-item-copy">
                <strong>{{ entry.manifest.name }}</strong>
                <small>{{ entryStatusLabel(entry) }} · v{{ entry.manifest.version }}</small>
              </span>
              <ChevronRight aria-hidden="true" />
            </button>
          </div>
        </aside>

        <main class="en-addon-browser-detail" :data-selected-addon-id="selectedEntry.id">
          <header class="en-addon-detail-header">
            <span class="en-addon-detail-logo"><AddonIcon :name="selectedEntry.manifest.icon" /></span>
            <div class="en-addon-detail-heading">
              <div>
                <h2>{{ selectedEntry.manifest.name }}</h2>
                <span>v{{ selectedEntry.manifest.version }}</span>
              </div>
              <p>By {{ selectedEntry.manifest.author || 'Elephant' }}</p>
            </div>
            <div class="en-addon-detail-actions">
              <template v-if="selectedEntry.installed">
                <button
                  class="en-switch"
                  type="button"
                  role="switch"
                  :aria-label="`Enable ${selectedEntry.manifest.name}`"
                  :aria-checked="selectedEntry.snapshot.enabled"
                  :class="{ active: selectedEntry.snapshot.enabled }"
                  :disabled="operationInProgress || isCommunityLocked(selectedEntry.snapshot)"
                  @click="toggleSelectedAddon"
                ><span /></button>
                <button class="en-danger-button" type="button" :disabled="operationInProgress" @click="uninstallSelectedAddon">Uninstall</button>
              </template>
              <button v-else class="en-primary-button" type="button" :disabled="operationInProgress" @click="installSelectedAddon">Install</button>
            </div>
          </header>

          <p class="en-addon-detail-description">{{ selectedEntry.manifest.description || 'No description.' }}</p>

          <section v-if="selectedEntry.id === AI_PARENT_ID && aiModules.length" class="en-addon-detail-section">
            <header>
              <div><h3>AI modules</h3><p>Each module remains independently downloadable from the catalogue.</p></div>
              <span>{{ installedAiModuleCount }}/{{ aiModules.length }}</span>
            </header>
            <div class="en-ai-module-list">
              <article v-for="module in aiModules" :key="module.id" class="en-ai-module-row">
                <span class="en-ai-module-icon"><AddonIcon :name="module.manifest.icon" /></span>
                <div class="en-ai-module-copy">
                  <strong>{{ module.manifest.name }}</strong>
                  <span>{{ module.manifest.description }}</span>
                </div>
                <template v-if="module.installed">
                  <button
                    class="en-switch"
                    type="button"
                    role="switch"
                    :aria-label="`Enable ${module.manifest.name}`"
                    :aria-checked="module.snapshot.enabled"
                    :class="{ active: module.snapshot.enabled }"
                    :disabled="operationInProgress"
                    @click="toggleAddon(module.snapshot)"
                  ><span /></button>
                  <button class="en-addon-module-remove" type="button" :disabled="operationInProgress" @click="uninstallAddon(module.snapshot)">Uninstall</button>
                </template>
                <button v-else class="en-secondary-button" type="button" :disabled="operationInProgress" @click="installAiModule(module)">Install</button>
              </article>
            </div>
          </section>

          <section v-if="selectedPermissions.length" class="en-addon-detail-section">
            <header><div><h3>Capabilities</h3><p>Access requested by this addon.</p></div></header>
            <div class="en-addon-detail-permissions">
              <span v-for="permission in selectedPermissions" :key="permission">{{ permission }}</span>
            </div>
          </section>

          <section v-if="selectedActions.length" class="en-addon-detail-section">
            <header><div><h3>Commands</h3><p>Actions exposed by this addon.</p></div></header>
            <div class="en-addon-detail-commands">
              <button
                v-for="action in selectedActions"
                :key="action.id"
                class="en-secondary-button"
                type="button"
                :disabled="operationInProgress || !selectedEntry.snapshot?.enabled || !action.enabled"
                @click="runAction(action)"
              >{{ action.title }}</button>
            </div>
          </section>
        </main>
      </section>
    </template>

    <template v-else>
      <div class="en-addon-packs-slot" data-elephant-addon-settings-slot="addons.packs" />
    </template>
  </div>
</template>

<script setup>
import { computed, nextTick, ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { ArrowLeft, Check, ChevronRight, Layers3, Package, Plus, RefreshCw, Search } from '@lucide/vue'
import { useAddonsStore } from '@/store/addons'
import AddonIcon from './AddonIcon.vue'
import { useAddonsSettings } from './useAddonsSettings'

const PACK_SEARCH_EVENT = 'elephantnote:addon-packs-search'
const PACK_REFRESH_EVENT = 'elephantnote:addon-packs-refresh'
const PACK_IMPORT_EVENT = 'elephantnote:addon-packs-import'
const AI_PARENT_ID = 'elephant.ai'
const AI_SUBMODULE_IDS = Object.freeze([
  'elephant.ai-chat',
  'elephant.ai-search',
  'elephant.ai-ocr',
  'elephant.wiki',
  'elephant.graph'
])

const addonsStore = useAddonsStore()
const activePage = ref('addons')
const packQuery = ref('')
const installedOnly = ref(false)
const selectedAddonId = ref('')
const {
  query,
  message,
  messageIsError,
  catalogLoading,
  catalogError,
  communityAddonsEnabled,
  communityConsentLoaded,
  operationInProgress,
  lastError,
  filteredInstalledAddons,
  availableAddons,
  actionsForAddon,
  isCommunityLocked,
  refreshCatalog,
  enableCommunityAddons,
  disableCommunityAddons,
  installAvailableAddon,
  installAddonPackage,
  toggleAddon,
  uninstallAddon,
  runAction
} = useAddonsSettings()

const installedById = computed(() => new Map(addonsStore.items.map((addon) => [addon.manifest.id, addon])))
const builtinCatalogById = computed(() => new Map(
  (addonsStore.manager?.listBuiltinCatalog?.() || []).map((entry) => [entry.manifest.id, entry.manifest])
))
const availableById = computed(() => new Map(availableAddons.value.map((addon) => [addon.id, addon])))

const browserEntries = computed(() => {
  const entriesById = new Map()
  for (const snapshot of filteredInstalledAddons.value) {
    entriesById.set(snapshot.manifest.id, {
      id: snapshot.manifest.id,
      manifest: snapshot.manifest,
      snapshot,
      installed: true,
      available: null
    })
  }
  for (const available of availableAddons.value) {
    const existing = entriesById.get(available.id)
    if (existing) {
      existing.available = available
      continue
    }
    entriesById.set(available.id, {
      id: available.id,
      manifest: available,
      snapshot: null,
      installed: false,
      available
    })
  }
  return [...entriesById.values()]
    .filter((entry) => !installedOnly.value || entry.installed)
    .sort((left, right) => Number(right.installed) - Number(left.installed) || left.manifest.name.localeCompare(right.manifest.name))
})

const selectedEntry = computed(() => browserEntries.value.find((entry) => entry.id === selectedAddonId.value) || null)
const selectedActions = computed(() => selectedEntry.value?.installed ? actionsForAddon(selectedEntry.value.id) : [])
const selectedPermissions = computed(() => {
  const permissions = selectedEntry.value?.manifest?.permissions
  if (Array.isArray(permissions)) return permissions
  if (!permissions || typeof permissions !== 'object') return []
  const labels = []
  for (const [scope, value] of Object.entries(permissions)) {
    if (Array.isArray(value)) labels.push(...value.map((item) => `${scope}: ${item}`))
    else if (value && typeof value === 'object') {
      for (const [action, enabled] of Object.entries(value)) {
        if (Array.isArray(enabled)) labels.push(...enabled.map((item) => `${scope}.${action}: ${item}`))
        else if (enabled) labels.push(`${scope}.${action}`)
      }
    } else if (value) labels.push(scope)
  }
  return labels
})

const aiModules = computed(() => AI_SUBMODULE_IDS.map((id) => {
  const snapshot = installedById.value.get(id) || null
  const available = availableById.value.get(id) || null
  const manifest = snapshot?.manifest || available || builtinCatalogById.value.get(id)
  return manifest ? { id, manifest, snapshot, available, installed: Boolean(snapshot) } : null
}).filter(Boolean))
const installedAiModuleCount = computed(() => aiModules.value.filter((module) => module.installed).length)

const entryStatusLabel = (entry) => {
  if (entry?.available?.updateAvailable) return 'Update available'
  if (!entry?.installed) return 'Available'
  return entry.snapshot?.enabled ? 'Enabled' : 'Installed'
}

watch(browserEntries, (entries) => {
  if (!selectedAddonId.value) return
  if (!entries.some((entry) => entry.id === selectedAddonId.value)) selectedAddonId.value = ''
})

const openAddon = (entry) => {
  selectedAddonId.value = entry?.id || ''
}

const closeAddonDetails = () => {
  selectedAddonId.value = ''
}

const dispatchPackEvent = (name, detail = undefined) => {
  window.dispatchEvent(new CustomEvent(name, { detail }))
}

watch(packQuery, (value) => dispatchPackEvent(PACK_SEARCH_EVENT, { query: value }))
watch(activePage, async (page) => {
  if (page !== 'addons') selectedAddonId.value = ''
  await nextTick()
  if (page === 'packs') dispatchPackEvent(PACK_SEARCH_EVENT, { query: packQuery.value })
})

const toggleCommunityAddons = async () => {
  if (communityAddonsEnabled.value) await disableCommunityAddons()
  else await enableCommunityAddons()
}

const refreshActivePage = async () => {
  if (activePage.value === 'addons') await refreshCatalog()
  else dispatchPackEvent(PACK_REFRESH_EVENT)
}

const addFromFile = async () => {
  if (activePage.value === 'addons') {
    await installAddonPackage()
    return
  }
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'ElephantNote addon pack', extensions: ['enaddonpack'] }]
  })
  if (typeof selected === 'string' && selected) dispatchPackEvent(PACK_IMPORT_EVENT, { path: selected })
}

const installSelectedAddon = async () => {
  if (!selectedEntry.value?.available) return
  const id = selectedEntry.value.id
  await installAvailableAddon(selectedEntry.value.available)
  selectedAddonId.value = id
}

const toggleSelectedAddon = async () => {
  const snapshot = selectedEntry.value?.snapshot
  if (!snapshot) return
  await toggleAddon(snapshot)
}

const uninstallSelectedAddon = async () => {
  const snapshot = selectedEntry.value?.snapshot
  if (!snapshot) return
  await uninstallAddon(snapshot)
}

const installAiModule = async (module) => {
  let parent = installedById.value.get(AI_PARENT_ID)
  if (!parent) {
    const parentManifest = availableById.value.get(AI_PARENT_ID) || builtinCatalogById.value.get(AI_PARENT_ID)
    if (parentManifest) await installAvailableAddon(parentManifest)
    parent = addonsStore.manager?.get?.(AI_PARENT_ID)
  }
  if (parent && !parent.enabled) await toggleAddon(parent)
  const available = module.available || module.manifest
  await installAvailableAddon(available)
  const installed = addonsStore.manager?.get?.(module.id)
  if (installed && !installed.enabled) await toggleAddon(installed)
}
</script>

<style scoped src="./addons-settings.css"></style>
