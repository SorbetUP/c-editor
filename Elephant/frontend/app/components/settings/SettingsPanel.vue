<template>
  <div
    class="en-settings-backdrop"
    :class="[`en-theme-${themeMode}`, `en-theme-${themeClassId}`]"
    :style="settingsStyle"
    @click.self="emit('close')"
  >
    <section
      class="en-settings-panel"
      :class="{ 'is-macos': isMacOS }"
      :style="settingsStyle"
      role="dialog"
      aria-modal="true"
      aria-label="ElephantNote settings"
    >
      <header class="en-settings-header">
        <button class="en-icon-button en-settings-close" type="button" aria-label="Close settings" @click="emit('close')"><X aria-hidden="true" /></button>
        <h2>Settings</h2>
        <label class="en-settings-search">
          <Search aria-hidden="true" />
          <input ref="searchInput" v-model="settingsQuery" type="search" placeholder="Search all settings" aria-label="Search all settings">
          <kbd v-if="!settingsQuery">{{ isMacOS ? '⌘' : 'Ctrl' }} F</kbd>
        </label>
      </header>

      <div class="en-settings-grid">
        <aside class="en-settings-nav" aria-label="Settings sections">
          <button v-for="item in sections" :key="item.id" type="button" :class="{ active: !settingsQuery && activeSection === item.id }" @click="selectSection(item.id)">
            <component :is="item.icon" aria-hidden="true" />
            <span>{{ item.label }}</span>
            <ChevronRight class="en-settings-nav-chevron" aria-hidden="true" />
          </button>
          <footer class="en-settings-nav-footer"><span>Local-first</span><span>v0.1.0</span></footer>
        </aside>

        <main
          ref="settingsContent"
          class="en-settings-content"
          :class="{ 'is-addons': !settingsQuery.trim() && activeSection === 'addons' }"
          :data-active-section="settingsQuery.trim() ? 'search' : activeSection"
        >
          <template v-if="settingsQuery.trim()">
            <div class="en-settings-page-title"><h1>Search</h1><span>{{ searchResults.length }} result{{ searchResults.length === 1 ? '' : 's' }}</span></div>
            <section v-if="searchResults.length" class="en-settings-search-results">
              <button v-for="result in searchResults" :key="result.id" type="button" @click="openSearchResult(result)">
                <span class="en-settings-result-icon"><component :is="result.icon" aria-hidden="true" /></span>
                <span class="en-settings-result-copy"><strong>{{ result.label }}</strong><small>{{ result.description }}</small></span>
                <span class="en-settings-result-section">{{ result.sectionLabel }}</span>
                <ChevronRight aria-hidden="true" />
              </button>
            </section>
            <div v-else class="en-settings-empty-state en-settings-search-empty"><Search aria-hidden="true" /><strong>No setting found</strong><span>Try another word, feature name or control.</span></div>
          </template>

          <template v-else>
            <div class="en-settings-page-title">
              <h1>{{ activeSectionMeta.label }}</h1>
            </div>

            <template v-if="activeSection === 'appearance'">
              <section class="en-settings-group">
                <div class="en-settings-row">
                  <div class="en-settings-row-copy"><strong>Color mode</strong><span>Use the light or dark variant of the selected theme.</span></div>
                  <div class="en-segmented" aria-label="Color mode">
                    <button type="button" :class="{ active: themeMode === 'light' }" @click="emit('update-theme', getThemeVariant(activeThemeFamily.id, 'light'))"><SunMedium aria-hidden="true" /> Light</button>
                    <button type="button" :class="{ active: themeMode === 'dark' }" @click="emit('update-theme', getThemeVariant(activeThemeFamily.id, 'dark'))"><Moon aria-hidden="true" /> Dark</button>
                  </div>
                </div>

                <language-settings-row />

                <div class="en-settings-row en-settings-row-stacked">
                  <header class="en-settings-collapsible-header">
                    <strong>Theme</strong>
                    <button type="button" :aria-expanded="themeExpanded" :title="themeExpanded ? 'Collapse themes' : 'Expand themes'" @click="themeExpanded = !themeExpanded">
                      <ChevronDown :class="{ collapsed: !themeExpanded }" aria-hidden="true" />
                    </button>
                  </header>
                  <div v-if="themeExpanded" class="en-theme-grid">
                    <button v-for="family in themeFamilies" :key="family.id" type="button" class="en-theme-card" :class="{ active: activeThemeFamily.id === family.id }" @click="emit('update-theme', getThemeVariant(family.id, themeMode))">
                      <span class="en-theme-card-preview" :style="{ background: family.swatches[0] }"><i class="sidebar" :style="{ background: family.swatches[1] }" /><i class="canvas" :style="{ background: family.swatches[2] }"><b :style="{ background: family.swatches[3] || family.swatches[2] }" /><b :style="{ background: family.swatches[3] || family.swatches[2] }" /></i></span>
                      <span class="en-theme-card-copy"><strong>{{ family.name }}</strong><small>{{ family.description }}</small></span>
                      <span v-if="activeThemeFamily.id === family.id" class="en-theme-card-check"><Check aria-hidden="true" /></span>
                    </button>
                  </div>
                </div>

                <div class="en-settings-row">
                  <div class="en-settings-row-copy"><strong>Floating surfaces</strong><span>Lift navigation, controls and the writing surface above the background.</span></div>
                  <button class="en-switch" type="button" role="switch" aria-label="Floating surfaces" :aria-checked="preferences.floatingSurfaces" :class="{ active: preferences.floatingSurfaces }" @click="setPreference('floatingSurfaces', !preferences.floatingSurfaces)"><span /></button>
                </div>

                <div class="en-settings-row en-settings-row-stacked en-settings-row-compact">
                  <icon-rail-layout-settings />
                </div>
              </section>
            </template>

            <template v-else-if="activeSection === 'editor'">
              <section class="en-settings-group">
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Editor footer</strong><span>Show word count, typography controls and the theme shortcut.</span></div><button class="en-switch" type="button" role="switch" aria-label="Show editor footer" :aria-checked="preferences.showEditorFooter" :class="{ active: preferences.showEditorFooter }" @click="setPreference('showEditorFooter', !preferences.showEditorFooter)"><span /></button></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Tag prefix</strong><span>Display # before tag names in the editor.</span></div><button class="en-switch" type="button" role="switch" aria-label="Show tag prefix" :aria-checked="preferences.showTagHashInEditor" :class="{ active: preferences.showTagHashInEditor }" @click="setPreference('showTagHashInEditor', !preferences.showTagHashInEditor)"><span /></button></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Quick insert menu</strong><span>Show the block command menu when its trigger is typed.</span></div><button class="en-switch" type="button" role="switch" aria-label="Show quick insert menu" :aria-checked="!preferences.hideQuickInsertHint" :class="{ active: !preferences.hideQuickInsertHint }" @click="setPreference('hideQuickInsertHint', !preferences.hideQuickInsertHint)"><span /></button></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Quick insert trigger</strong><span>The character that opens the insert menu. The default is /.</span></div><input class="en-compact-input en-trigger-input" :value="preferences.quickInsertTrigger" maxlength="1" aria-label="Quick insert trigger" @change="setQuickInsertTrigger($event.target.value)"></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Pair brackets</strong><span>Automatically insert the matching closing bracket.</span></div><button class="en-switch" type="button" role="switch" aria-label="Automatically pair brackets" :aria-checked="preferences.autoPairBracket" :class="{ active: preferences.autoPairBracket }" @click="setPreference('autoPairBracket', !preferences.autoPairBracket)"><span /></button></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Pair Markdown syntax</strong><span>Automatically close Markdown emphasis and formatting markers.</span></div><button class="en-switch" type="button" role="switch" aria-label="Automatically pair Markdown syntax" :aria-checked="preferences.autoPairMarkdownSyntax" :class="{ active: preferences.autoPairMarkdownSyntax }" @click="setPreference('autoPairMarkdownSyntax', !preferences.autoPairMarkdownSyntax)"><span /></button></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Pair quotes</strong><span>Automatically insert the matching closing quote.</span></div><button class="en-switch" type="button" role="switch" aria-label="Automatically pair quotes" :aria-checked="preferences.autoPairQuote" :class="{ active: preferences.autoPairQuote }" @click="setPreference('autoPairQuote', !preferences.autoPairQuote)"><span /></button></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Spellchecker</strong><span>Check spelling while writing.</span></div><button class="en-switch" type="button" role="switch" aria-label="Enable spellchecker" :aria-checked="preferences.spellcheckerEnabled" :class="{ active: preferences.spellcheckerEnabled }" @click="setPreference('spellcheckerEnabled', !preferences.spellcheckerEnabled)"><span /></button></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Code block line numbers</strong><span>Display line numbers in fenced code blocks.</span></div><button class="en-switch" type="button" role="switch" aria-label="Show code block line numbers" :aria-checked="preferences.codeBlockLineNumbers" :class="{ active: preferences.codeBlockLineNumbers }" @click="setPreference('codeBlockLineNumbers', !preferences.codeBlockLineNumbers)"><span /></button></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Note margins</strong><span>Horizontal space around the title and text.</span></div><label class="en-range-control"><input type="range" min="8" max="48" step="4" :value="preferences.noteEditorMargin" @input="setNoteEditorMargin(Number($event.target.value))"><output>{{ preferences.noteEditorMargin }} px</output></label></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Autosave</strong><span>Write changes to disk automatically.</span></div><button class="en-switch" type="button" role="switch" aria-label="Enable autosave" :aria-checked="preferences.autoSave" :class="{ active: preferences.autoSave }" @click="setPreference('autoSave', !preferences.autoSave)"><span /></button></div>
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Autosave delay</strong><span>How long ElephantNote waits after the last edit.</span></div><select class="en-compact-select" :disabled="!preferences.autoSave" :value="preferences.autoSaveDelay" @change="setPreference('autoSaveDelay', Number($event.target.value))"><option :value="250">Instant · 250 ms</option><option :value="500">Fast · 500 ms</option><option :value="1000">Balanced · 1 s</option><option :value="2000">Relaxed · 2 s</option><option :value="5000">Battery saver · 5 s</option></select></div>
              </section>
            </template>

            <template v-else-if="activeSection === 'vaults'">
              <section class="en-settings-group">
                <div class="en-settings-row"><div class="en-settings-row-copy"><strong>Active vault</strong><span>{{ activeVaultPath || 'No vault is currently open.' }}</span></div><span class="en-status-badge active"><HardDrive aria-hidden="true" />{{ activeVaultName }}</span></div>
                <div v-if="vaults.length" class="en-vault-list"><article v-for="vault in vaults" :key="vault.id" class="en-vault-row"><span class="en-vault-icon"><FolderOpen aria-hidden="true" /></span><div><strong>{{ vault.name }}</strong><p>{{ vault.path }}</p></div><button type="button" class="en-danger-button" :disabled="removingVaultId === vault.id" @click="removeVaultFromApp(vault)">{{ removingVaultId === vault.id ? 'Removing…' : 'Remove' }}</button></article></div>
                <div v-else class="en-settings-empty-state"><FolderOpen aria-hidden="true" /><strong>No vault registered</strong><span>Open a folder from the main workspace to add it.</span></div>
                <p v-if="vaultMessage" class="en-settings-feedback">{{ vaultMessage }}</p>
              </section>
            </template>

            <template v-else-if="activeSection === 'addons'"><addons-settings-panel /></template>
            <template v-else>
              <div class="en-addon-settings-page-anchor" />
              <div v-if="!sectionById[activeSection]" class="en-settings-empty-state">
                <Package aria-hidden="true" />
                <strong>{{ activeSectionMeta.label }} is unavailable</strong>
                <span>The addon is being reloaded or has been disabled. Elephant keeps this page selected instead of moving you to another menu.</span>
              </div>
            </template>
          </template>
        </main>
      </div>
    </section>
  </div>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import log from '@/platform/runtimeLogShim'
import { CalendarDays, Check, ChevronDown, ChevronRight, Cloud, Database, Download, FolderOpen, Globe2, HardDrive, Moon, Package, Palette, PenLine, Search, Sparkles, SunMedium, X } from '@lucide/vue'
import { usePreferencesStore } from '@/store/preferences'
import { useAddonsStore } from '@/store/addons'
import { ELEPHANTNOTE_THEME_FAMILIES, getThemeFamily, getThemeLabel, getThemeMode, getThemeTokens, getThemeVariant } from 'common/elephantnote/appearance'
import AddonsSettingsPanel from './AddonsSettingsPanel.vue'
import IconRailLayoutSettings from './IconRailLayoutSettings.vue'
import LanguageSettingsRow from './LanguageSettingsRow.vue'
import { useVaultStore } from '../../stores/vaultStore'

const props = defineProps({
  theme: { type: String, required: true },
  vaults: { type: Array, default: () => [] },
  activeVaultName: { type: String, default: 'No vault' },
  activeVaultPath: { type: String, default: '' },
  initialSection: { type: String, default: 'appearance' }
})
const emit = defineEmits(['close', 'update-theme'])

const LAST_SETTINGS_SECTION_KEY = 'elephantnote:lastSettingsSection'
const CORE_SECTIONS = Object.freeze([
  { id: 'appearance', label: 'Appearance', icon: Palette },
  { id: 'editor', label: 'Editor', icon: PenLine },
  { id: 'vaults', label: 'Vaults', icon: FolderOpen },
  { id: 'addons', label: 'Addons', icon: Package }
])
const ICONS = Object.freeze({
  calendar: CalendarDays,
  cloud: Cloud,
  database: Database,
  download: Download,
  globe: Globe2,
  package: Package,
  sparkles: Sparkles
})
const CORE_SETTINGS_INDEX = Object.freeze([
  { id: 'appearance-mode', section: 'appearance', label: 'Color mode', description: 'Light and dark appearance.' },
  { id: 'appearance-language', section: 'appearance', label: 'Language', description: 'System, built-in and ISO language packs.' },
  { id: 'appearance-theme', section: 'appearance', label: 'Theme', description: 'Elephant, Apple, Graphite, Nord, Solar, Forest, Beige, Pastel and Gamer Violet themes.' },
  { id: 'appearance-floating-surfaces', section: 'appearance', label: 'Floating surfaces', description: 'Lift navigation, controls and the writing surface above the background.' },
  { id: 'appearance-icon-rail', section: 'appearance', label: 'Vertical icon bar', description: 'Reorder, hide and divide navigation icons.' },
  { id: 'editor-footer', section: 'editor', label: 'Editor footer', description: 'Word count and typography controls.' },
  { id: 'editor-tags', section: 'editor', label: 'Tag prefix', description: 'Show or hide the # before tags.' },
  { id: 'editor-quick-insert', section: 'editor', label: 'Quick insert menu', description: 'Show block commands when typing the trigger.' },
  { id: 'editor-quick-trigger', section: 'editor', label: 'Quick insert trigger', description: 'Change the / command trigger.' },
  { id: 'editor-brackets', section: 'editor', label: 'Pair brackets', description: 'Automatically close brackets.' },
  { id: 'editor-markdown', section: 'editor', label: 'Pair Markdown syntax', description: 'Automatically close Markdown markers.' },
  { id: 'editor-quotes', section: 'editor', label: 'Pair quotes', description: 'Automatically close quotation marks.' },
  { id: 'editor-spellchecker', section: 'editor', label: 'Spellchecker', description: 'Check spelling while writing.' },
  { id: 'editor-code-lines', section: 'editor', label: 'Code block line numbers', description: 'Show line numbers in fenced code blocks.' },
  { id: 'editor-margin', section: 'editor', label: 'Note margins', description: 'Horizontal writing space.' },
  { id: 'editor-autosave', section: 'editor', label: 'Autosave', description: 'Automatically persist changes to disk.' },
  { id: 'editor-autosave-delay', section: 'editor', label: 'Autosave delay', description: 'Delay before writing the latest edit.' },
  { id: 'vault-active', section: 'vaults', label: 'Active vault', description: 'Current local workspace folder.' },
  { id: 'vault-open', section: 'vaults', label: 'Open vaults', description: 'Review or remove registered vaults.' },
  { id: 'addons-installed', section: 'addons', label: 'Installed addons', description: 'Installed addon packages.' },
  { id: 'addons-available', section: 'addons', label: 'Available addons', description: 'Install optional features and community packages.' },
  { id: 'addons-community', section: 'addons', label: 'Community addons', description: 'Third-party addon activation.' },
  { id: 'addons-packs', section: 'addons', label: 'Addon packs', description: 'Install or share complete addon configurations.' }
])

const addonsStore = useAddonsStore()
const addonSettingsContributions = computed(() => addonsStore.getContributions('settings.sections'))
const addonStandaloneSections = computed(() => {
  const unique = new Map()
  for (const entry of addonSettingsContributions.value) {
    const contribution = entry?.contribution || {}
    if (!contribution.standalone || !contribution.section || unique.has(contribution.section)) continue
    unique.set(contribution.section, {
      id: contribution.section,
      label: contribution.navigationLabel || contribution.title || contribution.section,
      icon: ICONS[contribution.navigationIcon] || Package,
      order: Number.isFinite(contribution.order) ? contribution.order : 1000
    })
  }
  return [...unique.values()].sort((a, b) => a.order - b.order || a.label.localeCompare(b.label))
})
const sections = computed(() => [...CORE_SECTIONS, ...addonStandaloneSections.value])
const sectionById = computed(() => Object.fromEntries(sections.value.map((section) => [section.id, section])))
const rememberedSectionMeta = ref({})
const normalizeSectionId = (section) => String(section || '').trim()
const normalizeSection = (section, { preserveUnknown = false } = {}) => {
  const candidate = normalizeSectionId(section)
  if (!candidate) return 'appearance'
  if (sectionById.value[candidate] || rememberedSectionMeta.value[candidate]) return candidate
  return preserveUnknown && /^[a-z0-9._-]+$/i.test(candidate) ? candidate : 'appearance'
}
const settingsIndex = computed(() => [
  ...CORE_SETTINGS_INDEX,
  ...addonSettingsContributions.value.map((entry) => {
    const contribution = entry.contribution || {}
    const section = contribution.section || 'addons'
    return {
      id: contribution.id || `${entry.addonId}:${section}`,
      section,
      label: contribution.navigationLabel || contribution.title || entry.addonId,
      description: contribution.description || 'Addon setting.'
    }
  })
].map((entry) => ({
  ...entry,
  sectionLabel: sectionById.value[entry.section]?.label || rememberedSectionMeta.value[entry.section]?.label || entry.section,
  icon: sectionById.value[entry.section]?.icon || rememberedSectionMeta.value[entry.section]?.icon || Package
})))

const storedInitialSection = window.localStorage.getItem(LAST_SETTINGS_SECTION_KEY)
const requestedInitialSection = props.initialSection && props.initialSection !== 'appearance'
  ? props.initialSection
  : storedInitialSection || props.initialSection
const activeSection = ref(normalizeSection(requestedInitialSection, { preserveUnknown: true }))
const settingsQuery = ref('')
const searchInput = ref(null)
const settingsContent = ref(null)
const themeExpanded = ref(true)
const isMacOS = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(`${navigator.platform || ''} ${navigator.userAgent || ''}`)
const activeSectionMeta = computed(() => sectionById.value[activeSection.value] ||
  rememberedSectionMeta.value[activeSection.value] || {
    id: activeSection.value,
    label: activeSection.value
      .split(/[._-]+/)
      .filter(Boolean)
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ') || 'Addon settings',
    icon: Package
  })
const searchResults = computed(() => {
  const terms = settingsQuery.value.toLocaleLowerCase().trim().split(/\s+/).filter(Boolean)
  if (!terms.length) return []
  return settingsIndex.value.filter((entry) => {
    const haystack = `${entry.label} ${entry.description} ${entry.sectionLabel}`.toLocaleLowerCase()
    return terms.every((term) => haystack.includes(term))
  })
})

const rememberSection = (section) => {
  const normalized = normalizeSectionId(section)
  if (normalized) window.localStorage.setItem(LAST_SETTINGS_SECTION_KEY, normalized)
}

watch(sections, (currentSections) => {
  const next = { ...rememberedSectionMeta.value }
  for (const section of currentSections) next[section.id] = section
  rememberedSectionMeta.value = next
}, { immediate: true })

watch(() => props.initialSection, (section) => {
  const nextSection = normalizeSection(section, { preserveUnknown: true })
  if (activeSection.value !== nextSection) {
    activeSection.value = nextSection
    rememberSection(nextSection)
    settingsQuery.value = ''
    log.info('[settings] initial-section:changed', { section: nextSection })
    scrollContentToTop()
  }
})

const vaults = computed(() => props.vaults)
const theme = computed(() => props.theme)
const themeFamilies = ELEPHANTNOTE_THEME_FAMILIES
const themeMode = computed(() => getThemeMode(theme.value))
const themeClassId = computed(() => theme.value.replace(/[^a-z0-9-]/gi, '-'))
const activeThemeFamily = computed(() => getThemeFamily(theme.value))
const activeThemeLabel = computed(() => getThemeLabel(theme.value))
const settingsStyle = computed(() => {
  const tokens = getThemeTokens(theme.value)
  return { ...tokens, '--en-card': tokens['--en-soft'], '--en-accent': tokens['--en-primary'], '--en-active-bg': tokens['--selectionColor'], '--en-active-border': tokens['--en-primary'], '--en-active-text': tokens['--en-primary'] }
})

const preferences = usePreferencesStore()
const vaultStore = useVaultStore()
const vaultMessage = ref('')
const removingVaultId = ref('')

const scrollContentToTop = () => nextTick(() => settingsContent.value?.scrollTo({ top: 0, behavior: 'instant' }))
const selectSection = (section) => {
  activeSection.value = section
  rememberSection(section)
  settingsQuery.value = ''
  log.info('[settings] section:selected', { section })
  scrollContentToTop()
}
const openSearchResult = (result) => {
  activeSection.value = result.section
  rememberSection(result.section)
  settingsQuery.value = ''
  log.info('[settings] search-result:opened', { id: result.id, section: result.section })
  scrollContentToTop()
}
const setPreference = (type, value) => preferences.SET_SINGLE_PREFERENCE({ type, value })
const setQuickInsertTrigger = (value) => setPreference('quickInsertTrigger', String(value || '/').slice(0, 1))
const setNoteEditorMargin = (value) => setPreference('noteEditorMargin', Math.max(8, Math.min(48, Number(value) || 12)))

const removeVaultFromApp = async (vault) => {
  if (!vault?.id || !window.confirm(`Remove "${vault.name}" from ElephantNote? The folder stays on disk.`)) return
  removingVaultId.value = vault.id
  vaultMessage.value = ''
  try {
    await vaultStore.removeVault(vault.id)
    vaultMessage.value = `Removed ${vault.name} from ElephantNote. The folder still exists.`
  } catch (error) {
    log.error('[settings] vault-remove:failed', error)
    vaultMessage.value = error instanceof Error ? error.message : 'Unable to remove vault.'
  } finally { removingVaultId.value = '' }
}

const handleKeyboard = (event) => {
  if (event.key === 'Escape') emit('close')
  if ((event.metaKey || event.ctrlKey) && event.key.toLocaleLowerCase() === 'f') { event.preventDefault(); searchInput.value?.focus(); searchInput.value?.select() }
}

onMounted(() => {
  rememberSection(activeSection.value)
  window.addEventListener('keydown', handleKeyboard)
  log.info('[settings] mounted', { sections: sections.value.map((section) => section.id), activeSection: activeSection.value, theme: activeThemeLabel.value })
})
onBeforeUnmount(() => window.removeEventListener('keydown', handleKeyboard))
</script>

<style scoped src="./settings-redesign.css"></style>
