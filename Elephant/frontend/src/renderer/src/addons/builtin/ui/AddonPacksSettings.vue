<template>
  <div class="en-addon-packs-settings">
    <section class="en-addon-pack-list-section">
      <header>
        <h3>Addon packs</h3>
        <span>{{ filteredPacks.length }}</span>
      </header>

      <div class="en-settings-group en-addon-pack-list">
        <article v-for="pack in filteredPacks" :key="pack.path" class="en-addon-pack-row">
          <span class="en-addon-pack-icon"><Layers3 aria-hidden="true" /></span>
          <div class="en-addon-pack-copy">
            <div>
              <strong>{{ pack.name }}</strong>
              <small>{{ pack.addonCount }} addon{{ pack.addonCount === 1 ? '' : 's' }}</small>
            </div>
            <p v-if="pack.description">{{ pack.description }}</p>
          </div>
          <div class="en-addon-pack-actions">
            <button
              :class="isPackInstalled(pack) ? 'en-danger-button' : 'en-primary-button'"
              type="button"
              :disabled="busy"
              @click="togglePack(pack)"
            >{{ isPackInstalled(pack) ? 'Uninstall' : (pack.protected ? 'Install' : 'Apply') }}</button>
            <button v-if="!pack.protected && confirmDeletePath !== pack.path" class="en-danger-button" type="button" :disabled="busy" @click="confirmDeletePath = pack.path">Delete</button>
            <template v-else-if="!pack.protected">
              <button class="en-danger-button" type="button" :disabled="busy" @click="deletePack(pack)">Confirm</button>
              <button class="en-secondary-button" type="button" :disabled="busy" @click="confirmDeletePath = ''">Cancel</button>
            </template>
          </div>
        </article>
        <div v-if="loading" class="en-addon-pack-empty">Loading addon packs…</div>
        <div v-else-if="!filteredPacks.length" class="en-addon-pack-empty">{{ query ? 'No addon pack matches this search.' : 'No addon pack installed.' }}</div>
      </div>
    </section>

    <p v-if="message" class="en-addons-feedback" :class="{ error: messageIsError }">{{ message }}</p>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { Layers3 } from '@lucide/vue'
import { useAddonsStore } from '@/store/addons'
import { elephantnoteClient } from 'elephant-front/services/elephantnoteClient'

const PACK_DIRECTORY = '.elephantnote/addons/packs'
const DEFAULT_PACK_PATH = `${PACK_DIRECTORY}/default.enaddonpack`
const BASE_PACK_PATH = `${PACK_DIRECTORY}/base.enaddonpack`
const DEVELOP_PARITY_PACK_PATH = `${PACK_DIRECTORY}/develop-parity.enaddonpack`
const PROTECTED_PACK_PATHS = new Set([BASE_PACK_PATH, DEVELOP_PARITY_PACK_PATH])
const PACK_SEARCH_EVENT = 'elephantnote:addon-packs-search'
const PACK_REFRESH_EVENT = 'elephantnote:addon-packs-refresh'
const PACK_IMPORT_EVENT = 'elephantnote:addon-packs-import'
const addonsStore = useAddonsStore()
const packs = ref([])
const query = ref('')
const loading = ref(false)
const busy = ref(false)
const message = ref('')
const messageIsError = ref(false)
const confirmDeletePath = ref('')

const filteredPacks = computed(() => {
  const normalized = query.value.trim().toLocaleLowerCase()
  if (!normalized) return packs.value
  return packs.value.filter((pack) => `${pack.name} ${pack.description}`.toLocaleLowerCase().includes(normalized))
})

const showMessage = (text, error = false) => {
  message.value = text
  messageIsError.value = error
}

const normalizeEntries = (result) => Array.isArray(result) ? result : Array.isArray(result?.entries) ? result.entries : []
const packPath = (entry) => {
  const path = String(entry?.path || entry?.relativePath || '')
  if (!path) return ''
  return path.includes('/') ? path : `${PACK_DIRECTORY}/${path}`
}

const decodeText = (data) => {
  if (typeof data === 'string') return data
  if (data instanceof ArrayBuffer) return new TextDecoder().decode(new Uint8Array(data))
  if (ArrayBuffer.isView(data)) return new TextDecoder().decode(new Uint8Array(data.buffer, data.byteOffset, data.byteLength))
  throw new Error('Unable to read this addon pack as text')
}

const validateImportedPack = (raw, sourcePath) => {
  let parsed
  try {
    parsed = JSON.parse(raw)
  } catch (error) {
    throw new Error(`Invalid JSON in ${sourcePath}: ${error.message}`)
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error('Addon pack must be a JSON object')
  if (parsed.format !== 'elephantnote-addon-pack') throw new Error('Unsupported addon pack format')
  if (parsed.version !== 1) throw new Error(`Unsupported addon pack version: ${parsed.version}`)
  if (!Array.isArray(parsed.addons)) throw new Error('Addon pack must contain an addons array')
  return parsed
}

const importedFilename = (sourcePath) => {
  const base = String(sourcePath || '').replaceAll('\\', '/').split('/').pop() || `imported-${Date.now()}.enaddonpack`
  const safe = base.replace(/[^a-zA-Z0-9._-]+/g, '-').replace(/^-+|-+$/g, '')
  return (safe || `imported-${Date.now()}.enaddonpack`).toLowerCase().endsWith('.enaddonpack')
    ? (safe || `imported-${Date.now()}.enaddonpack`)
    : `${safe || `imported-${Date.now()}`}.enaddonpack`
}

const readPack = async (path) => {
  const note = await elephantnoteClient.notes.read(path)
  const raw = typeof note === 'string' ? note : note?.content
  const parsed = validateImportedPack(String(raw || ''), path)
  return {
    path,
    name: String(parsed.name || path.split('/').pop()?.replace(/\.enaddonpack$/i, '') || 'Unnamed addon pack'),
    description: String(parsed.description || ''),
    addonCount: parsed.addons.length,
    addons: parsed.addons.map((entry) => ({
      id: String(entry?.id || ''),
      source: String(entry?.source || 'installed'),
      enabled: entry?.enabled === true
    })).filter((entry) => entry.id),
    createdAt: String(parsed.createdAt || ''),
    protected: parsed.protected === true || PROTECTED_PACK_PATHS.has(path)
  }
}

const discoverPackPaths = async () => {
  try {
    const entries = normalizeEntries(await elephantnoteClient.directory.list(PACK_DIRECTORY))
    const paths = entries.map(packPath).filter((path) => path.toLowerCase().endsWith('.enaddonpack'))
    if (paths.length) return [...new Set(paths)]
  } catch {
    // The hidden pack directory may not exist yet. Known paths are checked below.
  }
  const paths = []
  for (const path of [BASE_PACK_PATH, DEVELOP_PARITY_PACK_PATH, DEFAULT_PACK_PATH]) {
    try {
      await elephantnoteClient.notes.read(path)
      paths.push(path)
    } catch {
      // Optional pack is absent.
    }
  }
  return paths
}

const removablePackEntries = (pack) => (pack?.addons || []).filter((entry) => entry.source !== 'installed')

const isPackInstalled = (pack) => {
  const removable = removablePackEntries(pack)
  if (!removable.length) return false
  const installedIds = new Set(addonsStore.items.map((addon) => addon.manifest.id))
  return removable.every((entry) => installedIds.has(entry.id))
}

const loadPacks = async () => {
  loading.value = true
  try {
    await addonsStore.runAction('elephant.addon-packs.ensure-develop-parity')
    const paths = await discoverPackPaths()
    const results = await Promise.allSettled(paths.map(readPack))
    packs.value = results
      .filter((result) => result.status === 'fulfilled')
      .map((result) => result.value)
      .sort((left, right) => Number(right.protected) - Number(left.protected) || right.createdAt.localeCompare(left.createdAt) || left.name.localeCompare(right.name))
    const invalidCount = results.filter((result) => result.status === 'rejected').length
    if (invalidCount) showMessage(`${invalidCount} invalid addon pack${invalidCount === 1 ? ' was' : 's were'} ignored.`, true)
  } catch (error) {
    packs.value = []
    showMessage(error instanceof Error ? error.message : String(error), true)
  } finally {
    loading.value = false
  }
}

const importPack = async (sourcePath) => {
  if (!sourcePath || busy.value) return
  busy.value = true
  showMessage('')
  try {
    const raw = decodeText(await globalThis.fileUtils.readFile(sourcePath))
    const parsed = validateImportedPack(raw, sourcePath)
    const filename = importedFilename(sourcePath)
    let destination = `${PACK_DIRECTORY}/${filename}`
    if (packs.value.some((pack) => pack.path === destination)) {
      destination = `${PACK_DIRECTORY}/${filename.replace(/\.enaddonpack$/i, '')}-${Date.now()}.enaddonpack`
    }
    await elephantnoteClient.notes.write({
      relativePath: destination,
      content: `${JSON.stringify(parsed, null, 2)}\n`
    })
    showMessage(`Added ${parsed.name || filename}.`)
    await loadPacks()
  } catch (error) {
    showMessage(error instanceof Error ? error.message : String(error), true)
  } finally {
    busy.value = false
  }
}

const applyPack = async (pack) => {
  if (!pack?.path || busy.value) return
  busy.value = true
  showMessage('')
  try {
    await addonsStore.runAction('elephant.addon-packs.apply', { path: pack.path })
    addonsStore.refresh()
  } catch (error) {
    showMessage(error instanceof Error ? error.message : String(error), true)
  } finally {
    busy.value = false
  }
}

const uninstallPack = async (pack) => {
  if (!pack || busy.value) return
  busy.value = true
  showMessage('')
  try {
    for (const entry of [...removablePackEntries(pack)].reverse()) {
      const current = addonsStore.manager?.get(entry.id)
      if (!current) continue
      if (current.manifest.source === 'external') await addonsStore.uninstallExternalAddon(entry.id)
      else await addonsStore.manager.uninstallBuiltin(entry.id)
    }
    addonsStore.refresh()
  } catch (error) {
    addonsStore.refresh()
    showMessage(error instanceof Error ? error.message : String(error), true)
  } finally {
    busy.value = false
  }
}

const togglePack = (pack) => isPackInstalled(pack) ? uninstallPack(pack) : applyPack(pack)

const deletePack = async (pack) => {
  if (!pack?.path || pack.protected || busy.value) return
  busy.value = true
  try {
    await elephantnoteClient.entries.delete(pack.path)
    confirmDeletePath.value = ''
    showMessage(`Deleted ${pack.name}.`)
    await loadPacks()
  } catch (error) {
    showMessage(error instanceof Error ? error.message : String(error), true)
  } finally {
    busy.value = false
  }
}

const handleSearch = (event) => { query.value = String(event?.detail?.query || '') }
const handleRefresh = () => { void loadPacks() }
const handleImport = (event) => { void importPack(String(event?.detail?.path || '')) }

onMounted(() => {
  window.addEventListener(PACK_SEARCH_EVENT, handleSearch)
  window.addEventListener(PACK_REFRESH_EVENT, handleRefresh)
  window.addEventListener(PACK_IMPORT_EVENT, handleImport)
  void loadPacks()
})

onBeforeUnmount(() => {
  window.removeEventListener(PACK_SEARCH_EVENT, handleSearch)
  window.removeEventListener(PACK_REFRESH_EVENT, handleRefresh)
  window.removeEventListener(PACK_IMPORT_EVENT, handleImport)
})
</script>

<style scoped>
.en-addon-packs-settings { display: grid; gap: 16px; }
.en-addon-pack-list-section { display: grid; gap: 8px; }
.en-addon-pack-list-section > header { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 0 2px; }
.en-addon-pack-list-section h3 { margin: 0; font-size: 13px; }
.en-addon-pack-list-section > header > span { color: var(--en-muted, #667085); font-size: 10px; }
.en-addon-pack-list { display: grid; }
.en-addon-pack-row { min-height: 78px; display: grid; grid-template-columns: 36px minmax(0, 1fr) auto; align-items: center; gap: 12px; padding: 13px 15px; }
.en-addon-pack-row + .en-addon-pack-row { border-top: 1px solid var(--en-border, #c5cfdd); }
.en-addon-pack-icon { width: 36px; height: 36px; display: grid; place-items: center; border-radius: 9px; background: var(--en-soft, #e9eff7); color: var(--en-primary, #2563eb); }
.en-addon-pack-icon svg { width: 17px; height: 17px; }
.en-addon-pack-copy { min-width: 0; display: grid; gap: 3px; }
.en-addon-pack-copy > div { display: flex; align-items: baseline; gap: 8px; flex-wrap: wrap; }
.en-addon-pack-copy strong { font-size: 12.5px; }
.en-addon-pack-copy small, .en-addon-pack-copy p { color: var(--en-muted, #667085); font-size: 10px; }
.en-addon-pack-copy p { margin: 0; line-height: 1.4; }
.en-addon-pack-actions { display: flex; align-items: center; justify-content: flex-end; gap: 8px; flex-wrap: wrap; }
.en-addon-pack-empty { padding: 28px 16px; text-align: center; color: var(--en-muted, #667085); font-size: 12px; }
@media (max-width: 760px) {
  .en-addon-pack-row { grid-template-columns: 36px minmax(0, 1fr); }
  .en-addon-pack-actions { grid-column: 1 / -1; justify-content: stretch; }
  .en-addon-pack-actions button { flex: 1; }
}
</style>
