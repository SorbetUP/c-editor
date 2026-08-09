import { beforeEach, describe, expect, it, vi } from 'vitest'

import { MuyaRustDomRenderer } from '../../../../../Elephant/frontend/src/muya/lib/rust/domRenderer'
import { MuyaRustInputController } from '../../../../../Elephant/frontend/src/muya/lib/rust/inputController'

const selection = {
  anchor: { node: 4, offset_utf16: 0 },
  focus: { node: 4, offset_utf16: 4 }
}

const snapshot = {
  markdown: '**bold**',
  revision: 0,
  selection,
  can_undo: false,
  can_redo: false,
  composition_active: false,
  document: {
    root: 1,
    nodes: [
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
        children: [4],
        kind: { layer: 'inline', value: { type: 'strong' } },
        source: null
      },
      {
        id: 4,
        parent: 3,
        children: [],
        kind: { layer: 'inline', value: { type: 'text', value: 'bold' } },
        source: null
      }
    ]
  }
}

const clipboardEvent = (type) => {
  const values = new Map()
  const event = new Event(type, { bubbles: true, cancelable: true })
  Object.defineProperty(event, 'clipboardData', {
    value: {
      setData: vi.fn((format, value) => values.set(format, value))
    }
  })
  event.stopPropagation = vi.fn()
  return { event, values }
}

describe('Muya Rust copy and cut controller', () => {
  let container
  let renderer
  let bridge
  let controller

  beforeEach(() => {
    container = document.createElement('section')
    document.body.replaceChildren(container)
    renderer = new MuyaRustDomRenderer(container)
    renderer.applySnapshot(snapshot)
    renderer.restoreSelection(selection)
    bridge = {
      setSelection: vi.fn(async () => {}),
      dispatch: vi.fn(async () => {})
    }
    controller = new MuyaRustInputController(container, bridge, renderer).attach()
  })

  it('writes native Elephant, Muya, Markdown, HTML and plain payloads', () => {
    const { event, values } = clipboardEvent('copy')
    container.dispatchEvent(event)

    expect(event.defaultPrevented).toBe(true)
    expect(values.get('application/x-elephant-markdown')).toBe('**bold**')
    expect(values.get('application/x-muya-markdown')).toBe('**bold**')
    expect(values.get('text/markdown')).toBe('**bold**')
    expect(values.get('text/plain')).toBe('bold')
    expect(values.get('text/html')).toContain('<strong')
    expect(bridge.dispatch).not.toHaveBeenCalled()
  })

  it('copies first and then deletes the selection through Rust', async () => {
    const { event, values } = clipboardEvent('cut')
    container.dispatchEvent(event)
    await controller.idle()

    expect(values.get('application/x-elephant-markdown')).toBe('**bold**')
    expect(bridge.setSelection).toHaveBeenCalledWith(selection)
    expect(bridge.dispatch).toHaveBeenCalledWith({ type: 'delete_backward' })
  })

  it('leaves a collapsed copy event to the browser', () => {
    renderer.restoreSelection({
      anchor: { node: 4, offset_utf16: 2 },
      focus: { node: 4, offset_utf16: 2 }
    })
    const { event } = clipboardEvent('copy')
    container.dispatchEvent(event)
    expect(event.defaultPrevented).toBe(false)
  })
})
