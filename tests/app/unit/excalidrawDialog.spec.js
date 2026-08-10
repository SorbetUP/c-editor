import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, isProxy, nextTick, ref } from 'vue'

const rootRenderCalls = vi.hoisted(() => [])

vi.mock('react-dom/client', () => ({
  createRoot: vi.fn(() => ({
    render: vi.fn((element) => {
      rootRenderCalls.push(element)
      element.type(element.props)
    }),
    unmount: vi.fn()
  }))
}))

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

const createDialogApp = (render) => createApp({ render })

const mountDialog = async({ askNameOnClose = false } = {}) => {
  const host = document.createElement('div')
  document.body.append(host)
  const onClose = vi.fn()
  const app = createDialogApp(() => h(ExcalidrawDialog, {
      theme: 'Apple Dark',
      fileName: 'smoke.png',
      askNameOnClose,
      onClose
    }))
  app.mount(host)
  await nextTick()
  await vi.waitFor(() => expect(document.querySelector('[data-testid="excalidraw-dialog"]')).not.toBeNull())
  return { app, host, onClose }
}

afterEach(() => {
  document.body.innerHTML = ''
  document.body.className = ''
  rootRenderCalls.length = 0
  vi.restoreAllMocks()
})

describe('Excalidraw dialog exit contract', () => {
  it('keeps React-owned values raw across a Vue-driven theme rerender', async() => {
    const initialDataProxyChecks = []
    const apiProxyChecks = []
    const fakeApi = {
      getAppState() {
        apiProxyChecks.push(isProxy(this))
        return {}
      },
      updateScene: vi.fn()
    }
    const fakeModule = {
      Excalidraw: (props) => {
        initialDataProxyChecks.push(isProxy(props.initialData))
        props.excalidrawAPI(fakeApi)
        return null
      }
    }
    loadExcalidrawModule.mockResolvedValueOnce(fakeModule)

    const theme = ref('light')
    const host = document.createElement('div')
    document.body.append(host)
    const app = createDialogApp(() => h(ExcalidrawDialog, {
        theme: theme.value,
        fileName: 'smoke.png'
      }))
    app.mount(host)

    await vi.waitFor(() => expect(initialDataProxyChecks.length).toBeGreaterThan(0))
    theme.value = 'dark'
    await nextTick()
    await vi.waitFor(() => expect(fakeApi.updateScene).toHaveBeenCalled())

    expect(rootRenderCalls).toHaveLength(1)
    expect(initialDataProxyChecks).toEqual([false])
    expect(apiProxyChecks).toEqual([false])
    app.unmount()
  })

  it('asks for a name before closing a new library drawing', async() => {
    const { app, onClose } = await mountDialog({ askNameOnClose: true })
    document.querySelector('[data-testid="excalidraw-close"]').dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
    await nextTick()
    expect(document.querySelector('[data-testid="excalidraw-name-prompt"]')).not.toBeNull()
    expect(onClose).not.toHaveBeenCalled()
    app.unmount()
  })

  it('emits close from the visible cancel control and clears fullscreen state on unmount', async() => {
    const { app, onClose } = await mountDialog()
    const close = document.querySelector('[data-testid="excalidraw-close"]')
    expect(close).not.toBeNull()
    expect(close.querySelector('svg')).not.toBeNull()
    expect(close.textContent).not.toContain('✕')
    const save = document.querySelector('[data-testid="excalidraw-save"]')
    expect(save?.querySelector('svg')).not.toBeNull()
    expect(save?.textContent).not.toContain('✓')
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
