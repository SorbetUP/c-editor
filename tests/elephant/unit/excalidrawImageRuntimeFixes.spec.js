import { beforeEach, describe, expect, it, vi } from 'vitest'

const busMock = vi.hoisted(() => ({
  emit: vi.fn(),
  on: vi.fn(),
  off: vi.fn()
}))

const convertFileSrcMock = vi.hoisted(() => vi.fn((pathname) => `asset://localhost${pathname}`))

vi.mock('@/bus', () => ({
  default: busMock
}))

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: convertFileSrcMock
}))

import { installExcalidrawImageRuntimeFixes } from '@/platform/excalidrawImageRuntimeFixes'

const setImageComplete = (img, complete) => {
  Object.defineProperty(img, 'complete', {
    configurable: true,
    value: complete
  })
}

beforeEach(() => {
  window.__ELEPHANT_EXCALIDRAW_IMAGE_RUNTIME_FIXES__?.dispose?.()
  document.body.innerHTML = ''
  delete window.__ELEPHANT_EXCALIDRAW_IMAGE_RUNTIME_FIXES__
  busMock.emit.mockClear()
  busMock.on.mockClear()
  busMock.off.mockClear()
  convertFileSrcMock.mockClear()
  window.fileUtils = {
    pathExistsSync: vi.fn(() => true),
    readFile: vi.fn(async() => new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]))
  }
  window.requestAnimationFrame = (callback) => {
    callback()
    return 0
  }
})

describe('Excalidraw image runtime fixes', () => {
  it('keeps a loading object URL intact until the Excalidraw asset finishes loading', () => {
    const container = document.createElement('div')
    container.className = 'ag-image-container'

    const img = document.createElement('img')
    img.setAttribute('data-src', 'file:///vault/.assets/excalidraw-demo.png')
    img.setAttribute('src', 'blob:loading-object-url')
    setImageComplete(img, false)

    container.appendChild(img)
    document.body.appendChild(container)

    const runtime = installExcalidrawImageRuntimeFixes(window)
    expect(typeof runtime.dispose).toBe('function')
    expect(img.getAttribute('src')).toBe('blob:loading-object-url')

    setImageComplete(img, true)
    img.dispatchEvent(new Event('load', { bubbles: true }))

    expect(convertFileSrcMock).toHaveBeenCalledWith('/vault/.assets/excalidraw-demo.png')
    expect(img.getAttribute('src')).toContain('asset://localhost')
    expect(img.dataset.elephantExcalidrawPath).toBe('/vault/.assets/excalidraw-demo.png')
    expect(busMock.on).toHaveBeenCalledWith('invalidate-image-cache', expect.any(Function))
  })

  it('does not rewrite a local Excalidraw image that already finished loading as data URL', () => {
    const container = document.createElement('div')
    container.className = 'ag-image-container'

    const img = document.createElement('img')
    img.setAttribute('data-src', 'file:///vault/.assets/excalidraw-demo.png')
    img.setAttribute('src', 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+Xl9sAAAAASUVORK5CYII=')
    img.dataset.localImageLoaded = 'true'
    setImageComplete(img, true)

    container.appendChild(img)
    document.body.appendChild(container)

    const runtime = installExcalidrawImageRuntimeFixes(window)
    expect(typeof runtime.dispose).toBe('function')
    expect(img.getAttribute('src')).toContain('data:image/png;base64,')
    expect(convertFileSrcMock).not.toHaveBeenCalled()
  })

  it('rebuilds a failed Excalidraw image container from the local asset on disk', async() => {
    const container = document.createElement('div')
    container.className = 'ag-image ag-image-fail'
    container.dataset.imageSrc = 'file:///vault/.assets/excalidraw-demo.png'
    container.dataset.imageDomsrc = 'blob:http://127.0.0.1:1420/failed'
    container.dataset.imageError = 'local-object-url-load-error'

    const imageContainer = document.createElement('span')
    imageContainer.className = 'ag-image-container'
    container.appendChild(imageContainer)
    document.body.appendChild(container)

    const runtime = installExcalidrawImageRuntimeFixes(window)
    expect(typeof runtime.dispose).toBe('function')
    await Promise.resolve()
    await Promise.resolve()

    const img = container.querySelector('img')
    expect(img).toBeTruthy()
    expect(img.dataset.localPath).toBe('/vault/.assets/excalidraw-demo.png')
    expect(img.dataset.resolvedSrc).toMatch(/^data:image\/png;base64,/)
    expect(container.classList.contains('ag-image-fail')).toBe(false)
    expect(container.classList.contains('ag-image-success')).toBe(true)
  })

  it('removes injected controls and listeners when the addon is disabled', () => {
    const container = document.createElement('div')
    container.className = 'ag-image-container'
    const img = document.createElement('img')
    img.setAttribute('data-src', 'file:///vault/.assets/excalidraw-demo.png')
    setImageComplete(img, true)
    container.appendChild(img)
    document.body.appendChild(container)

    const runtime = installExcalidrawImageRuntimeFixes(window)
    expect(container.querySelector('.en-excalidraw-edit-button')).not.toBeNull()

    runtime.dispose()

    expect(container.querySelector('.en-excalidraw-edit-button')).toBeNull()
    expect(busMock.off).toHaveBeenCalledWith('invalidate-image-cache', expect.any(Function))
    expect(window.__ELEPHANT_EXCALIDRAW_IMAGE_RUNTIME_FIXES__).toBeUndefined()
  })
})
