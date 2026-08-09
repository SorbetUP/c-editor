import { describe, expect, it } from 'vitest'
import {
  createEmptyExcalidrawScene,
  ensureExcalidrawName,
  ensurePngName,
  getExcalidrawBackgroundColor,
  getExcalidrawPreviewPath,
  getExcalidrawScenePath,
  getExcalidrawSidecarPath,
  normalizeExcalidrawBaseName
} from 'common/elephantnote/excalidrawAssets'

describe('portable Excalidraw asset contract', () => {
  it('normalizes drawing and image output names', () => {
    expect(ensurePngName('sketch')).toBe('sketch.png')
    expect(ensurePngName('sketch.png')).toBe('sketch.png')
    expect(ensureExcalidrawName('scene')).toBe('scene.excalidraw')
    expect(ensureExcalidrawName('scene.excalidraw')).toBe('scene.excalidraw')
  })

  it('derives scene and preview sidecar paths without platform APIs', () => {
    expect(getExcalidrawScenePath('/vault/note/sketch.png')).toBe('/vault/note/sketch.excalidraw')
    expect(getExcalidrawScenePath('/vault/note/sketch')).toBe('/vault/note/sketch.excalidraw')
    expect(getExcalidrawSidecarPath('/vault/note/sketch.png')).toBe('/vault/note/sketch.excalidraw')
    expect(getExcalidrawPreviewPath('/vault/note/sketch.excalidraw')).toBe('/vault/note/sketch.png')
  })

  it('normalizes save base names and empty scene defaults', () => {
    expect(normalizeExcalidrawBaseName('diagram.excalidraw.png')).toBe('diagram')
    expect(normalizeExcalidrawBaseName('', 'fallback')).toBe('fallback')
    expect(getExcalidrawBackgroundColor('dark')).toBe('#121212')
    expect(createEmptyExcalidrawScene('light')).toMatchObject({
      elements: [],
      files: {},
      appState: {
        viewBackgroundColor: '#ffffff',
        exportBackground: true,
        exportEmbedScene: true
      }
    })
  })
})
