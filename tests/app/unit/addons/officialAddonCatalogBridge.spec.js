import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

import { installOfficialAddonCatalogBridge } from '../../../../Elephant/frontend/src/renderer/src/addons/officialAddonCatalogBridge'
import { useAddonsStore } from '../../../../Elephant/frontend/src/renderer/src/store/addons'

describe('official addon catalogue bridge', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    globalThis.__TAURI__ = {
      core: {
        invoke: vi.fn(async (command) => {
          if (command === 'tauri_official_addons_catalog_list') return null
          throw new Error(`Unexpected command: ${command}`)
        })
      }
    }
  })

  afterEach(() => {
    delete globalThis.__TAURI__
  })

  it('treats a null native catalogue as an empty collection instead of rendering an error banner', async () => {
    const store = useAddonsStore()
    store.loadAddonCatalog = vi.fn(async () => [])

    expect(installOfficialAddonCatalogBridge()).toBe(true)
    await expect(store.loadAddonCatalog()).resolves.toEqual([])

    expect(store.catalog).toEqual([])
    expect(store.catalogError).toBe('No addon catalogue is available.')
    expect(globalThis.__TAURI__.core.invoke).toHaveBeenCalledWith(
      'tauri_official_addons_catalog_list',
      {}
    )
  })
})
