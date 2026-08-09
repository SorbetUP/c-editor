import { describe, expect, it, vi } from 'vitest'

import { useRuntimeImageToolbar } from '../../../../../Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeImageToolbarState'

describe('Rust runtime image toolbar state', () => {
  const image = {
    image: 42,
    source: 'old.png',
    alt: 'old',
    title: 'Old',
    rect: { left: 10, top: 20 }
  }

  it('opens on a copied image descriptor and closes explicitly', () => {
    const toolbar = useRuntimeImageToolbar({})

    toolbar.open(image)
    expect(toolbar.state.active).toEqual(image)
    expect(toolbar.state.active).not.toBe(image)

    toolbar.close()
    expect(toolbar.state.active).toBeNull()
  })

  it('applies a replacement through Rust then closes', async () => {
    const handlers = { replace: vi.fn(async () => true) }
    const toolbar = useRuntimeImageToolbar(handlers)
    toolbar.open(image)
    const replacement = { ...image, source: 'new.png', alt: 'new', title: 'New' }

    await toolbar.apply(replacement)

    expect(handlers.replace).toHaveBeenCalledWith(replacement)
    expect(toolbar.state.active).toBeNull()
  })

  it('deletes through Rust then closes', async () => {
    const handlers = { remove: vi.fn(async () => true) }
    const toolbar = useRuntimeImageToolbar(handlers)
    toolbar.open(image)

    await toolbar.remove(image)

    expect(handlers.remove).toHaveBeenCalledWith(image)
    expect(toolbar.state.active).toBeNull()
  })

  it('keeps the toolbar open when file selection is cancelled', async () => {
    const handlers = { chooseReplacement: vi.fn(async () => false) }
    const toolbar = useRuntimeImageToolbar(handlers)
    toolbar.open(image)

    await toolbar.chooseFile(image)

    expect(toolbar.state.active).toEqual(image)
  })
})
