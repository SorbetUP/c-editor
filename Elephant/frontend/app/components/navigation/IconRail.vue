<template>
  <nav class="en-rail" :class="{ 'en-rail-macos': isMac }" aria-label="Workspace navigation">
    <div class="en-rail-nav">
      <template v-for="item in visibleRailItems" :key="item.id">
        <div v-if="item.separator" class="en-rail-separator en-rail-separator-custom" aria-hidden="true" />

        <div
          v-else-if="item.id === 'vault'"
          class="en-rail-vault-wrap"
          @mouseenter="showVaultMenu = true"
          @mouseleave="showVaultMenu = false"
        >
          <button
            class="en-rail-vault"
            type="button"
            :title="vaultTooltip"
            :aria-label="vaultTooltip"
            @click="runRailItem(item)"
          >
            <component :is="activeVaultIconComponent || Vault" class="en-rail-vault-lucide" aria-hidden="true" />
          </button>
          <transition name="en-vault-fade">
            <div
              v-if="showVaultMenu"
              class="en-vault-menu"
              @mouseenter="showVaultMenu = true"
              @mouseleave="showVaultMenu = false"
            >
              <div class="en-vault-menu-header">Vaults</div>
              <template v-for="vault in store.vaults" :key="vault.id">
                <div class="en-vault-menu-item" :class="{ active: vault.id === store.activeVaultId }">
                  <button class="en-vault-menu-select" type="button" @click="switchVault(vault.id)">
                    <span class="en-vault-menu-initial">
                      <component :is="getVaultIconComponent(vault) || Vault" class="en-vault-menu-lucide" aria-hidden="true" />
                    </span>
                    <span class="en-vault-menu-name">{{ vault.name }}</span>
                  </button>
                  <button class="en-vault-menu-edit" type="button" title="Change vault icon" @click.stop="toggleIconPicker(vault.id)">
                    <Pencil class="en-vault-menu-edit-icon" />
                  </button>
                  <svg v-if="vault.id === store.activeVaultId" class="en-vault-menu-check" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
                </div>
                <div v-if="editingVaultId === vault.id" class="en-vault-icon-picker" @click.stop>
                  <button class="en-vault-icon-choice" type="button" title="Use default vault icon" @mousedown.stop.prevent="setVaultIcon(vault.id, '')" @pointerdown.stop @pointerup.stop.prevent @click.stop.prevent><Vault class="en-vault-icon-choice-svg" /></button>
                  <button v-for="icon in vaultIconOptions" :key="icon.name" class="en-vault-icon-choice" :class="{ active: normalizeVaultIcon(vault.icon) === icon.name }" type="button" :title="icon.label" @mousedown.stop.prevent="setVaultIcon(vault.id, icon.name)" @pointerdown.stop @pointerup.stop.prevent @click.stop.prevent>
                    <component :is="icon.component" class="en-vault-icon-choice-svg" />
                  </button>
                </div>
              </template>
              <div class="en-vault-menu-divider" />
              <button class="en-vault-menu-item en-vault-menu-add" type="button" @click="addVault"><Plus class="en-vault-menu-add-icon" /><span>Add another vault</span></button>
            </div>
          </transition>
        </div>

        <button
          v-else
          class="en-rail-icon"
          :class="{ active: item.active, 'en-rail-sidebar-toggle': item.id === 'sidebar-toggle' }"
          type="button"
          :title="item.title"
          :aria-label="item.title"
          @click="runRailItem(item)"
        >
          <component :is="item.icon" class="en-rail-icon-svg" aria-hidden="true" />
        </button>
      </template>
    </div>

    <div class="en-rail-bottom">
      <button class="en-rail-icon" type="button" title="Settings" aria-label="Settings" @click="openSettings"><Settings class="en-rail-icon-svg" aria-hidden="true" /></button>
    </div>
  </nav>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  BookOpenText,
  CalendarDays,
  Database,
  FileText,
  GitFork,
  GraduationCap,
  Home,
  Landmark,
  LayoutDashboard,
  ListTodo,
  MessageCircle,
  PanelLeft,
  Pencil,
  Plus,
  Rocket,
  Search,
  Settings,
  Sparkles,
  Star,
  Terminal,
  Vault,
  Workflow
} from '@lucide/vue'
import { useVaultStore } from '../../stores/vaultStore'
import { getAddonSidebarItems } from '@/addons'
import { useAddonsStore } from '@/store/addons'
import { usePreferencesStore } from '@/store/preferences'
import { VAULT_ICON_OPTIONS, normalizeVaultIcon } from 'common/elephantnote/appearance'
import {
  addonViewRailId,
  extendIconRailOrder,
  isIconRailSeparatorId,
  normalizeIconRailHidden,
  normalizeIconRailOrder,
  pushIconRailLog
} from './iconRailLayout'

const props = defineProps({
  activeAddonViewId: { type: String, default: '' },
  sidebarVisible: { type: Boolean, default: true }
})
const emit = defineEmits(['open-settings', 'search', 'toggle-sidebar', 'open-addon-view', 'close-addon-view'])
const store = useVaultStore()
const addonsStore = useAddonsStore()
const preferences = usePreferencesStore()
const editingVaultId = ref('')
const showVaultMenu = ref(false)
const runtimeRailOrder = ref([])
const lastRailAction = ref({ id: '', at: 0 })
const DUPLICATE_ACTION_WINDOW_MS = 420

const VAULT_ICON_COMPONENTS = { Database, FileText, GraduationCap, Home, Landmark, Rocket, Star, Terminal, Workflow }
const ADDON_ICON_COMPONENTS = {
  'book-open-text': BookOpenText,
  'calendar-days': CalendarDays,
  calendar: CalendarDays,
  database: Database,
  'git-fork': GitFork,
  'list-todo': ListTodo,
  'message-circle': MessageCircle,
  tasks: ListTodo,
  dashboard: LayoutDashboard,
  graph: GitFork,
  search: Search,
  sparkles: Sparkles
}
const vaultIconOptions = VAULT_ICON_OPTIONS.map((option) => ({ ...option, component: VAULT_ICON_COMPONENTS[option.lucide] })).filter((option) => option.component)
const vaultIconComponentsByName = Object.fromEntries(vaultIconOptions.map((option) => [option.name, option.component]))
const isMac = navigator.platform ? navigator.platform.startsWith('Mac') : /mac/i.test(navigator.userAgent)
const getVaultIconComponent = (vault) => vaultIconComponentsByName[normalizeVaultIcon(vault?.icon)] || null
const activeVaultIconComponent = computed(() => getVaultIconComponent(store.activeVault))
const sidebarVisible = computed(() => props.sidebarVisible)
const vaultTooltip = computed(() => `${store.activeVault?.name || 'No vault'} — open vault switcher`)

const closeAddonAndOpen = (view) => {
  emit('close-addon-view')
  store.setWorkspaceView(view)
}

const toggleSidebar = () => {
  pushIconRailLog('action:sidebar-toggle', {
    currentVisible: sidebarVisible.value,
    nextVisible: !sidebarVisible.value
  })
  emit('toggle-sidebar')
}

const coreRailItems = computed(() => [
  {
    id: 'vault',
    title: vaultTooltip.value,
    icon: activeVaultIconComponent.value || Vault,
    active: false,
    run: openActiveVaultIconPicker
  },
  {
    id: 'sidebar-toggle',
    title: sidebarVisible.value ? 'Hide sidebar' : 'Show sidebar',
    icon: PanelLeft,
    active: false,
    run: toggleSidebar
  },
  { id: 'search', title: 'Search', icon: Search, active: false, run: () => emit('search') }
])

const addonViewItems = computed(() => addonsStore.getContributions('views')
  .filter((entry) => entry?.contribution?.id && entry?.contribution?.title)
  .map((entry) => {
    const contribution = entry.contribution
    return {
      id: addonViewRailId(contribution.id),
      title: contribution.title,
      icon: ADDON_ICON_COMPONENTS[contribution.icon] || Star,
      active: props.activeAddonViewId === contribution.id,
      run: () => emit('open-addon-view', contribution.id)
    }
  }))

const legacyAddonItems = computed(() => getAddonSidebarItems(addonsStore.contributions).map((item) => ({
  id: `addon-item:${item.addonId}:${item.id}`,
  title: item.tooltip || item.title || item.id,
  icon: ADDON_ICON_COMPONENTS[item.icon] || Star,
  active: Boolean(item.view && !props.activeAddonViewId && store.activeWorkspaceView === item.view),
  run: () => handleAddonSidebarItem(item)
})))

const allRailItems = computed(() => [...coreRailItems.value, ...addonViewItems.value, ...legacyAddonItems.value])
const allRailItemIds = computed(() => allRailItems.value.map((item) => item.id))
const configuredRailOrderSignature = computed(() => JSON.stringify(
  Array.isArray(preferences.iconRailOrder) ? preferences.iconRailOrder : []
))
const visibleRailItems = computed(() => {
  const ids = allRailItemIds.value
  const hidden = new Set(normalizeIconRailHidden(preferences.iconRailHidden, ids))
  const byId = new Map(allRailItems.value.map((item) => [item.id, item]))
  return normalizeIconRailOrder(runtimeRailOrder.value, ids)
    .filter((id) => isIconRailSeparatorId(id) || !hidden.has(id))
    .map((id) => isIconRailSeparatorId(id) ? { id, separator: true } : byId.get(id))
    .filter(Boolean)
})
const visibleRailIds = computed(() => visibleRailItems.value.map((item) => item.id))
const layoutSignature = computed(() => JSON.stringify({
  order: normalizeIconRailOrder(runtimeRailOrder.value, allRailItemIds.value),
  hidden: normalizeIconRailHidden(preferences.iconRailHidden, allRailItemIds.value),
  visible: visibleRailIds.value
}))

const runRailItem = async (item) => {
  const id = item?.id || ''
  const now = typeof performance !== 'undefined' ? performance.now() : Date.now()
  const previous = lastRailAction.value
  const ageMs = now - previous.at
  if (id && id !== 'sidebar-toggle' && previous.id === id && ageMs >= 0 && ageMs < DUPLICATE_ACTION_WINDOW_MS) {
    pushIconRailLog('action:ignored-duplicate', {
      id,
      ageMs: Math.round(ageMs),
      guardMs: DUPLICATE_ACTION_WINDOW_MS
    })
    return
  }
  lastRailAction.value = { id, at: now }

  pushIconRailLog('action:run', {
    id,
    title: item?.title || '',
    activeVaultId: store.activeVaultId || '',
    workspaceView: store.activeWorkspaceView || ''
  })
  try {
    await item?.run?.()
  } catch (error) {
    pushIconRailLog('action:error', {
      id,
      error: error?.message || String(error)
    })
    throw error
  }
}

const openSettings = () => runRailItem({
  id: 'settings',
  title: 'Settings',
  run: () => emit('open-settings')
})
const openActiveVaultIconPicker = () => {
  showVaultMenu.value = true
  editingVaultId.value = store.activeVaultId || ''
  pushIconRailLog('vault:menu-opened', {
    activeVaultId: store.activeVaultId || '',
    vaultCount: store.vaults.length
  })
}
const switchVault = async (vaultId) => {
  const previousVaultId = store.activeVaultId || ''
  await store.setActiveVault(vaultId)
  showVaultMenu.value = false
  pushIconRailLog('vault:switched', { previousVaultId, vaultId })
}
const addVault = async () => {
  showVaultMenu.value = false
  pushIconRailLog('vault:add:start', { vaultCount: store.vaults.length })
  const added = await store.chooseVault()
  if (added) {
    editingVaultId.value = store.activeVaultId
    showVaultMenu.value = true
  }
  pushIconRailLog('vault:add:done', {
    added: Boolean(added),
    activeVaultId: store.activeVaultId || '',
    vaultCount: store.vaults.length
  })
}
const toggleIconPicker = (vaultId) => {
  editingVaultId.value = editingVaultId.value === vaultId ? '' : vaultId
  pushIconRailLog('vault:icon-picker', {
    vaultId,
    open: editingVaultId.value === vaultId
  })
}
const setVaultIcon = async (vaultId, icon) => {
  await store.setVaultIcon(vaultId, icon)
  editingVaultId.value = ''
  pushIconRailLog('vault:icon-updated', { vaultId, icon: icon || 'default' })
}

const handleAddonSidebarItem = async (item) => {
  if (item.actionId) return addonsStore.runAction(item.actionId)
  if (item.view) closeAddonAndOpen(item.view)
}

watch(configuredRailOrderSignature, () => {
  runtimeRailOrder.value = extendIconRailOrder(preferences.iconRailOrder, allRailItemIds.value)
}, { immediate: true })

watch(allRailItemIds, (ids) => {
  const previous = runtimeRailOrder.value
  const next = extendIconRailOrder(previous, ids)
  if (JSON.stringify(previous) === JSON.stringify(next)) return
  runtimeRailOrder.value = next
  pushIconRailLog('layout:extended', {
    previous,
    next,
    added: next.filter((id) => !previous.includes(id))
  })
}, { immediate: true })

watch(layoutSignature, (signature) => {
  const snapshot = JSON.parse(signature)
  if (!snapshot.visible.includes('vault')) showVaultMenu.value = false
  pushIconRailLog('layout:resolved', snapshot)
}, { immediate: true })

onMounted(() => {
  pushIconRailLog('mounted', {
    isMac,
    activeVaultId: store.activeVaultId || '',
    sidebarVisible: sidebarVisible.value
  })
})

onBeforeUnmount(() => {
  showVaultMenu.value = false
  pushIconRailLog('unmounted')
})
</script>

<style scoped>
.en-rail { position: relative; z-index: 8; width: 48px; height: 100%; min-height: 0; display: flex; flex: 0 0 48px; flex-direction: column; align-items: center; overflow: visible; background: var(--en-sidebar-bg, var(--en-bg)); border-right: 1px solid var(--en-border); padding-top: 8px; -webkit-app-region: drag; }
.en-rail-macos { padding-top: 36px; }
.en-rail-vault-wrap { position: relative; flex: 0 0 34px; -webkit-app-region: no-drag; }
.en-rail-vault { width: 34px; height: 34px; border-radius: 10px; border: 0; background: var(--en-soft-strong, var(--en-soft)); color: var(--en-text); cursor: pointer; display: flex; align-items: center; justify-content: center; transition: opacity .15s; padding: 0; overflow: hidden; visibility: visible; opacity: 1; }
.en-rail-vault:hover { opacity: .85; }
.en-rail-vault-lucide { width: 19px; height: 19px; display: block; color: currentColor; stroke: currentColor; stroke-width: 2.2; }
.en-vault-menu { position: absolute; left: 52px; top: -4px; min-width: 250px; background: var(--en-surface); border: 1px solid var(--en-border); border-radius: 10px; box-shadow: var(--en-card-shadow, 0 8px 30px rgba(0,0,0,.18)); padding: 6px; z-index: 100; -webkit-app-region: no-drag !important; pointer-events: auto; }
.en-vault-menu-header { padding: 6px 10px 8px; font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: .06em; color: var(--en-muted); }
.en-vault-menu-item { width: 100%; display: flex; align-items: center; gap: 4px; padding: 4px; border: 0; border-radius: 6px; background: transparent; color: var(--en-text); font: inherit; font-size: 13px; text-align: left; transition: background .1s; }
.en-vault-menu-item:hover { background: var(--en-soft); }
.en-vault-menu-item.active { background: var(--en-soft-strong, var(--en-soft)); }
.en-vault-menu-select { min-width: 0; flex: 1; min-height: 32px; display: flex; align-items: center; gap: 10px; border: 0; color: inherit; background: transparent; font: inherit; text-align: left; cursor: pointer; }
.en-vault-menu-initial { width: 24px; height: 24px; border-radius: 6px; background: var(--en-soft-strong, var(--en-soft)); color: var(--en-text); font-size: 11px; font-weight: 700; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
.en-vault-menu-lucide { width: 15px; height: 15px; stroke-width: 2.2; }
.en-vault-menu-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.en-vault-menu-edit { width: 26px; height: 26px; flex: 0 0 auto; display: inline-flex; align-items: center; justify-content: center; border: 0; border-radius: 6px; color: var(--en-muted); background: transparent; cursor: pointer; opacity: 0; }
.en-vault-menu-item:hover .en-vault-menu-edit, .en-vault-menu-edit:focus-visible { opacity: 1; }
.en-vault-menu-edit:hover { color: var(--en-text); background: var(--en-soft); }
.en-vault-menu-edit-icon { width: 14px; height: 14px; }
.en-vault-menu-check { width: 16px; height: 16px; color: var(--en-primary); flex-shrink: 0; }
.en-vault-icon-picker { display: grid; grid-template-columns: repeat(5, 28px); gap: 5px; margin: 4px 4px 8px; padding: 8px; border: 1px solid var(--en-border); border-radius: 8px; background: var(--en-surface); box-shadow: var(--en-card-shadow, 0 8px 30px rgba(0,0,0,.18)); -webkit-app-region: no-drag !important; pointer-events: auto; }
.en-vault-icon-choice { width: 28px; height: 28px; display: inline-flex; align-items: center; justify-content: center; border: 0; border-radius: 7px; color: var(--en-muted); background: transparent; font-size: 12px; font-weight: 800; cursor: pointer; -webkit-app-region: no-drag !important; pointer-events: auto; }
.en-vault-icon-choice:hover, .en-vault-icon-choice.active { color: var(--en-text); background: var(--en-soft-strong, var(--en-soft)); }
.en-vault-icon-choice-svg { width: 16px; height: 16px; }
.en-vault-menu-divider { height: 1px; background: var(--en-border); margin: 4px 0; }
.en-vault-menu-add { color: var(--en-muted); }
.en-vault-menu-add:hover { color: var(--en-text); background: var(--en-soft); }
.en-vault-menu-add-icon { width: 16px; height: 16px; flex-shrink: 0; }
.en-rail-nav { min-height: 0; flex: 1; display: flex; flex-direction: column; align-items: center; gap: 2px; -webkit-app-region: no-drag; overflow-y: auto; overflow-x: visible; scrollbar-width: none; }
.en-rail-nav::-webkit-scrollbar { display: none; }
.en-rail-separator { width: 24px; height: 1px; flex: 0 0 1px; background: var(--en-border); margin: 5px 0; }
.en-rail-bottom { display: flex; flex-direction: column; align-items: center; gap: 2px; -webkit-app-region: no-drag; padding: 6px 0 8px; }
.en-rail-icon { width: 34px; height: 34px; flex: 0 0 34px; display: flex; align-items: center; justify-content: center; border: 0; border-radius: 8px; color: var(--en-muted); background: transparent; cursor: pointer; transition: color .12s, background .12s; visibility: visible; opacity: 1; }
.en-rail-icon:hover { color: var(--en-text); background: var(--en-soft); }
.en-rail-icon.active { color: var(--en-text); background: var(--en-soft-strong, var(--en-soft)); }
.en-rail-icon-svg { width: 18px; height: 18px; display: block; color: currentColor; stroke: currentColor; }
.en-vault-fade-enter-active, .en-vault-fade-leave-active { transition: opacity .12s ease, transform .12s ease; }
.en-vault-fade-enter-from, .en-vault-fade-leave-to { opacity: 0; transform: translateX(-4px); }
</style>
