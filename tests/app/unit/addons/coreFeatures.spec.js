import { describe, expect, it, vi } from 'vitest'
import { ADDON_EXTENSION_POINTS, createAddonHostRuntime, ElephantAddonManager } from '@/addons'
import { activateCoreFeature } from '@/addons/coreFeatures'

const createManifest = (id = 'com.example.additive-api') => ({
  id,
  name: 'Additive API',
  version: '1.0.0',
  apiVersion: 1,
  source: 'builtin',
  defaultEnabled: false
})

describe('core feature runtime', () => {
  it('registers contributions without creating addon records and disposes them atomically', async () => {
    const manager = new ElephantAddonManager({ logger: { info: vi.fn(), warn: vi.fn(), error: vi.fn() } })
    const disposed = vi.fn()
    const feature = {
      id: 'core.example',
      activate(ctx) {
        expect(ctx.source).toBe('core')
        ctx.addAction({ id: 'core.example.run', title: 'Run', run: vi.fn() })
        ctx.addSettingsSection({ id: 'core.example.settings', section: 'addons', fields: [] })
        return disposed
      }
    }

    const handle = await activateCoreFeature(manager, feature)
    expect(manager.list()).toEqual([])
    expect(manager.coreFeatures.get('core.example')).toBe(handle)
    expect(manager.getContributions(ADDON_EXTENSION_POINTS.actions))
      .toEqual([expect.objectContaining({ source: 'core', coreFeatureId: 'core.example' })])
    expect(manager.getContributions(ADDON_EXTENSION_POINTS.settingsSections)).toHaveLength(1)

    handle.dispose()
    expect(disposed).toHaveBeenCalledOnce()
    expect(manager.coreFeatures.has('core.example')).toBe(false)
    expect(manager.getContributions(ADDON_EXTENSION_POINTS.actions)).toEqual([])
    expect(manager.getContributions(ADDON_EXTENSION_POINTS.settingsSections)).toEqual([])
  })

  it('returns the same handle for duplicate activation attempts', async () => {
    const manager = new ElephantAddonManager()
    const activate = vi.fn()
    const feature = { id: 'core.singleton', activate }
    const first = await activateCoreFeature(manager, feature)
    const second = await activateCoreFeature(manager, feature)
    expect(first).toBe(second)
    expect(activate).toHaveBeenCalledOnce()
  })

  it('rolls back contributions when activation fails', async () => {
    const manager = new ElephantAddonManager()
    const feature = {
      id: 'core.failure',
      activate(ctx) {
        ctx.addAction({ id: 'core.failure.run', title: 'Run' })
        throw new Error('activation failed')
      }
    }

    await expect(activateCoreFeature(manager, feature)).rejects.toThrow('activation failed')
    expect(manager.coreFeatures.has('core.failure')).toBe(false)
    expect(manager.getContributions(ADDON_EXTENSION_POINTS.actions)).toEqual([])
  })
})

describe('additive host addon API', () => {
  it('keeps legacy helpers and exposes scoped namespaced contributions', async () => {
    const addonHost = createAddonHostRuntime()
    let context
    const manager = new ElephantAddonManager({ addonHost })
    manager.register({
      manifest: createManifest(),
      activate(value) {
        context = value
        value.addAction({ id: 'legacy.action', title: 'Legacy', run: () => 'legacy' })
        value.api.commands.register({ id: 'modern.action', title: 'Modern', run: () => 'modern' })
      }
    })
    manager.register({
      manifest: createManifest('com.example.other'),
      activate(value) {
        value.addAction({ id: 'other.action', title: 'Other', run: () => 'other' })
      }
    })

    await manager.enable('com.example.additive-api')
    await manager.enable('com.example.other')

    expect(typeof context.addView).toBe('function')
    expect(typeof context.addEditorFooterItem).toBe('function')
    expect(context.api.version).toBe(1)
    expect(context.api.ids.qualify('refresh')).toBe('com.example.additive-api.refresh')
    expect(context.api.ids.owns('com.example.additive-api.refresh')).toBe(true)
    expect(context.api.capabilities.supports(ADDON_EXTENSION_POINTS.editorFooterItems)).toBe(true)
    expect(context.api.contributions.list(ADDON_EXTENSION_POINTS.actions)).toHaveLength(2)
    expect(context.api.commands.get('other.action')).toBeNull()
    expect(await context.api.commands.execute('legacy.action')).toBe('legacy')
    expect(await context.api.commands.execute('modern.action')).toBe('modern')
    expect(addonHost.get('addon.api.com.example.additive-api')).toBe(context.api)
  })

  it('registers batches atomically and rejects unknown public extension points', async () => {
    let context
    const manager = new ElephantAddonManager()
    manager.register({ manifest: createManifest(), activate(value) { context = value } })
    await manager.enable('com.example.additive-api')

    context.api.contributions.registerMany({
      [ADDON_EXTENSION_POINTS.sidebarItems]: [
        { id: 'nav.one', title: 'One' },
        { id: 'nav.two', title: 'Two' }
      ],
      [ADDON_EXTENSION_POINTS.statusBarItems]: { id: 'status.one', title: 'Status' }
    })

    expect(context.api.contributions.list()).toHaveLength(3)
    expect(context.api.contributions.has(ADDON_EXTENSION_POINTS.sidebarItems, 'nav.one')).toBe(true)
    expect(() => context.api.contributions.register('unknown.area', { id: 'bad' }))
      .toThrow('Unknown addon extension point')
  })

  it('scopes events, storage, resources and lifecycle cleanup to the addon', async () => {
    const addonHost = createAddonHostRuntime()
    const listener = vi.fn()
    const dispose = vi.fn()
    const onAbort = vi.fn()
    let context
    let signal
    const manager = new ElephantAddonManager({ addonHost })
    manager.register({
      manifest: createManifest(),
      activate(value) {
        context = value
        signal = value.api.lifecycle.signal
        value.api.events.on('refresh', listener)
        value.api.lifecycle.onAbort(onAbort)
        value.api.lifecycle.addDisposable({ dispose })
        value.api.resources.provide('example.resource', { ready: true })
      }
    })

    await manager.enable('com.example.additive-api')
    await context.api.storage.set('counter', 1)
    await context.api.storage.update('counter', value => value + 1)
    expect(await context.api.storage.get('counter')).toBe(2)
    expect(await context.api.storage.has('counter')).toBe(true)
    expect(await context.api.storage.keys()).toEqual(['counter'])
    expect(context.api.resources.get('example.resource')).toEqual({ ready: true })

    context.api.events.emit('refresh', { source: 'test' })
    expect(listener).toHaveBeenCalledWith({ source: 'test' })
    expect(signal.aborted).toBe(false)

    await manager.disable('com.example.additive-api')

    context.api.events.emit('refresh', { source: 'after-disable' })
    expect(listener).toHaveBeenCalledTimes(1)
    expect(signal.aborted).toBe(true)
    expect(onAbort).toHaveBeenCalledTimes(1)
    expect(dispose).toHaveBeenCalledTimes(1)
    expect(addonHost.has('example.resource')).toBe(false)
    expect(addonHost.has('addon.api.com.example.additive-api')).toBe(false)
  })
})
