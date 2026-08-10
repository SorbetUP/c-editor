import { describe, expect, it, vi } from 'vitest'

const excalidrawMock = vi.hoisted(() => ({
  loadFromBlob: vi.fn(async() => ({ elements: [], appState: {}, files: {} }))
}))

vi.mock('@excalidraw/excalidraw', () => ({
  Excalidraw: function Excalidraw() {},
  default: {},
  loadFromBlob: excalidrawMock.loadFromBlob
}))
import {
  createInitialExcalidrawData,
  ensureExcalidrawName,
  ensurePngName,
  getExcalidrawPreviewPath,
  getExcalidrawSidecarPath,
  resolveExcalidrawModule
} from '@/elephantnote/services/excalidraw'

describe('ElephantNote Excalidraw helpers', () => {
  it('normalizes drawing and image output names', () => {
    expect(ensurePngName('sketch')).toBe('sketch.png')
    expect(ensurePngName('sketch.png')).toBe('sketch.png')
    expect(ensureExcalidrawName('scene')).toBe('scene.excalidraw')
    expect(ensureExcalidrawName('scene.excalidraw')).toBe('scene.excalidraw')
  })

  it('derives the editable sidecar path from generated previews', () => {
    const originalPath = window.path
    window.path = {
      extname: (pathname) => pathname.match(/\.[^/.]+$/)?.[0] || ''
    }

    expect(getExcalidrawSidecarPath('/vault/note/sketch.png')).toBe('/vault/note/sketch.excalidraw')
    expect(getExcalidrawSidecarPath('/vault/note/sketch')).toBe('/vault/note/sketch.excalidraw')
    expect(getExcalidrawPreviewPath('/vault/note/sketch.excalidraw')).toBe('/vault/note/sketch.png')

    window.path = originalPath
  })

  it('starts with an empty scene when no source file is selected', async () => {
    const data = await createInitialExcalidrawData({
      blob: null,
      theme: 'dark'
    })

    expect(data.elements).toEqual([])
    expect(data.files).toEqual({})
    expect(data.appState.viewBackgroundColor).toBe('#121212')
  })

  it('restores an empty JSON sidecar as a scene instead of treating it as an image', async () => {
    const scene = {
      type: 'excalidraw',
      version: 2,
      source: 'elephant-test',
      elements: [],
      appState: { viewBackgroundColor: '#ffffff' },
      files: {}
    }

    const data = await createInitialExcalidrawData({
      blob: new Blob([JSON.stringify(scene)], { type: 'application/vnd.excalidraw+json' }),
      theme: 'light'
    })

    expect(data).toEqual(scene)
    expect(excalidrawMock.loadFromBlob).not.toHaveBeenCalled()
  })

  it('restores a PNG preview with an omitted MIME type through Excalidraw', async () => {
    const scene = { elements: [{ id: 'embedded-image' }], appState: {}, files: {} }
    excalidrawMock.loadFromBlob.mockImplementationOnce(async(blob) => {
      expect(blob.type).toBe('image/png')
      return scene
    })

    const data = await createInitialExcalidrawData({
      blob: new Blob(['png-payload']),
      fileName: 'drawing.png',
      theme: 'light'
    })

    expect(data).toEqual(scene)
  })

  it('builds an image scene when a PNG has no embedded Excalidraw scene', async () => {
    const originalImage = globalThis.Image
    globalThis.Image = class {
      naturalWidth = 64
      naturalHeight = 32
      imageSource = ''
      get src() {
        return this.imageSource
      }
      set src(_value) {
        this.imageSource = _value
        queueMicrotask(() => this.onload?.())
      }
    }
    try {
      const data = await createInitialExcalidrawData({
        blob: new Blob(['png-payload'], { type: 'image/png' }),
        theme: 'light'
      })

      const [image] = data.elements
      expect(image.type).toBe('image')
      expect(data.files[image.fileId].dataURL).toMatch(/^data:image\/png;base64,/)
    } finally {
      globalThis.Image = originalImage
    }
  })

  it('resolves both native and default Excalidraw package exports', () => {
    const nativeModule = { Excalidraw: function NativeExcalidraw() {} }
    const defaultModule = { default: { Excalidraw: function DefaultExcalidraw() {} } }

    expect(resolveExcalidrawModule(nativeModule)).toMatchObject(nativeModule)
    expect(resolveExcalidrawModule(defaultModule)).toMatchObject(defaultModule.default)
    expect(() => resolveExcalidrawModule({ default: {} })).toThrow('Excalidraw could not be loaded')
  })

  it('keeps data helpers when the package splits them across module namespaces', () => {
    const loadFromBlob = () => Promise.resolve({ elements: [], files: {} })
    const module = {
      Excalidraw: function Excalidraw() {},
      default: { loadFromBlob }
    }

    expect(resolveExcalidrawModule(module).loadFromBlob).toBe(loadFromBlob)
  })
})
