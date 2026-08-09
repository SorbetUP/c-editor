import { describe, expect, it, vi } from 'vitest'
import {
  ADDON_EXTENSION_POINTS,
  getAddonActions,
  getAddonContributions,
  getAddonSettingsSections,
  getAddonSidebarItems
} from '@/addons'

describe('addon contribution selectors', () => {
  it('returns normalized contributions for a known area', () => {
    const map = {
      [ADDON_EXTENSION_POINTS.views]: [
        { addonId: 'demo.addon', contribution: { id: 'view-a' } },
        null,
        { addonId: '', contribution: { id: 'bad' } }
      ]
    }

    expect(getAddonContributions(map, ADDON_EXTENSION_POINTS.views)).toEqual([
      { addonId: 'demo.addon', contribution: { id: 'view-a' } }
    ])
  })

  it('sorts addon actions by title and keeps executable handlers', () => {
    const run = vi.fn()
    const actions = getAddonActions({
      [ADDON_EXTENSION_POINTS.actions]: [
        { addonId: 'b.addon', contribution: { id: 'b', title: 'Beta', run } },
        { addonId: 'a.addon', contribution: { id: 'a', title: 'Alpha' } }
      ]
    })

    expect(actions.map((action) => action.id)).toEqual(['a', 'b'])
    expect(actions[1].run).toBe(run)
  })

  it('sorts sidebar items by order then title', () => {
    const items = getAddonSidebarItems({
      [ADDON_EXTENSION_POINTS.sidebarItems]: [
        { addonId: 'demo', contribution: { id: 'z', title: 'Zeta', order: 2 } },
        { addonId: 'demo', contribution: { id: 'a', title: 'Alpha', order: 1 } }
      ]
    })

    expect(items.map((item) => item.id)).toEqual(['a', 'z'])
  })

  it('normalizes addon settings sections', () => {
    const sections = getAddonSettingsSections({
      [ADDON_EXTENSION_POINTS.settingsSections]: [
        { addonId: 'demo', contribution: { id: 'settings', title: 'Settings', description: 'Demo' } }
      ]
    })

    expect(sections).toEqual([
      {
        addonId: 'demo',
        id: 'settings',
        title: 'Settings',
        description: 'Demo',
        order: 1000
      }
    ])
  })
})
