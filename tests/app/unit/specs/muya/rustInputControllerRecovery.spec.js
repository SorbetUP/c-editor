import { describe, expect, it, vi } from 'vitest'

import { ElephantRustDomRenderer } from '../../../../../Elephant/frontend/src/renderer/src/editor-rust/domRenderer'
import { ElephantRustInputController } from '../../../../../Elephant/frontend/src/renderer/src/editor-rust/inputController'

const snapshot = (nodeId, revision = 0, value = 'alpha') => ({
  root: 1,
  revision,
  document: {
    root: 1,
    nodes: [
      { id: 1, parent: null, children: [2], kind: { layer: 'document' }, source: null },
      { id: 2, parent: 1, children: [nodeId], kind: { layer: 'block', value: { type: 'paragraph' } }, source: null },
      { id: nodeId, parent: 2, children: [], kind: { layer: 'inline', value: { type: 'text', value } }, source: null }
    ]
  },
  selection: {
    anchor: { node: nodeId, offset_utf16: 0 },
    focus: { node: nodeId, offset_utf16: 0 }
  },
  can_undo: false,
  can_redo: false,
  composition_active: false
})

const event = (type, values = {}) => {
  const result = new Event(type, { bubbles: true, cancelable: true })
  for (const [key, value] of Object.entries(values)) {
    Object.defineProperty(result, key, { configurable: true, value })
  }
  return result
}

describe('Elephant Rust input selection recovery', () => {
  it('refreshes the authoritative caret when a queued input holds a detached node id', async () => {
    const container = document.createElement('section')
    document.body.replaceChildren(container)
    const renderer = new ElephantRustDomRenderer(container)
    renderer.applySnapshot(snapshot(3))
    renderer.restoreSelection({
      anchor: { node: 3, offset_utf16: 2 },
      focus: { node: 3, offset_utf16: 2 }
    })

    const bridge = {
      selection: snapshot(8, 1).selection,
      setSelection: vi.fn(async () => {}),
      snapshot: vi.fn(async () => snapshot(8, 1)),
      dispatch: vi.fn(async () => {})
    }
    const controller = new ElephantRustInputController(container, bridge, renderer).attach()

    container.dispatchEvent(event('keydown', { key: 'Enter', isComposing: false }))
    renderer.applySnapshot(snapshot(8, 1))
    await controller.idle()

    expect(bridge.snapshot).toHaveBeenCalledOnce()
    expect(bridge.setSelection).toHaveBeenCalledWith(snapshot(8, 1).selection)
    expect(bridge.dispatch).toHaveBeenCalledWith({ type: 'insert_paragraph' })
  })
})
