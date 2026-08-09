import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick } from 'vue'

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key) => key })
}))

vi.mock('elephant-front/services/excalidraw', () => ({
  loadExcalidrawModule: vi.fn(async() => ({ Excalidraw: () => null })),
  createInitialExcalidrawData: vi.fn(async() => ({ elements: [], appState: {}, files: {} })),
  exportExcalidrawBlob: vi.fn(),
  exportExcalidrawSceneBlob: vi.fn(),
  ensurePngName: (value) => `${value}.png`
}))

import ExcalidrawDialog from '../../../Elephant/frontend/app/components/editor/ExcalidrawDialog.vue'
import { loadExcalidrawModule } from 'elephant-front/services/excalidraw'

const mountDialog = async() => {
  const host = document.createElement('div')
  document.body.append(host)
  const onClose = vi.fn()
  const app = createApp({
    render: () => h(ExcalidrawDialog, {
      theme: 'Apple Dark',
      fileName: 'smoke.png',
      onClose
    })
  })
  app.mount(host)
  await nextTick()
  await vi.waitFor(() => expect(document.querySelector('[data-testid="excalidraw-dialog"]')).not.toBeNull())
  return { app, host, onClose }
}

afterEach(() => {
  document.body.innerHTML = ''
  document.body.className = ''
  vi.restoreAllMocks()
})

describe('Excalidraw dialog exit contract', () => {
  it('emits close from the visible cancel control and clears fullscreen state on unmount', async() => {
    const { app, onClose } = await mountDialog()
    const close = document.querySelector('[data-testid="excalidraw-close"]')
    expect(close).not.toBeNull()
    expect(document.body.classList.contains('en-excalidraw-open')).toBe(true)

    close.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
    expect(onClose).toHaveBeenCalledTimes(1)

    app.unmount()
    expect(document.body.classList.contains('en-excalidraw-open')).toBe(false)
  })

  it('emits close from Escape without throwing', async() => {
    const { app, onClose } = await mountDialog()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(onClose).toHaveBeenCalledTimes(1)

    app.unmount()
  })

  it('records initialization failures in the shared debug log', async() => {
    loadExcalidrawModule.mockRejectedValueOnce(new Error('load failed'))
    const { app } = await mountDialog()
    await vi.waitFor(() => expect(document.querySelector('[role="alert"]')).not.toBeNull())
    expect(window.__ELEPHANT_DEBUG_LOGS__).toEqual(expect.arrayContaining([
      expect.objectContaining({ message: '[excalidraw-dialog] initialization failed', level: 'error' })
    ]))
    app.unmount()
  })
})
