import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it, vi } from 'vitest'
import DashboardAddon from '../../../../addons/official/dashboard/main.js'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('Dashboard physical addon note-only contract', () => {
  it('registers one vertical-rail action and opens the normal hidden Dashboard note', async () => {
    const store = {
      activeVault: null,
      openNote: vi.fn(async () => {})
    }
    const invoke = vi.fn()
    let action = null
    let sidebarItem = null
    const api = {
      app: {
        pinia: {
          _s: {
            get: vi.fn((id) => id === 'elephantnoteVaults' ? store : null)
          }
        }
      },
      experimental: {
        window: {
          __TAURI__: {
            core: { invoke }
          }
        }
      },
      commands: {
        register: vi.fn((entry) => { action = entry })
      },
      workspace: {
        registerSidebarItem: vi.fn((entry) => { sidebarItem = entry })
      }
    }

    const addon = new DashboardAddon(api)
    addon.onload(api)

    expect(api.commands.register).toHaveBeenCalledTimes(1)
    expect(api.workspace.registerSidebarItem).toHaveBeenCalledTimes(1)
    expect(sidebarItem).toMatchObject({
      title: 'Dashboard',
      icon: 'dashboard',
      actionId: 'elephant.dashboard.open'
    })

    store.activeVault = { path: '/vault' }
    invoke
      .mockRejectedValueOnce(new Error('missing Dashboard note'))
      .mockResolvedValueOnce({
        path: '.elephantnote/Dashboard.md',
        title: 'Dashboard',
        type: 'note'
      })

    await action.run()

    expect(invoke).toHaveBeenNthCalledWith(1, 'tauri_notes_read', {
      relativePath: '.elephantnote/Dashboard.md'
    })
    expect(invoke).toHaveBeenNthCalledWith(2, 'tauri_notes_create', {
      relativePath: '.elephantnote',
      filename: 'Dashboard.md',
      title: 'Dashboard'
    })
    expect(store.openNote).toHaveBeenCalledWith(expect.objectContaining({
      path: '.elephantnote/Dashboard.md',
      fullPath: '/vault/.elephantnote/Dashboard.md',
      title: 'Dashboard',
      kind: 'note',
      type: 'note'
    }), { record: false })
  })

  it('contains no custom dashboard workspace, cards, timer or styling', () => {
    const source = read('addons/official/dashboard/main.js')
    const manifest = JSON.parse(read('addons/official/dashboard/manifest.json'))

    expect(source).toContain("const DASHBOARD_DIRECTORY = '.elephantnote'")
    expect(source).toContain('api.workspace.registerSidebarItem')
    expect(source).not.toContain('api.workspace.registerView')
    expect(source).not.toContain('createDomComponent')
    expect(source).not.toContain('registerStyle')
    expect(source).not.toContain('setInterval')
    expect(source).not.toContain('dashboard.provider')
    expect(source).not.toContain('Recently edited')
    expect(manifest.version).toBe('1.0.1')
    expect(manifest.permissions?.views).not.toBe(true)
  })
})
