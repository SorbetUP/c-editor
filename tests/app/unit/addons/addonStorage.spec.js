import { describe, expect, it, vi } from 'vitest'
import { ElephantAddonManager } from '@/addons'

const createMemoryStorage = () => {
  const values = new Map()
  return {
    getItem: vi.fn((key) => values.has(key) ? values.get(key) : null),
    setItem: vi.fn((key, value) => values.set(key, String(value))),
    removeItem: vi.fn((key) => values.delete(key)),
    key: vi.fn((index) => [...values.keys()][index] || null),
    clear: vi.fn(() => values.clear()),
    get length() {
      return values.size
    },
    dump: () => Object.fromEntries(values.entries())
  }
}

const createStorageAddon = (id, testBody) => ({
  manifest: {
    id,
    name: `Storage ${id}`
  },
  activate: vi.fn(testBody)
})

describe('addon scoped storage', () => {
  it('persists JSON values under an addon-scoped namespace', async () => {
    const backend = createMemoryStorage()
    const manager = new ElephantAddonManager({ addonStorageBackend: backend })

    manager.register(createStorageAddon('alpha.addon', async (ctx) => {
      await ctx.storage.set('layout', { mode: 'compact', width: 42 })
      expect(await ctx.storage.get('layout')).toEqual({ mode: 'compact', width: 42 })
    }))

    await manager.enable('alpha.addon')

    expect(backend.dump()).toEqual({
      'elephantnote:addons:alpha.addon:layout': '{"mode":"compact","width":42}'
    })
  })

  it('isolates storage keys between addons', async () => {
    const backend = createMemoryStorage()
    const manager = new ElephantAddonManager({ addonStorageBackend: backend })

    manager.register(createStorageAddon('alpha.addon', async (ctx) => {
      await ctx.storage.set('shared', 'alpha')
    }))
    manager.register(createStorageAddon('beta.addon', async (ctx) => {
      await ctx.storage.set('shared', 'beta')
      expect(await ctx.storage.get('shared')).toBe('beta')
    }))

    await manager.enable('alpha.addon')
    await manager.enable('beta.addon')

    expect(backend.dump()).toEqual({
      'elephantnote:addons:alpha.addon:shared': '"alpha"',
      'elephantnote:addons:beta.addon:shared': '"beta"'
    })
  })

  it('supports default values, removal and namespace clearing', async () => {
    const backend = createMemoryStorage()
    const manager = new ElephantAddonManager({ addonStorageBackend: backend })

    manager.register(createStorageAddon('cleanup.addon', async (ctx) => {
      expect(await ctx.storage.get('missing', 'fallback')).toBe('fallback')
      await ctx.storage.set('a', 1)
      await ctx.storage.set('b', 2)
      await ctx.storage.remove('a')
      expect(await ctx.storage.get('a', null)).toBe(null)
      await ctx.storage.clear()
      expect(await ctx.storage.entries()).toEqual({})
    }))

    await manager.enable('cleanup.addon')
    expect(backend.dump()).toEqual({})
  })

  it('rejects unsafe storage keys before touching the backend', async () => {
    const backend = createMemoryStorage()
    const manager = new ElephantAddonManager({ addonStorageBackend: backend })

    manager.register(createStorageAddon('safe.addon', async (ctx) => {
      await expect(ctx.storage.set('../secret', true)).rejects.toThrow(/storage key/i)
    }))

    await expect(manager.enable('safe.addon')).resolves.toMatchObject({ enabled: true })
    expect(backend.setItem).not.toHaveBeenCalled()
  })
})
