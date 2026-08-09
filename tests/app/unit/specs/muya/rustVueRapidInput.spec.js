import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, ref } from 'vue'

import { RustMuyaRuntimeEditor } from '../../../../../Elephant/frontend/src/renderer/src/muya'

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
const selectionAt = (offset) => ({
  anchor: { node: 3, offset_utf16: offset },
  focus: { node: 3, offset_utf16: offset }
})

const browserEvent = (data) => {
  const event = new Event('beforeinput', { bubbles: true, cancelable: true })
  Object.defineProperties(event, {
    inputType: { configurable: true, value: 'insertText' },
    data: { configurable: true, value: data }
  })
  return event
}

const delayedEngine = () => {
  let revision = 0
  let text = 'alpha'
  let selection = selectionAt(5)
  let snapshotCalls = 0

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
    snapshot_json: vi.fn(async () => {
      snapshotCalls += 1
      const captured = snapshot()
      // The first call mounts the runtime. A later early sync deliberately
      // resolves after a newer sync, reproducing the stale v-model/remount race.
      const delay = snapshotCalls === 2 ? 30 : snapshotCalls > 2 ? 1 : 0
      if (delay) await new Promise((resolve) => setTimeout(resolve, delay))
      return response('snapshot', captured)
    }),
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
        selection = selectionAt(start + inserted.length)
        return response('update', {
          ...snapshot(),
          patches: [{ type: 'replace_text', node: 3, range: { start, end }, inserted }]
        })
      }
      throw new Error(`Unexpected command ${request.command.type}`)
    })
  }
}

const settle = async (milliseconds = 0) => {
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, milliseconds))
  await nextTick()
}

describe('Rust Vue editor rapid input synchronization', () => {
  afterEach(() => document.body.replaceChildren())

  it('keeps exact input order and does not remount on stale internal model emissions', async () => {
    const markdown = ref('alpha')
    const runtimeRef = ref(null)
    const factory = vi.fn(async () => delayedEngine())
    const app = createApp({
      setup: () => () => h(RustMuyaRuntimeEditor, {
        modelValue: markdown.value,
        factory,
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
    runtimeRef.value.renderer.restoreSelection(selectionAt(5))
    for (const character of 'XYZ123') editor.dispatchEvent(browserEvent(character))

    await runtimeRef.value.inputController.idle()
    await settle(80)

    expect(editor.textContent).toBe('alphaXYZ123')
    expect(markdown.value).toBe('alphaXYZ123')
    expect(factory).toHaveBeenCalledTimes(1)
    expect(document.querySelector('.muya-rust-runtime-error')).toBeNull()
    app.unmount()
  })
})
