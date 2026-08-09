import { beforeEach, describe, expect, it, vi } from 'vitest'

import { MuyaRustDomRenderer } from '../../../../../Elephant/frontend/src/muya/lib/rust/domRenderer'
import { MuyaRustInputController } from '../../../../../Elephant/frontend/src/muya/lib/rust/inputController'

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

describe('Muya Rust image click controller', () => {
  let container
  let renderer
  let bridge

  beforeEach(() => {
    container = document.createElement('section')
    document.body.replaceChildren(container)
    renderer = new MuyaRustDomRenderer(container)
    renderer.applySnapshot(snapshot())
    bridge = {
      setSelection: vi.fn(async () => {}),
      dispatch: vi.fn(async () => {})
    }
  })

  it('exposes stable image metadata without mutating the document', () => {
    const onImageClick = vi.fn()
    new MuyaRustInputController(container, bridge, renderer, { onImageClick }).attach()
    const image = renderer.element(3)
    image.getBoundingClientRect = vi.fn(() => ({
      left: 40,
      top: 80,
      right: 140,
      bottom: 160,
      width: 100,
      height: 80
    }))

    image.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))

    expect(onImageClick).toHaveBeenCalledWith({
      image: 3,
      source: 'image.png',
      alt: 'diagram',
      title: 'Architecture',
      rect: {
        left: 40,
        top: 80,
        right: 140,
        bottom: 160,
        width: 100,
        height: 80
      }
    })
    expect(bridge.dispatch).not.toHaveBeenCalled()
  })

  it('leaves image clicks untouched without an application callback', () => {
    new MuyaRustInputController(container, bridge, renderer).attach()
    const observer = vi.fn()
    container.addEventListener('click', observer)

    renderer.element(3).dispatchEvent(new MouseEvent('click', { bubbles: true }))

    expect(observer).toHaveBeenCalledTimes(1)
    expect(bridge.dispatch).not.toHaveBeenCalled()
  })
})
