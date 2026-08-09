import { readFileSync } from 'node:fs'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

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
import loadImageAsync from 'muya/lib/parser/render/renderInlines/loadImageAsync.js'

const pngBytes = () => new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13])
const updatedPngBytes = () => new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10, 1, 2, 3, 4])
const flushPromises = async (rounds = 8) => {
  for (let index = 0; index < rounds; index += 1) await Promise.resolve()
}

let runtime = null

it('keeps ordinary Markdown image loading independent from Excalidraw', () => {
  const loader = readFileSync(
    'Elephant/frontend/src/muya/lib/parser/render/renderInlines/loadImageAsync.js',
    'utf8'
  )
  expect(loader).toContain("from 'common/elephantnote/imageAssetContract'")
  expect(loader).not.toContain('EXCALIDRAW_ASSET_RE')
  expect(loader).toContain('isStandardMarkdownImagePath')
})

beforeEach(() => {
  document.body.innerHTML = ''
  window.__ELEPHANT_DEBUG_LOGS__ = []
  delete window.__ELEPHANT_EXCALIDRAW_IMAGE_RUNTIME_FIXES__
  delete window.__ELEPHANT_GET_ACTIVE_VAULT_PATH__
  busMock.emit.mockClear()
  busMock.on.mockClear()
  busMock.off.mockClear()
  convertFileSrcMock.mockClear()
  window.path = {
    join: (...parts) => parts
      .filter(Boolean)
      .join('/')
      .replace(/\/+/g, '/')
      .replace(':/', '://')
  }
  window.fileUtils = {
    pathExistsSync: vi.fn(() => false),
    stat: vi.fn(async () => ({ isFile: true, isDirectory: false, size: 12 })),
    readFile: vi.fn(async () => pngBytes())
  }
  window.requestAnimationFrame = (callback) => {
    callback()
    return 0
  }
  runtime = null
})

afterEach(() => {
  runtime?.dispose?.()
  runtime = null
  delete window.__ELEPHANT_GET_ACTIVE_VAULT_PATH__
})

describe('Excalidraw Tauri image loading', () => {
  it('repairs a nested-note drawing from the active vault even when the sync metadata cache is cold', async () => {
    window.__ELEPHANT_GET_ACTIVE_VAULT_PATH__ = () => '/vault'

    const failed = document.createElement('div')
    failed.className = 'ag-image ag-image-fail'
    failed.dataset.imageSrc = '../../.assets/excalidraw-demo.png'
    failed.dataset.imageError = 'local-file-not-found'

    const imageContainer = document.createElement('span')
    imageContainer.className = 'ag-image-container'
    failed.appendChild(imageContainer)
    document.body.appendChild(failed)

    runtime = installExcalidrawImageRuntimeFixes(window)
    await flushPromises()

    expect(window.fileUtils.pathExistsSync).toHaveBeenCalledWith('/vault/.assets/excalidraw-demo.png')
    expect(window.fileUtils.readFile).toHaveBeenCalledWith('/vault/.assets/excalidraw-demo.png')
    expect(failed.classList.contains('ag-image-fail')).toBe(false)
    expect(failed.classList.contains('ag-image-success')).toBe(true)

    const img = failed.querySelector('img')
    expect(img).toBeTruthy()
    expect(img.dataset.elephantExcalidrawPath).toBe('/vault/.assets/excalidraw-demo.png')
    expect(img.getAttribute('src')).toMatch(/^data:image\/png;base64,/)
    expect(failed.querySelector('.en-excalidraw-edit-button')).toBeTruthy()
    expect(window.__ELEPHANT_DEBUG_LOGS__).toEqual(expect.arrayContaining([
      expect.objectContaining({ message: '[excalidraw-image] failed image repair:success' }),
      expect.objectContaining({ message: '[excalidraw-image] local image read:success' })
    ]))

    const firstDataUrl = img.getAttribute('src')
    const invalidateHandler = busMock.on.mock.calls.find(([eventName]) => eventName === 'invalidate-image-cache')?.[1]
    expect(invalidateHandler).toBeTypeOf('function')
    window.fileUtils.readFile.mockResolvedValueOnce(updatedPngBytes())
    invalidateHandler()
    await flushPromises()

    expect(window.fileUtils.readFile).toHaveBeenCalledTimes(2)
    expect(img.getAttribute('src')).not.toBe(firstDataUrl)
    expect(window.__ELEPHANT_DEBUG_LOGS__).toEqual(expect.arrayContaining([
      expect.objectContaining({ message: '[excalidraw-image] cache refresh:success' })
    ]))
  })

  it('reads an absolute Excalidraw preview instead of rejecting a false pathExistsSync cache result', async () => {
    const wrapper = document.createElement('div')
    const imageText = document.createElement('span')
    imageText.className = 'ag-image-loading'
    wrapper.appendChild(imageText)
    document.body.appendChild(wrapper)

    const originalQuerySelector = document.querySelector.bind(document)
    document.querySelector = (selector) => {
      if (String(selector || '').startsWith('#')) return imageText
      return originalQuerySelector(selector)
    }

    try {
      const context = {
        loadImageMap: new Map(),
        urlMap: new Map()
      }

      const result = loadImageAsync.call(
        context,
        { src: '/vault/.assets/excalidraw-demo.png' },
        {},
        'ag-image',
        'ag-image'
      )

      await flushPromises()

      expect(result.domsrc).toMatch(/^file:\/\/\/vault\/\.assets\/excalidraw-demo\.png\?msec=/)
      expect(window.fileUtils.pathExistsSync).toHaveBeenCalledWith('/vault/.assets/excalidraw-demo.png')
      expect(window.fileUtils.readFile).toHaveBeenCalledWith('/vault/.assets/excalidraw-demo.png')

      const insertedImage = wrapper.querySelector('img')
      expect(insertedImage).toBeTruthy()
      expect(insertedImage.getAttribute('src')).toMatch(/^data:image\/png;base64,/)
      expect(insertedImage.dataset.localResolvedPath).toBe('/vault/.assets/excalidraw-demo.png')
      expect(window.__ELEPHANT_DEBUG_LOGS__).toEqual(expect.arrayContaining([
        expect.objectContaining({ message: '[image-loader] local read:success' })
      ]))
    } finally {
      document.querySelector = originalQuerySelector
    }
  })

  it('replaces a Tauri asset URL after the browser rejects an existing preview', async () => {
    window.__ELEPHANT_GET_ACTIVE_VAULT_PATH__ = () => '/vault'

    const wrapper = document.createElement('div')
    const img = document.createElement('img')
    img.dataset.src = '/vault/.assets/excalidraw-browser-error.png'
    img.src = 'asset://localhost/vault/.assets/excalidraw-browser-error.png'
    wrapper.appendChild(img)
    document.body.appendChild(wrapper)

    runtime = installExcalidrawImageRuntimeFixes(window)
    await flushPromises()
    img.dispatchEvent(new Event('error', { bubbles: true }))
    await flushPromises()

    expect(window.fileUtils.readFile).toHaveBeenCalledWith('/vault/.assets/excalidraw-browser-error.png')
    expect(img.dataset.localImageLoaded).toBe('true')
    expect(img.getAttribute('src')).toMatch(/^data:image\/png;base64,/)
    expect(window.__ELEPHANT_DEBUG_LOGS__).toEqual(expect.arrayContaining([
      expect.objectContaining({ message: '[excalidraw-image] cache refresh:success' })
    ]))
  })

  it('keeps authoritative stat probes and lifecycle diagnostics in the overlay implementation', () => {
    const overlaySource = readFileSync(
      'Elephant/frontend/src/renderer/src/addons/builtin/ui/ExcalidrawEditorOverlay.vue',
      'utf8'
    )
    const coreSource = readFileSync(
      'Elephant/frontend/src/renderer/src/addons/builtin/excalidraw.js',
      'utf8'
    )

    expect(overlaySource).toContain('const info = await window.fileUtils.stat(pathname)')
    expect(overlaySource).toContain('while (await pathExists(candidate))')
    expect(overlaySource).toContain('const imagePath = resolveDrawingPath(src)')
    expect(overlaySource).toContain("message: `[excalidraw-addon] ${message}`")
    expect(coreSource).toContain('globalThis.__ELEPHANT_GET_ACTIVE_VAULT_PATH__ = getActiveVaultPath')
  })
})
