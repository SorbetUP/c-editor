import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { installAddonSystem } from '@/addons'
import { useAddonsStore } from '@/store/addons'

const createAppStub = () => ({
  provide: vi.fn(),
  config: { globalProperties: {} }
})

describe('installAddonSystem', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('provides the manager through Vue and global properties', () => {
    const app = createAppStub()

    const manager = installAddonSystem(app, { addons: [] })

    expect(app.provide).toHaveBeenCalledOnce()
    expect(app.config.globalProperties.$addons).toBe(manager)
    expect(window.__ELEPHANT_ADDONS__).toBe(manager)
  })

  it('registers supplied addons without enabling disabled addons', () => {
    const app = createAppStub()

    const manager = installAddonSystem(app, {
      addons: [
        {
          manifest: {
            id: 'sample.addon',
            name: 'Sample Addon'
          },
          activate: vi.fn()
        }
      ]
    })

    expect(manager.list()).toHaveLength(1)
    expect(manager.get('sample.addon').enabled).toBe(false)
  })

  it('installs the Pinia mirror when Pinia is provided', () => {
    const app = createAppStub()
    const pinia = createPinia()
    setActivePinia(pinia)

    const manager = installAddonSystem(app, {
      pinia,
      addons: [
        {
          manifest: {
            id: 'pinia.addon',
            name: 'Pinia Addon'
          },
          activate: vi.fn()
        }
      ]
    })

    const store = useAddonsStore(pinia)
    expect(store.installed).toBe(true)
    expect(store.manager).toBe(manager)
    expect(store.items).toHaveLength(1)
  })
})
