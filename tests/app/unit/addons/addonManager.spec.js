import { describe, expect, it, vi } from 'vitest'
import { ADDON_EXTENSION_POINTS, ADDON_STATUS, ElephantAddonManager } from '@/addons'

const createTestAddon = (overrides = {}) => ({
  manifest: {
    id: 'test.addon',
    name: 'Test Addon',
    ...overrides.manifest
  },
  activate: overrides.activate || vi.fn(),
  deactivate: overrides.deactivate || vi.fn()
})

describe('ElephantAddonManager', () => {
  it('registers addons and returns immutable snapshots', () => {
    const manager = new ElephantAddonManager()
    const snapshot = manager.register(createTestAddon())

    expect(snapshot.manifest.id).toBe('test.addon')
    expect(snapshot.enabled).toBe(false)
    expect(snapshot.status).toBe(ADDON_STATUS.disabled)
    expect(manager.list()).toHaveLength(1)
  })

  it('activates an addon and stores contributions', async () => {
    const manager = new ElephantAddonManager()
    const activate = vi.fn((ctx) => {
      ctx.addAction({ id: 'open-graph', title: 'Open graph' })
    })

    manager.register(createTestAddon({ activate }))
    const snapshot = await manager.enable('test.addon')

    expect(snapshot.enabled).toBe(true)
    expect(snapshot.status).toBe(ADDON_STATUS.enabled)
    expect(activate).toHaveBeenCalledOnce()
    expect(manager.getContributions(ADDON_EXTENSION_POINTS.actions)).toEqual([
      {
        addonId: 'test.addon',
        contribution: { id: 'open-graph', title: 'Open graph' }
      }
    ])
    expect(manager.getContributionMap()[ADDON_EXTENSION_POINTS.actions]).toHaveLength(1)
  })

  it('ignores duplicate contribution ids from the same addon and area', () => {
    const logger = { info: vi.fn(), warn: vi.fn(), error: vi.fn() }
    const manager = new ElephantAddonManager({ logger })
    manager.register(createTestAddon())

    manager.registerContribution('test.addon', ADDON_EXTENSION_POINTS.layoutZones, {
      id: 'chat-sidebar',
      zone: 'shell.right'
    })
    manager.registerContribution('test.addon', ADDON_EXTENSION_POINTS.layoutZones, {
      id: 'chat-sidebar',
      zone: 'shell.right'
    })

    expect(manager.getContributions(ADDON_EXTENSION_POINTS.layoutZones)).toHaveLength(1)
    expect(logger.warn).toHaveBeenCalledWith('duplicate addon contribution ignored', {
      addonId: 'test.addon',
      area: ADDON_EXTENSION_POINTS.layoutZones,
      contributionId: 'chat-sidebar'
    })
  })

  it('shares one activation when enable is called concurrently', async () => {
    const manager = new ElephantAddonManager()
    let release
    const activate = vi.fn(() => new Promise((resolve) => { release = resolve }))
    manager.register(createTestAddon({ activate }))

    const first = manager.enable('test.addon')
    const second = manager.enable('test.addon')
    expect(activate).toHaveBeenCalledOnce()
    release()

    await expect(Promise.all([first, second])).resolves.toHaveLength(2)
    expect(manager.get('test.addon').enabled).toBe(true)
  })

  it('runs executable addon actions', async () => {
    const manager = new ElephantAddonManager()
    const run = vi.fn((payload, meta) => ({ payload, meta }))

    manager.register(createTestAddon({
      activate: vi.fn((ctx) => {
        ctx.addAction({ id: 'test.run', title: 'Run test', run })
      })
    }))

    await manager.enable('test.addon')
    const result = await manager.runAction('test.run', { value: 42 })

    expect(run).toHaveBeenCalledOnce()
    expect(result.payload).toEqual({ value: 42 })
    expect(result.meta).toMatchObject({
      addonId: 'test.addon',
      actionId: 'test.run',
      addons: manager
    })
  })

  it('rejects unknown or non executable actions', async () => {
    const manager = new ElephantAddonManager()
    manager.register(createTestAddon({
      activate: vi.fn((ctx) => {
        ctx.addAction({ id: 'test.not-executable', title: 'No run' })
      })
    }))

    await manager.enable('test.addon')

    await expect(manager.runAction('missing.action')).rejects.toThrow(/Unknown addon action/)
    await expect(manager.runAction('test.not-executable')).rejects.toThrow(/not executable/)
  })

  it('emits contribution changes when entries are added and removed', async () => {
    const manager = new ElephantAddonManager()
    const changed = vi.fn()
    manager.on('contribution:changed', changed)

    manager.register(createTestAddon({
      activate: vi.fn((ctx) => {
        ctx.addView({ id: 'sample-view' })
      })
    }))

    await manager.enable('test.addon')
    await manager.disable('test.addon')

    expect(changed).toHaveBeenCalled()
    expect(manager.getContributionMap()).toEqual({})
  })

  it('cleans contributions and disposables when disabled', async () => {
    const manager = new ElephantAddonManager()
    const dispose = vi.fn()
    const deactivate = vi.fn()
    const activate = vi.fn((ctx) => {
      ctx.addSidebarItem({ id: 'library', title: 'Library' })
      ctx.addDisposable(dispose)
    })

    manager.register(createTestAddon({ activate, deactivate }))
    await manager.enable('test.addon')
    await manager.disable('test.addon')

    expect(deactivate).toHaveBeenCalledOnce()
    expect(dispose).toHaveBeenCalledOnce()
    expect(manager.getContributions(ADDON_EXTENSION_POINTS.sidebarItems)).toEqual([])
    expect(manager.get('test.addon').status).toBe(ADDON_STATUS.disabled)
  })

  it('cleans partial contributions when activation throws', async () => {
    const logger = { info: vi.fn(), warn: vi.fn(), error: vi.fn() }
    const manager = new ElephantAddonManager({ logger })
    const dispose = vi.fn()

    manager.register(createTestAddon({
      activate: vi.fn((ctx) => {
        ctx.addStatusBarItem({ id: 'partial-status' })
        ctx.addDisposable(dispose)
        throw new Error('boom')
      })
    }))

    await expect(manager.enable('test.addon')).rejects.toThrow('boom')
    expect(dispose).toHaveBeenCalledOnce()
    expect(manager.getContributionMap()).toEqual({})
    expect(manager.get('test.addon')).toMatchObject({
      enabled: false,
      status: ADDON_STATUS.error,
      error: { message: 'boom' }
    })
    expect(logger.error).toHaveBeenCalledOnce()
  })
})
