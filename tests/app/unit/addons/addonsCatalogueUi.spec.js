import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('addon catalogue interface', () => {
  it('keeps addon and pack search plus installed filtering in the shared top toolbar', () => {
    const panel = read('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue')

    expect(panel).toContain('class="en-addons-toolbar"')
    expect(panel).toContain('class="en-addons-search"')
    expect(panel).toContain('v-model.trim="query"')
    expect(panel).toContain('placeholder="Search addons"')
    expect(panel).toContain('v-model.trim="packQuery"')
    expect(panel).toContain('placeholder="Search addon packs"')
    expect(panel).toContain('class="en-installed-only-control"')
    expect(panel).not.toContain('class="en-addon-browser-search"')
    expect(panel).not.toContain('class="en-addon-browser-filter"')
  })

  it('renders clear tiles first and the persistent list/detail browser after selection', () => {
    const panel = read('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue')

    expect(panel).toContain('v-if="!selectedEntry" class="en-addon-catalogue"')
    expect(panel).toContain('class="en-addon-tile"')
    expect(panel).toContain('v-else class="en-addon-browser"')
    expect(panel).toContain("'en-addon-browser-detail-mode': true")
    expect(panel).toContain('class="en-addon-browser-sidebar"')
    expect(panel).toContain('class="en-addon-browser-detail"')
    expect(panel).toContain('<span>Catalogue</span>')
    expect(panel).not.toContain('en-addon-browser-overview')
    expect(panel).not.toContain('Browse the complete catalogue. Open an addon to manage it.')
    expect(panel).not.toContain('<h2>All addons</h2>')
  })

  it('keeps every AI module visible and independently installable', () => {
    const panel = read('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue')

    for (const addonId of [
      'elephant.ai-chat',
      'elephant.ai-search',
      'elephant.ai-ocr',
      'elephant.wiki',
      'elephant.graph'
    ]) expect(panel).toContain(`'${addonId}'`)
    expect(panel).toContain('const installAiModule = async (module) =>')
    expect(panel).toContain('@click="installAiModule(module)"')
  })

  it('deduplicates installed entries that also have updates', () => {
    const panel = read('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue')

    expect(panel).toContain('const entriesById = new Map()')
    expect(panel).toContain('existing.available = available')
    expect(panel).toContain("return 'Update available'")
  })
})
