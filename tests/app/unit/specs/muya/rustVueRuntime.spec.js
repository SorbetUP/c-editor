import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, ref } from 'vue'

import {
  RustMuyaRuntimeEditor,
  isMuyaRuntimeActive,
  isMuyaRuntimeEnabled,
  isMuyaRustRuntime,
  readMuyaRuntimeMode
} from '../../../../../Elephant/frontend/src/renderer/src/muya'
import { createGlobalMuyaRuntimeBridge } from '../../../../../Elephant/frontend/src/renderer/src/muya/globalRuntimeBridge'

const nodeTree = (text) => [
  { id: 1, parent: null, children: [2], kind: { layer: 'document' }, source: null },
  {
    id: 2,
    parent: 1,
    children: [3],
    kind: { layer: 'block', value: { type: 'paragraph' } },
    source: null
  },
  {
    id: 3,
    parent: 2,
    children: [],
    kind: { layer: 'inline', value: { type: 'text', value: text } },
    source: null
  }
]

const response = (type, payload) => JSON.stringify({ type, payload })

const fakeEngine = () => {
  let revision = 0
  let text = 'alpha'
  let selection = {
    anchor: { node: 3, offset_utf16: 0 },
    focus: { node: 3, offset_utf16: 0 }
  }

  const snapshot = () => ({
    markdown: text,
    document: { root: 1, nodes: nodeTree(text) },
    revision,
    selection,
    can_undo: revision > 0,
    can_redo: false,
    composition_active: false
  })

  return {
    snapshot_json: vi.fn(async () => response('snapshot', snapshot())),
    handle_json: vi.fn(async (raw) => {
      const request = JSON.parse(raw)
      if (request.command.type === 'set_selection') {
        selection = request.command.selection
        return response('update', { ...snapshot(), patches: [] })
      }
      if (request.command.type === 'insert_text') {
        const start = Math.min(selection.anchor.offset_utf16, selection.focus.offset_utf16)
        const end = Math.max(selection.anchor.offset_utf16, selection.focus.offset_utf16)
        const inserted = request.command.text
        text = `${text.slice(0, start)}${inserted}${text.slice(end)}`
        revision += 1
        selection = {
          anchor: { node: 3, offset_utf16: start + inserted.length },
          focus: { node: 3, offset_utf16: start + inserted.length }
        }
        return response('update', {
          ...snapshot(),
          patches: [{ type: 'replace_text', node: 3, range: { start, end }, inserted }]
        })
      }
      throw new Error(`Unexpected command ${request.command.type}`)
    })
  }
}

const browserEvent = (type, values = {}) => {
  const event = new Event(type, { bubbles: true, cancelable: true })
  for (const [key, value] of Object.entries(values)) {
    Object.defineProperty(event, key, { configurable: true, value })
  }
  return event
}

const fileTransfer = (files) => ({
  types: ['Files'],
  files,
  dropEffect: 'none',
  getData: () => ''
})

const settle = async () => {
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await nextTick()
}

describe('Rust Muya Vue runtime', () => {
  afterEach(() => {
    document.body.replaceChildren()
  })

  it('recognizes rust as an explicit active runtime mode', () => {
    expect(readMuyaRuntimeMode({ __ELEPHANT_MUYA_RUNTIME_MODE__: 'rust' })).toBe('rust')
    expect(isMuyaRuntimeEnabled('rust')).toBe(true)
    expect(isMuyaRuntimeActive('rust')).toBe(true)
    expect(isMuyaRustRuntime('rust')).toBe(true)
  })

  it('notifies the application immediately when the mode changes', () => {
    const target = new EventTarget()
    target.Event = Event
    const listener = vi.fn()
    target.addEventListener('elephantnote:muya-runtime-mode-changed', listener)
    const bridge = createGlobalMuyaRuntimeBridge(target)

    expect(bridge.setMode('rust')).toBe('rust')
    expect(listener).toHaveBeenCalledTimes(1)
    expect(bridge.active()).toBe(true)
  })

  it('mounts the isolated Rust runtime and synchronizes edits to v-model', async () => {
    const markdown = ref('alpha')
    const runtimeRef = ref(null)
    const app = createApp({
      setup: () => () =>
        h(RustMuyaRuntimeEditor, {
          modelValue: markdown.value,
          factory: async () => fakeEngine(),
          'onUpdate:modelValue': (value) => {
            markdown.value = value
          },
          onReady: (runtime) => {
            runtimeRef.value = runtime
          }
        })
    })

    const host = document.createElement('div')
    document.body.appendChild(host)
    app.mount(host)
    await settle()

    const editor = document.querySelector('[data-testid="muya-rust-runtime-editor"]')
    expect(editor?.textContent).toBe('alpha')
    expect(runtimeRef.value).toBeTruthy()
    runtimeRef.value.renderer.restoreSelection({
      anchor: { node: 3, offset_utf16: 5 },
      focus: { node: 3, offset_utf16: 5 }
    })

    editor.dispatchEvent(browserEvent('beforeinput', { inputType: 'insertText', data: '!' }))
    await runtimeRef.value.inputController.idle()
    await settle()

    expect(editor.textContent).toBe('alpha!')
    expect(markdown.value).toBe('alpha!')
    app.unmount()
  })

  it('forwards file drops through the mounted Vue runtime', async () => {
    const runtimeRef = ref(null)
    const onFileDrop = vi.fn(async () => {})
    const app = createApp({
      setup: () => () =>
        h(RustMuyaRuntimeEditor, {
          modelValue: 'alpha',
          factory: async () => fakeEngine(),
          onFileDrop,
          onReady: (runtime) => {
            runtimeRef.value = runtime
          }
        })
    })
    const host = document.createElement('div')
    document.body.appendChild(host)
    app.mount(host)
    await settle()

    const editor = document.querySelector('[data-testid="muya-rust-runtime-editor"]')
    const files = [{ name: 'image.png', type: 'image/png' }]
    const drop = browserEvent('drop', { dataTransfer: fileTransfer(files) })
    editor.dispatchEvent(drop)
    await runtimeRef.value.inputController.idle()

    expect(drop.defaultPrevented).toBe(true)
    expect(onFileDrop).toHaveBeenCalledWith(files)
    app.unmount()
  })
})
