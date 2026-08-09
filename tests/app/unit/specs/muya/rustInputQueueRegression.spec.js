import { describe, expect, it, vi } from 'vitest'

import { MuyaRustInputController } from '../../../../../Elephant/frontend/src/muya/lib/rust/inputController'

const beforeInput = (data) => {
  const event = new Event('beforeinput', { bubbles: true, cancelable: true })
  Object.defineProperties(event, {
    inputType: { configurable: true, value: 'insertText' },
    data: { configurable: true, value: data }
  })
  return event
}

const selectionAt = (offset) => ({
  anchor: { node: 3, offset_utf16: offset },
  focus: { node: 3, offset_utf16: offset }
})

describe('Rust editor queued input selection', () => {
  it('continues queued characters from each previous Rust selection update', async () => {
    const container = document.createElement('section')
    const renderer = { logical: {} }
    const bridge = {
      selection: selectionAt(0),
      setSelection: vi.fn(async (selection) => {
        bridge.selection = selection
      }),
      dispatch: vi.fn(async () => {
        bridge.selection = selectionAt(bridge.selection.focus.offset_utf16 + 1)
      })
    }
    const controller = new MuyaRustInputController(container, bridge, renderer).attach()
    // All browser events observe the same stale DOM caret because they arrive
    // before the asynchronous Rust update has patched the DOM.
    controller.readSelection = vi.fn(() => selectionAt(2))

    container.dispatchEvent(beforeInput('A'))
    container.dispatchEvent(beforeInput('B'))
    container.dispatchEvent(beforeInput('C'))
    await controller.idle()

    expect(bridge.dispatch.mock.calls.map(([command]) => command)).toEqual([
      { type: 'insert_text', text: 'A' },
      { type: 'insert_text', text: 'B' },
      { type: 'insert_text', text: 'C' }
    ])
    expect(bridge.setSelection.mock.calls.map(([selection]) => selection.focus.offset_utf16)).toEqual([
      2,
      3,
      4
    ])
  })
})
