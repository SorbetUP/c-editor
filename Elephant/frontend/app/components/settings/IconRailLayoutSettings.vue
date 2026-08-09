<template>
  <div class="en-rail-layout-settings">
    <header class="en-rail-layout-header">
      <strong>Vertical icon bar</strong>
      <div class="en-rail-layout-header-actions">
        <button type="button" title="Add divider" aria-label="Add divider" @click="addDivider"><Plus aria-hidden="true" /></button>
        <button type="button" title="Reset icon bar" aria-label="Reset icon bar" @click="resetLayout"><RotateCcw aria-hidden="true" /></button>
        <button type="button" :title="collapsed ? 'Expand icon bar settings' : 'Collapse icon bar settings'" :aria-expanded="!collapsed" @click="toggleCollapsed">
          <ChevronDown :class="{ collapsed }" aria-hidden="true" />
        </button>
      </div>
    </header>

    <div v-if="!collapsed" class="en-rail-layout-list" role="list" aria-label="Vertical icon bar order">
      <article
        v-for="(item, index) in orderedItems"
        :key="item.id"
        class="en-rail-layout-item"
        :class="{ hidden: isHidden(item.id), dragging: draggingId === item.id, separator: item.separator }"
        draggable="true"
        role="listitem"
        @dragstart="startDrag(item.id, $event)"
        @dragend="endDrag(item.id)"
        @dragover.prevent
        @drop.prevent="dropOn(item.id)"
      >
        <GripVertical class="en-rail-layout-grip" aria-hidden="true" />
        <span class="en-rail-layout-icon-preview" :class="{ divider: item.separator }" aria-hidden="true">
          <component :is="item.iconComponent" v-if="item.iconComponent" />
          <span v-else-if="item.separator" />
          <Star v-else />
        </span>
        <div class="en-rail-layout-copy">
          <strong>{{ item.separator ? 'Divider' : item.label }}</strong>
          <span v-if="item.separator" class="en-rail-layout-divider-preview" aria-hidden="true" />
        </div>
        <div class="en-rail-layout-actions">
          <button type="button" :disabled="index === 0" :aria-label="`Move ${item.label || 'divider'} up`" @click="move(item.id, index - 1)"><ChevronUp aria-hidden="true" /></button>
          <button type="button" :disabled="index === orderedItems.length - 1" :aria-label="`Move ${item.label || 'divider'} down`" @click="move(item.id, index + 1)"><ChevronDown aria-hidden="true" /></button>
          <button v-if="item.separator" type="button" aria-label="Remove divider" @click="removeDivider(item.id)"><Trash2 aria-hidden="true" /></button>
          <button v-else type="button" :aria-label="`${isHidden(item.id) ? 'Show' : 'Hide'} ${item.label}`" @click="toggleVisibility(item.id)">
            <EyeOff v-if="isHidden(item.id)" aria-hidden="true" />
            <Eye v-else aria-hidden="true" />
          </button>
        </div>
      </article>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import {
  BookOpenText,
  CalendarDays,
  ChevronDown,
  ChevronUp,
  Database,
  Eye,
  EyeOff,
  GitFork,
  GripVertical,
  LayoutDashboard,
  ListTodo,
  MessageCircle,
  PanelLeft,
  Plus,
  RotateCcw,
  Search,
  Sparkles,
  Star,
  Trash2,
  Vault
} from '@lucide/vue'
import { usePreferencesStore } from '@/store/preferences'
import { useAddonsStore } from '@/store/addons'
import { getAddonSidebarItems } from '@/addons'
import {
  CORE_ICON_RAIL_ITEMS,
  DEFAULT_ICON_RAIL_ORDER,
  addonViewRailId,
  createIconRailSeparatorId,
  extendIconRailOrder,
  isIconRailSeparatorId,
  moveIconRailItem,
  normalizeIconRailHidden,
  normalizeIconRailOrder,
  pushIconRailLog
} from '../navigation/iconRailLayout'

const preferences = usePreferencesStore()
const addonsStore = useAddonsStore()
const draggingId = ref('')
const collapsed = ref(false)
const runtimeOrder = ref([])

const CORE_ICON_COMPONENTS = {
  vault: Vault,
  'sidebar-toggle': PanelLeft,
  search: Search
}
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

const addonItems = computed(() => addonsStore.getContributions('views')
  .filter((entry) => entry?.contribution?.id && entry?.contribution?.title)
  .map((entry) => ({
    id: addonViewRailId(entry.contribution.id),
    label: entry.contribution.title,
    source: 'addon',
    iconComponent: ADDON_ICON_COMPONENTS[entry.contribution.icon] || Star
  })))

const legacyAddonItems = computed(() => getAddonSidebarItems(addonsStore.contributions).map((item) => ({
  id: `addon-item:${item.addonId}:${item.id}`,
  label: item.title || item.tooltip || item.id,
  source: 'addon',
  iconComponent: ADDON_ICON_COMPONENTS[item.icon] || Star
})))

const availableItems = computed(() => [
  ...CORE_ICON_RAIL_ITEMS.map((item) => ({
    ...item,
    source: 'core',
    iconComponent: CORE_ICON_COMPONENTS[item.id] || Star
  })),
  ...addonItems.value,
  ...legacyAddonItems.value
])
const availableIds = computed(() => availableItems.value.map((item) => item.id))
const configuredOrderSignature = computed(() => JSON.stringify(
  Array.isArray(preferences.iconRailOrder) ? preferences.iconRailOrder : []
))
const orderedIds = computed(() => normalizeIconRailOrder(runtimeOrder.value, availableIds.value))
const hiddenIds = computed(() => normalizeIconRailHidden(preferences.iconRailHidden, availableIds.value))
const orderedItems = computed(() => {
  const byId = new Map(availableItems.value.map((item) => [item.id, item]))
  return orderedIds.value.map((id) => isIconRailSeparatorId(id) ? { id, separator: true } : byId.get(id)).filter(Boolean)
})

const persistOrder = (order, reason, details = {}) => {
  const normalized = normalizeIconRailOrder(order, availableIds.value)
  pushIconRailLog('settings:order-persist', {
    reason,
    previous: orderedIds.value,
    next: normalized,
    ...details
  })
  preferences.SET_SINGLE_PREFERENCE({
    type: 'iconRailOrder',
    value: normalized
  })
}

const persistHidden = (hidden, reason, details = {}) => {
  const normalized = normalizeIconRailHidden(hidden, availableIds.value)
  pushIconRailLog('settings:hidden-persist', {
    reason,
    previous: hiddenIds.value,
    next: normalized,
    ...details
  })
  preferences.SET_SINGLE_PREFERENCE({
    type: 'iconRailHidden',
    value: normalized
  })
}

const isHidden = (id) => hiddenIds.value.includes(id)
const move = (id, index) => persistOrder(
  moveIconRailItem(orderedIds.value, id, index),
  'button-move',
  { id, targetIndex: index }
)
const addDivider = () => {
  const dividerId = createIconRailSeparatorId()
  persistOrder([...orderedIds.value, dividerId], 'divider-add', { dividerId })
}
const removeDivider = (id) => persistOrder(
  orderedIds.value.filter((candidate) => candidate !== id),
  'divider-remove',
  { id }
)

const toggleVisibility = (id) => {
  const next = new Set(hiddenIds.value)
  const willShow = next.has(id)
  if (willShow) next.delete(id)
  else next.add(id)
  persistHidden([...next], 'visibility-toggle', {
    id,
    visible: willShow
  })
}

const startDrag = (id, event) => {
  draggingId.value = id
  pushIconRailLog('settings:drag-start', { id })
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', id)
  }
}

const endDrag = (id) => {
  draggingId.value = ''
  pushIconRailLog('settings:drag-end', { id })
}

const dropOn = (targetId) => {
  const sourceId = draggingId.value
  draggingId.value = ''
  if (!sourceId || sourceId === targetId) {
    pushIconRailLog('settings:drop-ignored', { sourceId, targetId })
    return
  }
  const targetIndex = orderedIds.value.indexOf(targetId)
  persistOrder(
    moveIconRailItem(orderedIds.value, sourceId, targetIndex),
    'drag-drop',
    { sourceId, targetId, targetIndex }
  )
}

const resetLayout = () => {
  pushIconRailLog('settings:reset', {
    availableIds: availableIds.value,
    defaultOrder: DEFAULT_ICON_RAIL_ORDER
  })
  persistOrder(normalizeIconRailOrder(DEFAULT_ICON_RAIL_ORDER, availableIds.value), 'reset')
  persistHidden([], 'reset')
}

const toggleCollapsed = () => {
  collapsed.value = !collapsed.value
  pushIconRailLog('settings:collapsed', { collapsed: collapsed.value })
}

watch(configuredOrderSignature, () => {
  runtimeOrder.value = extendIconRailOrder(preferences.iconRailOrder, availableIds.value)
}, { immediate: true })

watch(availableIds, (ids) => {
  const previous = runtimeOrder.value
  const next = extendIconRailOrder(previous, ids)
  if (JSON.stringify(previous) === JSON.stringify(next)) return
  runtimeOrder.value = next
  pushIconRailLog('settings:layout-extended', {
    previous,
    next,
    added: next.filter((id) => !previous.includes(id))
  })
}, { immediate: true })

onMounted(() => {
  pushIconRailLog('settings:mounted', {
    available: availableItems.value.map((item) => ({ id: item.id, label: item.label, source: item.source })),
    order: orderedIds.value,
    hidden: hiddenIds.value
  })
})
</script>

<style scoped>
.en-rail-layout-settings { width: 100%; display: grid; gap: 10px; }
.en-rail-layout-header { min-height: 34px; display: flex; align-items: center; justify-content: space-between; gap: 16px; }
.en-rail-layout-header > strong { font-size: 13px; font-weight: 650; }
.en-rail-layout-header-actions, .en-rail-layout-actions { display: flex; gap: 4px; }
.en-rail-layout-header-actions button, .en-rail-layout-actions button { width: 28px; height: 28px; display: inline-flex; align-items: center; justify-content: center; padding: 0; border: 1px solid var(--en-border); border-radius: 7px; color: var(--en-text); background: var(--en-surface); cursor: pointer; }
.en-rail-layout-header-actions button:hover, .en-rail-layout-actions button:hover:not(:disabled) { background: var(--en-soft); }
.en-rail-layout-header-actions svg, .en-rail-layout-actions svg { width: 14px; height: 14px; }
.en-rail-layout-header-actions .collapsed { transform: rotate(-90deg); }
.en-rail-layout-list { display: grid; border: 1px solid var(--en-border); border-radius: 10px; overflow: hidden; }
.en-rail-layout-item { min-height: 48px; display: grid; grid-template-columns: 22px 24px minmax(0, 1fr) auto; align-items: center; gap: 10px; padding: 6px 9px; background: var(--en-surface); }
.en-rail-layout-item + .en-rail-layout-item { border-top: 1px solid var(--en-border); }
.en-rail-layout-item.dragging { opacity: .48; }
.en-rail-layout-item.hidden .en-rail-layout-copy, .en-rail-layout-item.hidden .en-rail-layout-icon-preview { opacity: .55; }
.en-rail-layout-item.separator { min-height: 42px; }
.en-rail-layout-grip { width: 16px; height: 16px; color: var(--en-muted); cursor: grab; }
.en-rail-layout-icon-preview { width: 24px; height: 24px; display: inline-flex; align-items: center; justify-content: center; border-radius: 6px; color: var(--en-muted); background: var(--en-soft); }
.en-rail-layout-icon-preview svg { width: 15px; height: 15px; display: block; stroke: currentColor; }
.en-rail-layout-icon-preview.divider { background: transparent; }
.en-rail-layout-icon-preview.divider > span { width: 20px; height: 1px; background: var(--en-border-strong, var(--en-border)); }
.en-rail-layout-copy { min-width: 0; display: flex; align-items: center; gap: 10px; }
.en-rail-layout-copy strong { flex: 0 0 auto; font-size: 12px; }
.en-rail-layout-divider-preview { width: 34px; height: 1px; background: var(--en-border-strong, var(--en-border)); }
.en-rail-layout-actions button:disabled { opacity: .35; cursor: default; }
</style>
