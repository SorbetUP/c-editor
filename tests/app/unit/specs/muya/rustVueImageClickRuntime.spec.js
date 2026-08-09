import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick, ref } from 'vue'

import { RustMuyaRuntimeEditor } from '../../../../../Elephant/frontend/src/renderer/src/muya'

const response = (type, payload) => JSON.stringify({ type, payload })

const snapshot = () => ({
  markdown: '![diagram](image.png "Architecture")',
  document: {
    root: 1,
    nodes: [
      { id: 1, parent: null, children: [2], kind: { layer: 'document' }, source: null },
      {
        id: 2,
        parent: 1,
        children: [3, 4],
        kind: { layer: 'block', value: { type: 'paragraph' } },
        source: null
      },
      {
        id: 3,
        parent: 2,
        children: [],
        kind: {
          layer: 'inline',
          value: {
            type: 'image',
            source: 'image.png',
            alt: 'diagram',
            title: 'Architecture'
          }
        },
        source: null
      },
      {
        id: 4,
        parent: 2,
        children: [],
        kind: { layer: 'inline', value: { type: 'text', value: '' } },
        source: null
      }
    ]
  },
  revision: 0,
  selection: {
    anchor: { node: 4, offset_utf16: 0 },
    focus: { node: 4, offset_utf16: 0 }
  },
  can_undo: false,
  can_redo: false,
  composition_active: false
})

const settle = async () => {
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await nextTick()
}

describe('Rust Muya Vue image clicks', () => {
  afterEach(() => document.body.replaceChildren())

  it('forwards stable image metadata through the mounted runtime', async () => {
    const runtimeRef = ref(null)
    const onImageClick = vi.fn()
    const engine = {
      snapshot_json: vi.fn(async () => response('snapshot', snapshot())),
      handle_json: vi.fn(async () => {
        throw new Error('No protocol command expected for an image click.')
      })
    }
    const app = createApp({
      setup: () => () => h(RustMuyaRuntimeEditor, {
        modelValue: snapshot().markdown,
        factory: async () => engine,
        onImageClick,
        onReady: (runtime) => { runtimeRef.value = runtime }
      })
    })
    const host = document.createElement('div')
    document.body.appendChild(host)
    app.mount(host)
    await settle()

    const image = runtimeRef.value.renderer.element(3)
    image.dispatchEvent(new MouseEvent('click', { bubbles: true }))

    expect(onImageClick).toHaveBeenCalledWith(expect.objectContaining({
      image: 3,
      source: 'image.png',
      alt: 'diagram',
      title: 'Architecture'
    }))
    expect(engine.handle_json).not.toHaveBeenCalled()
    app.unmount()
  })
})
