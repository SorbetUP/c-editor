import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  CORE_ICON_RAIL_ITEMS,
  DEFAULT_ICON_RAIL_ORDER,
  addonViewRailId,
  createIconRailSeparatorId,
  extendIconRailOrder,
  isIconRailSeparatorId,
  moveIconRailItem,
  normalizeIconRailHidden,
  normalizeIconRailOrder
} from '../../../../Elephant/frontend/app/components/navigation/iconRailLayout.js'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')
const exists = (relativePath) => fs.existsSync(path.join(root, relativePath))

describe('optional first-party physical addons and configurable icon rail', () => {
  it('normalizes persisted order as core controls, dividers and addon views change', () => {
    const tasks = addonViewRailId('com.elephantnote.elephant-tasks.workspace')
    const divider = createIconRailSeparatorId()
    const available = ['vault', 'sidebar-toggle', tasks, 'search']
    expect(CORE_ICON_RAIL_ITEMS.map((item) => item.id)).toEqual(['vault', 'sidebar-toggle', 'search'])
    expect(DEFAULT_ICON_RAIL_ORDER).toEqual(['vault', 'sidebar-toggle', 'search'])
    expect(isIconRailSeparatorId(divider)).toBe(true)
    expect(normalizeIconRailOrder(['search', divider, 'unknown', 'search', tasks], available))
      .toEqual(['vault', 'sidebar-toggle', 'search', divider, tasks])
    expect(normalizeIconRailHidden(['unknown', divider, 'vault', tasks, tasks], available))
      .toEqual(['vault', tasks])
    expect(moveIconRailItem(['vault', 'sidebar-toggle', divider, tasks], tasks, 0))
      .toEqual([tasks, 'vault', 'sidebar-toggle', divider])
  })

  it('appends late addon registrations without moving existing rail buttons', () => {
    const chat = 'addon-item:elephant.ai-chat:elephant.ai-chat.sidebar-item'
    const wiki = addonViewRailId('elephant.wiki.workspace')
    const graph = addonViewRailId('elephant.graph.workspace')
    const initial = ['vault', 'sidebar-toggle', 'search', chat]
    const withWiki = extendIconRailOrder(initial, ['vault', 'sidebar-toggle', 'search', wiki, chat])
    const withGraph = extendIconRailOrder(withWiki, ['vault', 'sidebar-toggle', 'search', wiki, graph, chat])

    expect(withWiki).toEqual(['vault', 'sidebar-toggle', 'search', chat, wiki])
    expect(withGraph).toEqual(['vault', 'sidebar-toggle', 'search', chat, wiki, graph])
    expect(normalizeIconRailOrder(withGraph, ['vault', 'sidebar-toggle', 'search', wiki, graph, chat]))
      .toEqual(['vault', 'sidebar-toggle', 'search', chat, wiki, graph])
  })

  it('renders vault and sidebar through the same customizable icon layout', () => {
    const rail = read('Elephant/frontend/app/components/navigation/IconRail.vue')
    const organizer = read('Elephant/frontend/app/components/settings/IconRailLayoutSettings.vue')
    const layout = read('Elephant/frontend/app/components/navigation/iconRailLayout.js')
    const shell = read('Elephant/frontend/app/components/shell/AppShell.vue')
    expect(layout).toContain("{ id: 'vault', label: 'Vault'")
    expect(layout).toContain("{ id: 'sidebar-toggle', label: 'Sidebar'")
    expect(layout).not.toContain("{ id: 'dashboard', label: 'Dashboard'")
    expect(rail).toContain("id: 'vault'")
    expect(rail).toContain("id: 'sidebar-toggle'")
    expect(rail).toContain('icon: PanelLeft')
    expect(rail).toContain('activeVaultIconComponent || Vault')
    expect(rail).toContain('extendIconRailOrder')
    expect(rail).toContain("action:ignored-duplicate")
    expect(shell).toContain(':sidebar-visible="sidebarVisible"')
    expect(rail).toContain('v-if="item.separator"')
    expect(organizer).toContain('vault: Vault')
    expect(organizer).toContain("'sidebar-toggle': PanelLeft")
  })

  it('loads Recently edited only from its physical package', () => {
    const sidebar = read('Elephant/frontend/app/components/navigation/SidebarNav.vue')
    const recentAddon = read('addons/official/recently-edited/main.js')
    expect(sidebar).toContain("addonsStore.getContributions('layout.zones')")
    expect(sidebar).toContain("entry?.contribution?.zone === 'sidebar.after-tree'")
    expect(recentAddon).toContain("zone: 'sidebar.after-tree'")
    expect(exists('Elephant/frontend/src/renderer/src/addons/builtin/recentlyEdited.js')).toBe(false)
    expect(exists('Elephant/frontend/app/components/navigation/RecentlyEditedSidebarSection.vue')).toBe(false)
  })

  it('loads Calendar, Sync and Sites only from physical package entries', () => {
    const builtins = read('Elephant/frontend/src/renderer/src/addons/builtin/index.js')
    const calendar = read('addons/official/calendar/main.js')
    const sync = read('addons/official/sync/main.js')
    const sites = read('addons/official/sites/main.js')
    const navigation = read('Elephant/frontend/app/components/navigation/NavigationBar.vue')
    const router = read('Elephant/frontend/app/components/views/AddonWorkspaceRouter.vue')

    expect(builtins).not.toContain("import('./calendar')")
    expect(builtins).not.toContain("import('./sync')")
    expect(builtins).not.toContain("import('./sites')")
    expect(calendar).toContain('api.workspace.registerView')
    expect(calendar).toContain("const PROVIDER_RESOURCE = 'calendar.provider'")
    expect(sync).toContain("registerContribution('top-bar.items'")
    expect(sites).toContain('api.settings.registerSection')
    expect(sites).toContain("const PROVIDER_RESOURCE = 'sites.provider'")
    expect(navigation).toContain(':is="entry.contribution.component"')
    expect(router).toContain(':is="view.contribution.component"')
  })

  it('keeps AI capabilities independently downloadable under the physical AI parent', () => {
    const shell = read('Elephant/frontend/app/components/shell/AppShell.vue')
    const router = read('Elephant/frontend/app/components/views/AddonWorkspaceRouter.vue')
    const parent = read('addons/official/ai/main.js')
    const panel = read('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue')
    const catalog = read('addons/catalog.json')
    const chat = read('addons/official/ai-chat/main.js')
    const search = read('addons/official/ai-search/main.js')
    const ocrManifest = read('addons/official/ai-ocr/manifest.json')
    const wiki = read('addons/official/wiki/main.js')
    const graph = read('addons/official/graph/main.js')
    const openModels = read('addons/official/open-models/main.js')

    expect(shell).toContain("entry?.contribution?.zone === 'shell.right'")
    expect(router).toContain(':is="view.contribution.component"')
    expect(parent).toContain("setAttribute('data-elephant-addon-settings-slot', active.slot)")
    expect(panel).toContain('const AI_SUBMODULE_IDS = Object.freeze([')
    expect(panel).toContain('const installAiModule = async (module) =>')
    for (const addonId of ['elephant.ai-chat', 'elephant.ai-search', 'elephant.ai-ocr', 'elephant.wiki', 'elephant.graph']) {
      expect(panel).toContain(`'${addonId}'`)
      expect(catalog).toContain(`"id": "${addonId}"`)
    }
    expect(chat).toContain("slot: 'ai.chat'")
    expect(search).toContain("slot: 'ai.search'")
    expect(ocrManifest).toContain('"elephant.ai": ">=2.0.0"')
    expect(wiki).toContain('api.workspace.registerView')
    expect(graph).toContain('api.workspace.registerView')
    expect(openModels).toContain("registerContribution('ai.providers'")
  })

  it('has no builtin addon lazy loader and bootstraps Addon Packs plus Excalidraw as core features', () => {
    const builtins = read('Elephant/frontend/src/renderer/src/addons/builtin/index.js')
    const main = read('Elephant/frontend/src/renderer/src/main.js')
    expect(builtins).toContain('builtinAddons = Object.freeze([])')
    expect(builtins).not.toContain('import(')
    expect(main).toContain('addonPacksCoreFeature')
    expect(main).toContain('excalidrawCoreFeature')
    expect(main).toContain('activateCoreFeature')
  })
})
