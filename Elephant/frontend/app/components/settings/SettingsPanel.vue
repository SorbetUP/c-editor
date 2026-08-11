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

                <div class="en-settings-row en-settings-row-stacked en-settings-row-compact">
                  <icon-rail-layout-settings />
                </div>

                <div class="en-settings-row">
                  <div class="en-settings-row-copy"><strong>Floating surfaces</strong><span>Lift navigation, controls and the writing surface above the background.</span></div>
                  <button class="en-switch" type="button" role="switch" aria-label="Floating surfaces" :aria-checked="preferences.floatingSurfaces" :class="{ active: preferences.floatingSurfaces }" @click="setPreference('floatingSurfaces', !preferences.floatingSurfaces)"><span /></button>
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
              <section class="en-settings-group" aria-labelledby="vaults-section-title">
                <h2 id="vaults-section-title" class="sr-only">Vaults</h2>

                <div v-if="vaultLoading && !vaults.length" class="en-settings-empty-state en-vault-state" role="status" aria-live="polite">
                  <LoaderCircle class="en-vault-loading-icon" aria-hidden="true" />
                  <strong>Loading vaults</strong>
                  <span>Reading the registered workspaces.</span>
                </div>
                <div v-else-if="vaultError" class="en-settings-empty-state en-vault-state en-vault-state-error" role="alert">
                  <CircleAlert aria-hidden="true" />
                  <strong>Vaults could not be loaded</strong>
                  <span>{{ vaultError }}</span>
                  <button class="secondary compact" type="button" :disabled="vaultLoading" @click="retryVaultLoad">
                    <RotateCw aria-hidden="true" />{{ vaultLoading ? 'Retrying…' : 'Retry' }}
                  </button>
                </div>
                <div v-else-if="vaults.length" class="en-vault-list">
                  <article
                    v-for="vault in vaults"
                    :key="vault.id"
                    class="en-vault-row en-vault-row-detailed"
                    :class="{ 'is-active': isActiveVault(vault), 'is-disabled': vault.enabled === false, 'is-missing': vaultStatus(vault) === 'missing' }"
                    :aria-label="vaultAriaLabel(vault)"
                  >
                    <button
                      class="en-vault-icon en-vault-icon-button"
                      type="button"
                      :aria-label="`Change icon for ${vault.name}`"
                      title="Change vault icon"
                      @click="toggleVaultIconEditor(vault.id)"
                    >
                      <component :is="getVaultIconComponent(vault)" aria-hidden="true" />
                    </button>
                    <div class="en-vault-main">
                      <div class="en-vault-name-line">
                        <form v-if="editingVaultId === vault.id" class="en-vault-name-form" @submit.prevent="saveVaultName(vault)">
                          <input
                            v-model="draftVaultName"
                            class="en-compact-input"
                            type="text"
                            maxlength="120"
                            :aria-label="`Vault name for ${vault.name}`"
                            @keyup.esc="cancelVaultNameEdit"
                          >
                          <button class="primary compact" type="submit" :disabled="savingVaultAction === `name:${vault.id}`">
                            {{ savingVaultAction === `name:${vault.id}` ? 'Saving…' : 'Save' }}
                          </button>
                          <button class="secondary compact" type="button" @click="cancelVaultNameEdit">Cancel</button>
                        </form>
                        <template v-else>
                          <strong>{{ vault.name }}</strong>
                          <button class="en-vault-icon-action" type="button" :aria-label="`Edit name for ${vault.name}`" title="Edit vault name" @click="startVaultNameEdit(vault)">
                            <Pencil aria-hidden="true" />
                          </button>
                        </template>
                      </div>
                      <div class="en-vault-path-line">
                        <p class="en-vault-path" :title="vault.path">{{ vault.path || 'Path unavailable' }}</p>
                        <button v-if="vault.path" class="en-vault-icon-action" type="button" :aria-label="`Copy path for ${vault.name}`" title="Copy path" @click="copyVaultPath(vault)">
                          <Copy aria-hidden="true" />
                        </button>
                      </div>
                      <p v-if="vault.lastOpenedAt" class="en-vault-meta">Last opened {{ formatLastOpenedAt(vault.lastOpenedAt) }}</p>
                      <p v-else class="en-vault-meta">Last opened: not available</p>
                      <div v-if="editingIconVaultId === vault.id" class="en-vault-icon-picker" role="group" :aria-label="`Choose icon for ${vault.name}`">
                        <button class="en-vault-icon-choice" :class="{ active: !normalizeVaultIcon(vault.icon) }" type="button" aria-label="Use default vault icon" title="Use default vault icon" :disabled="savingVaultAction === `icon:${vault.id}`" @click="setVaultIconFromSettings(vault, '')">
                          <Vault aria-hidden="true" />
                        </button>
                        <button
                          v-for="icon in vaultIconOptions"
                          :key="icon.name"
                          class="en-vault-icon-choice"
                          :class="{ active: normalizeVaultIcon(vault.icon) === icon.name }"
                          type="button"
                          :aria-label="`Use ${icon.label} icon`"
                          :title="icon.label"
                          :disabled="savingVaultAction === `icon:${vault.id}`"
                          @click="setVaultIconFromSettings(vault, icon.name)"
                        >
                          <component :is="icon.component" aria-hidden="true" />
                        </button>
                      </div>
                    </div>
                    <div class="en-vault-actions">
                      <button
                        v-if="vault.enabled !== false && !isActiveVault(vault)"
                        class="en-vault-icon-action"
                        type="button"
                        :aria-label="`Activate ${vault.name}`"
                        title="Activate vault"
                        @click="activateVault(vault)"
                      >
                        <Power aria-hidden="true" />
                      </button>
                      <button
                        class="en-vault-enable-toggle"
                        type="button"
                        role="switch"
                        :aria-checked="vault.enabled !== false"
                        :aria-label="`${vault.enabled === false ? 'Enable' : 'Disable'} ${vault.name}`"
                        :title="vault.enabled === false ? 'Enable vault' : 'Disable vault'"
                        @click="toggleVaultEnabled(vault)"
                      >
                        <PowerOff v-if="vault.enabled === false" aria-hidden="true" />
                        <Power v-else aria-hidden="true" />
                      </button>
                      <button
                        class="en-vault-icon-action en-vault-remove-action"
                        type="button"
                        :disabled="removingVaultId === vault.id"
                        :aria-label="`Remove ${vault.name} from list`"
                        title="Remove from list"
                        @click="removeVaultFromApp(vault)"
                      >
                        <Trash2 aria-hidden="true" />
                      </button>
                    </div>
                  </article>
                </div>
                <div v-else class="en-settings-empty-state en-vault-state">
                  <FolderOpen aria-hidden="true" />
                  <strong>No vault registered</strong>
                  <span>Open a folder from the main workspace to add it. Existing folders stay on disk when removed from this list.</span>
                  <button class="primary compact" type="button" :disabled="vaultLoading" @click="chooseVaultFromSettings">
                    <Plus aria-hidden="true" />Open a folder
                  </button>
                </div>
                <p v-if="vaultMessage" class="en-settings-feedback" :class="{ 'is-error': vaultMessageIsError }" role="status" aria-live="polite">{{ vaultMessage }}</p>
                <section class="en-vault-trash" aria-label="Vault trash">
                  <div class="en-vault-trash-summary">
                    <div class="en-vault-trash-status" role="status" aria-live="polite">
                      <Trash2 aria-hidden="true" />
                      <strong v-if="trashLoading">Reading trash…</strong>
                      <strong v-else-if="trashError">Trash unavailable</strong>
                      <strong v-else-if="trashItems.length">{{ trashItems.length }} deleted {{ trashItems.length === 1 ? 'item' : 'items' }}</strong>
                      <strong v-else>Trash is empty</strong>
                    </div>
                    <button
                      v-if="trashItems.length"
                      data-testid="vault-trash-toggle"
                      class="en-vault-trash-toggle"
                      type="button"
                      :aria-expanded="trashExpanded"
                      :aria-label="trashExpanded ? 'Collapse trash' : 'Expand trash'"
                      :title="trashExpanded ? 'Collapse trash' : 'Expand trash'"
                      @click="toggleTrash"
                    >
                      <ChevronDown :class="{ expanded: trashExpanded }" aria-hidden="true" />
                    </button>
                  </div>
                  <div v-if="trashError" class="en-vault-trash-state is-error" role="alert">
                    <CircleAlert aria-hidden="true" />{{ trashError }}
                    <button class="secondary compact" type="button" @click="loadVaultTrash">Retry</button>
                  </div>
                  <div v-if="trashExpanded && trashItems.length" class="en-vault-trash-body">
                    <div class="en-vault-trash-list">
                      <article v-for="item in trashItems" :key="item.trashPath" class="en-vault-trash-item">
                        <div><strong>{{ item.name || item.originalPath }}</strong><small>{{ item.originalPath }}</small></div>
                        <button class="secondary compact" type="button" :disabled="trashAction === item.trashPath" @click="restoreVaultTrash(item)"><ArchiveRestore aria-hidden="true" />{{ trashAction === item.trashPath ? 'Restoring…' : 'Restore' }}</button>
                      </article>
                      <button class="en-danger-button" type="button" :disabled="trashAction === 'empty'" @click="emptyVaultTrash"><Trash2 aria-hidden="true" />{{ trashAction === 'empty' ? 'Emptying…' : 'Empty trash' }}</button>
                    </div>
                  </div>
                </section>
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
import { ArchiveRestore, CalendarDays, Check, ChevronDown, ChevronRight, CircleAlert, Cloud, Copy, Database, Download, FileText, FolderOpen, Globe2, GraduationCap, Home, Landmark, LoaderCircle, Moon, Package, Palette, PenLine, Pencil, Plus, Power, PowerOff, Rocket, RotateCw, Search, Sparkles, SunMedium, Terminal, Trash2, Vault, Workflow, X } from '@lucide/vue'
import { usePreferencesStore } from '@/store/preferences'
import { useAddonsStore } from '@/store/addons'
import { ELEPHANTNOTE_THEME_FAMILIES, getThemeFamily, getThemeLabel, getThemeMode, getThemeTokens, getThemeVariant, normalizeVaultIcon, VAULT_ICON_OPTIONS } from 'common/elephantnote/appearance'
import AddonsSettingsPanel from './AddonsSettingsPanel.vue'
import IconRailLayoutSettings from './IconRailLayoutSettings.vue'
import LanguageSettingsRow from './LanguageSettingsRow.vue'
import { useVaultStore } from '../../stores/vaultStore'
import { elephantnoteClient } from '../../services/elephantnoteClient'

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
  { id: 'appearance-icon-rail', section: 'appearance', label: 'Vertical icon bar', description: 'Reorder, hide and divide navigation icons.' },
  { id: 'appearance-floating-surfaces', section: 'appearance', label: 'Floating surfaces', description: 'Lift navigation, controls and the writing surface above the background.' },
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
const activeVaultPath = computed(() => props.activeVaultPath)
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
const vaultMessageIsError = ref(false)
const removingVaultId = ref('')
const editingVaultId = ref('')
const draftVaultName = ref('')
const editingIconVaultId = ref('')
const savingVaultAction = ref('')
const vaultPathStatuses = ref({})
const trashExpanded = ref(false)
const trashItems = ref([])
const trashLoading = ref(false)
const trashError = ref('')
const trashAction = ref('')
let vaultPathStatusRequest = 0

const VAULT_ICON_COMPONENTS = { Database, FileText, GraduationCap, Home, Landmark, Rocket, Terminal, Workflow }
const vaultIconOptions = VAULT_ICON_OPTIONS
  .map((option) => ({ ...option, component: VAULT_ICON_COMPONENTS[option.lucide] }))
  .filter((option) => option.component)
const vaultIconComponentsByName = Object.fromEntries(vaultIconOptions.map((option) => [option.name, option.component]))
const getVaultIconComponent = (vault) => vaultIconComponentsByName[normalizeVaultIcon(vault?.icon)] || Vault

const vaultLoading = computed(() => vaultStore.loading)
const vaultError = computed(() => vaultStore.error)
const isActiveVault = (vault) => vault?.id === vaultStore.activeVaultId || Boolean(vault?.path && vault.path === activeVaultPath.value)
const vaultStatus = (vault) => {
  if (vault?.enabled === false) return 'disabled'
  const pathStatus = vaultPathStatuses.value[vault?.id] || 'unknown'
  if (pathStatus === 'checking') return 'checking'
  if (pathStatus === 'missing') return 'missing'
  if (isActiveVault(vault)) return 'active'
  if (pathStatus === 'available') return 'available'
  return 'unknown'
}
const vaultAriaLabel = (vault) => {
  const name = vault?.name || 'Unnamed vault'
  if (vault?.enabled === false) return `${name}: Disabled`
  if (isActiveVault(vault)) return `${name}: Active`
  return name
}

const setVaultMessage = (message, isError = false) => {
  vaultMessage.value = message
  vaultMessageIsError.value = isError
}

const refreshVaultPathStatuses = async () => {
  const requestId = ++vaultPathStatusRequest
  const currentVaults = vaults.value.filter((vault) => vault?.id && vault?.path)
  vaultPathStatuses.value = Object.fromEntries(currentVaults.map((vault) => [vault.id, 'checking']))
  const stat = window.fileUtils?.stat
  if (typeof stat !== 'function' || window.fileUtils?.__elephantnoteBootstrapFallback) {
    vaultPathStatuses.value = Object.fromEntries(currentVaults.map((vault) => [vault.id, 'unknown']))
    return
  }
  await Promise.all(currentVaults.map(async (vault) => {
    let status = 'missing'
    try {
      const info = await stat.call(window.fileUtils, vault.path)
      status = info?.isDirectory ? 'available' : 'missing'
    } catch (error) {
      log.warn('[settings] vault-path:probe-failed', { vaultId: vault.id, error: error?.message || String(error) })
    }
    if (requestId === vaultPathStatusRequest) {
      vaultPathStatuses.value = { ...vaultPathStatuses.value, [vault.id]: status }
    }
  }))
}

watch(() => vaults.value.map((vault) => `${vault?.id || ''}:${vault?.path || ''}`).join('|'), refreshVaultPathStatuses, { immediate: true })

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

const startVaultNameEdit = (vault) => {
  editingVaultId.value = vault.id
  draftVaultName.value = vault.name || ''
  setVaultMessage('')
}
const cancelVaultNameEdit = () => {
  editingVaultId.value = ''
  draftVaultName.value = ''
}
const saveVaultName = async (vault) => {
  const name = draftVaultName.value.trim()
  if (!name) {
    setVaultMessage('Vault name cannot be empty.', true)
    return
  }
  savingVaultAction.value = `name:${vault.id}`
  setVaultMessage('')
  log.info('[settings] vault-name:update:start', { vaultId: vault.id })
  try {
    await vaultStore.setVaultName(vault.id, name)
    cancelVaultNameEdit()
    setVaultMessage(`Renamed vault to ${name}.`)
    log.info('[settings] vault-name:update:done', { vaultId: vault.id })
  } catch (error) {
    log.error('[settings] vault-name:update:failed', { vaultId: vault.id, error: error?.message || String(error) })
    setVaultMessage(error instanceof Error ? error.message : 'Unable to rename vault.', true)
  } finally {
    savingVaultAction.value = ''
  }
}
const toggleVaultIconEditor = (vaultId) => {
  editingIconVaultId.value = editingIconVaultId.value === vaultId ? '' : vaultId
  setVaultMessage('')
}
const setVaultIconFromSettings = async (vault, icon) => {
  savingVaultAction.value = `icon:${vault.id}`
  setVaultMessage('')
  log.info('[settings] vault-icon:update:start', { vaultId: vault.id, icon: icon || 'default' })
  try {
    await vaultStore.setVaultIcon(vault.id, icon)
    editingIconVaultId.value = ''
    setVaultMessage(`Updated icon for ${vault.name}.`)
    log.info('[settings] vault-icon:update:done', { vaultId: vault.id, icon: icon || 'default' })
  } catch (error) {
    log.error('[settings] vault-icon:update:failed', { vaultId: vault.id, error: error?.message || String(error) })
    setVaultMessage(error instanceof Error ? error.message : 'Unable to update vault icon.', true)
  } finally {
    savingVaultAction.value = ''
  }
}
const activateVault = async (vault) => {
  if (!vault?.id || vault.enabled === false || isActiveVault(vault)) return
  setVaultMessage('')
  savingVaultAction.value = `active:${vault.id}`
  log.info('[settings] vault-activate:start', { vaultId: vault.id })
  try {
    await vaultStore.setActiveVault(vault.id)
    if (vaultStore.error) throw new Error(vaultStore.error)
    setVaultMessage(`Activated ${vault.name}.`)
    log.info('[settings] vault-activate:done', { vaultId: vault.id })
  } catch (error) {
    log.error('[settings] vault-activate:failed', { vaultId: vault.id, error: error?.message || String(error) })
    setVaultMessage(error instanceof Error ? error.message : 'Unable to activate vault.', true)
  } finally {
    savingVaultAction.value = ''
  }
}
const toggleVaultEnabled = async (vault) => {
  if (!vault?.id) return
  const enabled = vault.enabled === false
  savingVaultAction.value = `enabled:${vault.id}`
  setVaultMessage('')
  log.info('[settings] vault-enabled:update:start', { vaultId: vault.id, enabled })
  try {
    await vaultStore.setVaultEnabled(vault.id, enabled)
    setVaultMessage(`${enabled ? 'Enabled' : 'Disabled'} ${vault.name}.`)
    log.info('[settings] vault-enabled:update:done', { vaultId: vault.id, enabled })
  } catch (error) {
    log.error('[settings] vault-enabled:update:failed', { vaultId: vault.id, enabled, error: error?.message || String(error) })
    setVaultMessage(error instanceof Error ? error.message : 'Unable to update vault state.', true)
  } finally {
    savingVaultAction.value = ''
  }
}
const formatLastOpenedAt = (value) => {
  const numericValue = Number(value)
  const date = Number.isFinite(numericValue) && numericValue > 0
    ? new Date(numericValue < 10_000_000_000 ? numericValue * 1000 : numericValue)
    : new Date(value)
  if (Number.isNaN(date.getTime())) return 'not available'
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(date)
}
const copyVaultPath = async (vault) => {
  const writeText = window.navigator?.clipboard?.writeText
  if (typeof writeText !== 'function') {
    setVaultMessage('Copy is unavailable in this runtime. Select the path manually.', true)
    return
  }
  log.info('[settings] vault-path:copy:start', { vaultId: vault.id })
  try {
    await writeText.call(window.navigator.clipboard, vault.path)
    setVaultMessage(`Copied path for ${vault.name}.`)
    log.info('[settings] vault-path:copy:done', { vaultId: vault.id })
  } catch (error) {
    log.error('[settings] vault-path:copy:failed', { vaultId: vault.id, error: error?.message || String(error) })
    setVaultMessage('Unable to copy the path. Select it manually.', true)
  }
}
const chooseVaultFromSettings = async () => {
  setVaultMessage('')
  const chosen = await vaultStore.chooseVault()
  if (!chosen && vaultStore.error) setVaultMessage(vaultStore.error, true)
}
const retryVaultLoad = async () => {
  await vaultStore.load()
  if (vaultStore.error) setVaultMessage(vaultStore.error, true)
}

const loadVaultTrash = async () => {
  if (!activeVaultPath.value) {
    trashItems.value = []
    trashError.value = ''
    trashLoading.value = false
    return
  }
  trashLoading.value = true
  trashError.value = ''
  log.info('[settings] vault-trash:list:start', { hasActiveVault: Boolean(activeVaultPath.value) })
  try {
    const result = await elephantnoteClient.vaults.trash.list()
    trashItems.value = Array.isArray(result) ? result : []
    log.info('[settings] vault-trash:list:done', { count: trashItems.value.length })
  } catch (error) {
    trashError.value = error instanceof Error ? error.message : 'Unable to read vault trash.'
    log.error('[settings] vault-trash:list:failed', { error: trashError.value })
  } finally {
    trashLoading.value = false
  }
}
watch([activeSection, activeVaultPath], ([section]) => {
  trashExpanded.value = false
  if (section === 'vaults') loadVaultTrash()
}, { immediate: true })
const toggleTrash = async () => {
  trashExpanded.value = !trashExpanded.value
  if (trashExpanded.value) await loadVaultTrash()
}
const restoreVaultTrash = async (item) => {
  if (!item?.trashPath) return
  trashAction.value = item.trashPath
  trashError.value = ''
  log.info('[settings] vault-trash:restore:start', { trashPath: item.trashPath })
  try {
    await elephantnoteClient.vaults.trash.restore(item.trashPath)
    await loadVaultTrash()
    await vaultStore.load()
    setVaultMessage(`Restored ${item.name || item.originalPath}.`)
    log.info('[settings] vault-trash:restore:done', { trashPath: item.trashPath })
  } catch (error) {
    trashError.value = error instanceof Error ? error.message : 'Unable to restore the deleted entry.'
    log.error('[settings] vault-trash:restore:failed', { trashPath: item.trashPath, error: trashError.value })
  } finally {
    trashAction.value = ''
  }
}
const emptyVaultTrash = async () => {
  if (!window.confirm('Permanently delete every entry in this vault trash?')) return
  trashAction.value = 'empty'
  trashError.value = ''
  log.info('[settings] vault-trash:empty:start', { hasActiveVault: Boolean(activeVaultPath.value) })
  try {
    await elephantnoteClient.vaults.trash.empty()
    await loadVaultTrash()
    setVaultMessage('Vault trash emptied permanently.')
    log.info('[settings] vault-trash:empty:done', { hasActiveVault: Boolean(activeVaultPath.value) })
  } catch (error) {
    trashError.value = error instanceof Error ? error.message : 'Unable to empty vault trash.'
    log.error('[settings] vault-trash:empty:failed', { error: trashError.value })
  } finally {
    trashAction.value = ''
  }
}

const removeVaultFromApp = async (vault) => {
  if (!vault?.id || !window.confirm(`Remove "${vault.name}" from the ElephantNote list? Its folder and notes stay on disk.`)) return
  removingVaultId.value = vault.id
  setVaultMessage('')
  log.info('[settings] vault-remove:start', { vaultId: vault.id })
  try {
    await vaultStore.removeVault(vault.id)
    setVaultMessage(`Removed ${vault.name} from the list. The folder still exists.`)
    log.info('[settings] vault-remove:done', { vaultId: vault.id })
  } catch (error) {
    log.error('[settings] vault-remove:failed', { vaultId: vault.id, error: error?.message || String(error) })
    setVaultMessage(error instanceof Error ? error.message : 'Unable to remove vault.', true)
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

<style scoped>
.en-vault-row-detailed { grid-template-columns: 32px minmax(0, 1fr) auto; align-items: start; }
.en-vault-row-detailed.is-active { background: color-mix(in srgb, var(--en-primary, #2563eb) 5%, transparent); }
.en-vault-row-detailed.is-missing { background: color-mix(in srgb, var(--en-danger, #b42318) 4%, transparent); }
.en-vault-row-detailed.is-disabled { opacity: .62; }
.en-vault-icon { display: grid; place-items: center; width: 32px; height: 32px; border-radius: 9px; background: var(--en-soft, #e9eff7); color: var(--en-primary, #2563eb); }
.en-vault-icon-button { padding: 0; border: 1px solid transparent; cursor: pointer; }
.en-vault-icon-button:hover, .en-vault-icon-button:focus-visible { border-color: var(--en-primary, #2563eb); background: color-mix(in srgb, var(--en-primary, #2563eb) 10%, var(--en-soft, #e9eff7)); }
.en-vault-icon svg { width: 17px; height: 17px; }
.en-vault-main { min-width: 0; display: grid; gap: 5px; }
.en-vault-name-line, .en-vault-path-line { min-width: 0; display: flex; align-items: center; gap: 7px; }
.en-vault-name-line > strong { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.en-vault-name-form { min-width: 0; display: flex; align-items: center; flex-wrap: wrap; gap: 6px; }
.en-vault-name-form input { min-width: 150px; max-width: 260px; }
.en-vault-icon-action { flex: 0 0 auto; width: 25px; height: 25px; display: inline-grid; place-items: center; padding: 0; border: 1px solid transparent; border-radius: 7px; background: transparent; color: var(--en-muted, #667085); cursor: pointer; }
.en-vault-icon-action:hover, .en-vault-icon-action:focus-visible { border-color: var(--en-border, #c5cfdd); background: var(--en-soft, #e9eff7); color: var(--en-text, #101828); }
.en-vault-icon-action:disabled, .en-vault-enable-toggle:disabled { opacity: .45; cursor: wait; }
.en-vault-icon-action svg { width: 13px; height: 13px; }
.en-vault-path { flex: 1 1 auto; min-width: 0; margin: 0; overflow: hidden; color: var(--en-muted, #667085); font: 10.5px/1.4 ui-monospace, SFMono-Regular, Menlo, monospace; text-overflow: ellipsis; white-space: nowrap; }
.en-vault-meta { margin: 0; color: var(--en-muted, #667085); font-size: 10px; }
.en-vault-actions { display: flex; align-items: center; justify-content: flex-end; flex-wrap: wrap; gap: 7px; }
.en-vault-actions button svg { width: 13px; height: 13px; }
.en-vault-enable-toggle { width: 26px; height: 26px; display: inline-grid; place-items: center; padding: 0; border: 1px solid transparent; border-radius: 7px; background: transparent; color: var(--en-primary, #2563eb); cursor: pointer; }
.en-vault-enable-toggle[aria-checked="false"] { color: var(--en-muted, #667085); }
.en-vault-enable-toggle:hover, .en-vault-enable-toggle:focus-visible { border-color: var(--en-border, #c5cfdd); background: var(--en-soft, #e9eff7); }
.en-vault-remove-action:hover, .en-vault-remove-action:focus-visible { border-color: color-mix(in srgb, var(--en-danger, #b42318) 40%, transparent); background: color-mix(in srgb, var(--en-danger, #b42318) 10%, transparent); color: var(--en-danger, #b42318); }
.en-vault-icon-picker { display: flex; align-items: center; flex-wrap: wrap; gap: 5px; padding-top: 3px; }
.en-vault-icon-choice { width: 28px; height: 28px; display: inline-grid; place-items: center; padding: 0; border: 1px solid var(--en-border, #c5cfdd); border-radius: 7px; background: var(--en-surface, #fff); color: var(--en-muted, #667085); cursor: pointer; }
.en-vault-icon-choice:hover, .en-vault-icon-choice.active { border-color: var(--en-primary, #2563eb); background: color-mix(in srgb, var(--en-primary, #2563eb) 10%, var(--en-surface, #fff)); color: var(--en-primary, #2563eb); }
.en-vault-icon-choice:disabled { opacity: .55; cursor: wait; }
.en-vault-icon-choice svg { width: 14px; height: 14px; }
.en-vault-state { min-height: 190px; border-top: 1px solid var(--en-border, #c5cfdd); }
.en-vault-state-error { color: var(--en-danger, #b42318); }
.en-vault-state-error span { color: var(--en-muted, #667085); max-width: 52ch; }
.en-vault-loading-icon { animation: en-vault-spin .9s linear infinite; }
.en-settings-feedback.is-error { color: var(--en-danger, #b42318); }
.en-vault-trash { margin-top: 16px; border: 1px solid var(--en-border, #c5cfdd); border-radius: 12px; overflow: hidden; }
.en-vault-trash-summary { min-height: 58px; display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 11px 14px; background: var(--en-soft, #e9eff7); }
.en-vault-trash-status { min-width: 0; display: flex; align-items: center; gap: 9px; color: var(--en-muted, #667085); }
.en-vault-trash-status svg { width: 20px; height: 20px; flex: 0 0 auto; }
.en-vault-trash-status strong { overflow: hidden; color: var(--en-text, #101828); text-overflow: ellipsis; white-space: nowrap; }
.en-vault-trash-toggle { width: 28px; height: 28px; display: inline-grid; place-items: center; padding: 0; border: 1px solid transparent; border-radius: 7px; background: transparent; color: var(--en-muted, #667085); cursor: pointer; }
.en-vault-trash-toggle:hover, .en-vault-trash-toggle:focus-visible { border-color: var(--en-border, #c5cfdd); background: var(--en-surface, #fff); color: var(--en-text, #101828); }
.en-vault-trash-toggle svg { width: 16px; height: 16px; transition: transform .16s ease; }
.en-vault-trash-toggle svg.expanded { transform: rotate(180deg); }
.en-vault-trash-state button, .en-vault-trash-item button { flex: 0 0 auto; }
.en-vault-trash-body { display: grid; gap: 9px; padding: 10px 14px 13px; }
.en-vault-trash-list { display: grid; gap: 7px; }
.en-vault-trash-item { display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 8px 0; border-bottom: 1px solid var(--en-border, #c5cfdd); }
.en-vault-trash-item > div { min-width: 0; display: grid; gap: 2px; }
.en-vault-trash-item strong, .en-vault-trash-item small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.en-vault-trash-item small { color: var(--en-muted, #667085); font: 10px/1.4 ui-monospace, SFMono-Regular, Menlo, monospace; }
.en-vault-trash-state { display: flex; align-items: center; flex-wrap: wrap; gap: 7px; padding: 8px 0; color: var(--en-muted, #667085); }
.en-vault-trash-state.is-error { color: var(--en-danger, #b42318); }
@keyframes en-vault-spin { to { transform: rotate(360deg); } }
@media (max-width: 820px) {
  .en-vault-row-detailed { grid-template-columns: 32px minmax(0, 1fr); }
  .en-vault-actions { grid-column: 2; justify-content: flex-start; }
  .en-vault-trash-summary { align-items: flex-start; }
}
</style>
