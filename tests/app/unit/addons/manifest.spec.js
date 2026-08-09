import { describe, expect, it } from 'vitest'
import { ADDON_API_VERSION, normalizeAddonManifest } from '@/addons'

describe('normalizeAddonManifest', () => {
  it('normalizes a minimal manifest', () => {
    const manifest = normalizeAddonManifest({
      id: 'demo.addon',
      name: 'Demo Addon'
    })

    expect(manifest).toMatchObject({
      id: 'demo.addon',
      name: 'Demo Addon',
      version: '0.0.0',
      apiVersion: ADDON_API_VERSION,
      defaultEnabled: false
    })
    expect(manifest.permissions).toEqual([])
  })

  it('rejects unsafe ids', () => {
    expect(() => normalizeAddonManifest({ id: '../bad-addon' })).toThrow(/id/)
    expect(() => normalizeAddonManifest({ id: 'BadAddon' })).toThrow(/id/)
  })

  it('rejects unsupported api versions', () => {
    expect(() => normalizeAddonManifest({
      id: 'demo.future',
      apiVersion: ADDON_API_VERSION + 1
    })).toThrow(/Unsupported addon apiVersion/)
  })
})
