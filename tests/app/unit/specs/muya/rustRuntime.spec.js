import { beforeEach, describe, expect, it, vi } from 'vitest'

import { initializeExperimentalRustRuntime } from '../../../../../Elephant/frontend/src/muya/lib/rust/runtime'

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
  const requests = []

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
    requests,
    snapshot_json: vi.fn(async () => response('snapshot', snapshot())),
    handle_json: vi.fn(async (raw) => {
      const request = JSON.parse(raw)
      requests.push(request)
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

describe('initializeExperimentalRustRuntime', () => {
  let container

  beforeEach(() => {
    container = document.createElement('section')
    document.body.replaceChildren(container)
  })

  it('renders and edits inside an explicit isolated container', async () => {
    const engine = fakeEngine()
    const runtime = await initializeExperimentalRustRuntime(
      { markdown: 'alpha' },
      { factory: async () => engine, domContainer: container, captureInput: true },
      vi.fn()
    )

    expect(container.getAttribute('contenteditable')).toBe('true')
    expect(runtime.renderer.element(3).textContent).toBe('alpha')
    runtime.renderer.restoreSelection({
      anchor: { node: 3, offset_utf16: 5 },
      focus: { node: 3, offset_utf16: 5 }
    })

    const event = browserEvent('beforeinput', { inputType: 'insertText', data: '!' })
    container.dispatchEvent(event)
    await runtime.inputController.idle()

    expect(event.defaultPrevented).toBe(true)
    expect(runtime.renderer.element(3).textContent).toBe('alpha!')
    expect(engine.requests.map((request) => request.command.type)).toEqual([
      'set_selection',
      'insert_text'
    ])
  })

  it('forwards image files after synchronizing the Rust selection', async () => {
    const engine = fakeEngine()
    const onFileDrop = vi.fn(async () => {})
    const runtime = await initializeExperimentalRustRuntime(
      { markdown: 'alpha' },
      {
        factory: async () => engine,
        domContainer: container,
        captureInput: true,
        onFileDrop
      },
      vi.fn()
    )
    const files = [{ name: 'image.png', type: 'image/png' }]
    const dataTransfer = fileTransfer(files)
    const drop = browserEvent('drop', { dataTransfer })

    container.dispatchEvent(drop)
    await runtime.inputController.idle()

    expect(drop.defaultPrevented).toBe(true)
    expect(engine.requests[0].command.type).toBe('set_selection')
    expect(onFileDrop).toHaveBeenCalledWith(files)
  })

  it('detaches input handling when the runtime is destroyed', async () => {
    const engine = fakeEngine()
    const runtime = await initializeExperimentalRustRuntime(
      { markdown: 'alpha' },
      { factory: async () => engine, domContainer: container, captureInput: true },
      vi.fn()
    )
    runtime.destroy()

    container.dispatchEvent(browserEvent('beforeinput', { inputType: 'insertText', data: '!' }))
    await runtime.inputController.idle()
    expect(engine.requests).toEqual([])
  })

  it('supports protocol-only shadow mode without a DOM container', async () => {
    const runtime = await initializeExperimentalRustRuntime(
      { markdown: 'alpha' },
      { factory: async () => fakeEngine() },
      vi.fn()
    )

    expect(runtime.shadow).toBe(true)
    expect(runtime.renderer).toBeNull()
    expect(runtime.inputController).toBeNull()
  })
})
