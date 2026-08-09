import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import { ADDON_EXTENSION_POINTS, ElephantAddonManager } from '@/addons'
import { activateCoreFeature } from '@/addons/coreFeatures'
import { addonPacksCoreFeature } from '@/addons/builtin/addonProfiles'
import { excalidrawCoreFeature } from '@/addons/builtin/excalidraw'
import { builtinAddons } from '@/addons/builtin'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')
const exists = (relativePath) => fs.existsSync(path.join(root, relativePath))
const PHYSICAL_ADDONS = [
  'ai',
  'ai-chat',
  'ai-search',
  'ai-ocr',
  'wiki',
  'graph',
  'open-models',
  'codex-connection',
  'sync',
  'calendar',
  'sites',
  'code-execution',
  'google-keep-import',
  'recently-edited'
]

const createManager = () => {
  const manager = new ElephantAddonManager()
  manager.host = { get: () => null }
  return manager
}

describe('core features and physical addons', () => {
  it('does not expose any application capability as a builtin addon', () => {
    expect(builtinAddons).toEqual([])
    for (const slug of PHYSICAL_ADDONS) {
      expect(read(`addons/official/${slug}/manifest.json`)).toContain('"runtime"')
      expect(read(`addons/official/${slug}/main.js`).length).toBeGreaterThan(20)
    }
  })

  it('runs Addon Packs as an always-available core feature rather than an addon record', async () => {
    const manager = createManager()
    await activateCoreFeature(manager, addonPacksCoreFeature)

    expect(manager.list()).toEqual([])
    expect(manager.coreFeatures.has('core.addon-packs')).toBe(true)
    expect(manager.getContributions(ADDON_EXTENSION_POINTS.actions)
      .map((entry) => entry.contribution.id)
      .sort())
      .toEqual([
        'elephant.addon-packs.apply',
        'elephant.addon-packs.create',
        'elephant.addon-packs.ensure-develop-parity'
      ])
    expect(manager.getContributions(ADDON_EXTENSION_POINTS.settingsSections))
      .toEqual([expect.objectContaining({
        source: 'core',
        coreFeatureId: 'core.addon-packs'
      })])
  })

  it('boots Excalidraw as a core editor capability without an addon manifest', () => {
    expect(excalidrawCoreFeature.id).toBe('core.excalidraw')
    expect(excalidrawCoreFeature.manifest).toBeUndefined()
    expect(read('Elephant/frontend/src/renderer/src/main.js')).toContain('await installCoreFeatures(addonManager)')
    expect(read('Elephant/frontend/src/renderer/src/main.js')).toContain('excalidrawCoreFeature')
  })

  it('keeps Google Keep Import and Recently edited physically outside core source trees', () => {
    expect(exists('Elephant/frontend/src/renderer/src/addons/builtin/googleKeepImport.js')).toBe(false)
    expect(exists('Elephant/frontend/src/renderer/src/addons/builtin/recentlyEdited.js')).toBe(false)
    expect(exists('Elephant/frontend/src/renderer/src/addons/builtin/ui/ImportSettings.vue')).toBe(false)
    expect(exists('Elephant/frontend/app/components/navigation/RecentlyEditedSidebarSection.vue')).toBe(false)

    expect(read('addons/official/google-keep-import/main.js')).toContain('ElephantGoogleKeepImportAddon')
    expect(read('addons/official/recently-edited/main.js')).toContain('ElephantRecentlyEditedAddon')
  })
})
