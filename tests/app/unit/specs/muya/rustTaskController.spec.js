import { beforeEach, describe, expect, it, vi } from 'vitest'

import { MuyaRustDomRenderer } from '../../../../../Elephant/frontend/src/muya/lib/rust/domRenderer'
import { MuyaRustInputController } from '../../../../../Elephant/frontend/src/muya/lib/rust/inputController'

const selection = {
  anchor: { node: 5, offset_utf16: 0 },
  focus: { node: 5, offset_utf16: 0 }
}

const snapshot = {
  markdown: '- [ ] alpha',
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
        kind: { layer: 'block', value: { type: 'list', kind: 'task', start: null } },
        source: null
      },
      {
        id: 3,
        parent: 2,
        children: [4],
        kind: { layer: 'block', value: { type: 'list_item', checked: false } },
        source: null
      },
      {
        id: 4,
        parent: 3,
        children: [5],
        kind: { layer: 'block', value: { type: 'paragraph' } },
        source: null
      },
      {
        id: 5,
        parent: 4,
        children: [],
        kind: { layer: 'inline', value: { type: 'text', value: 'alpha' } },
        source: null
      }
    ]
  }
}

describe('Muya Rust task checkbox controller', () => {
  let container
  let renderer
  let bridge

  beforeEach(() => {
    container = document.createElement('section')
    document.body.replaceChildren(container)
    renderer = new MuyaRustDomRenderer(container)
    renderer.applySnapshot(snapshot)
    bridge = {
      setSelection: vi.fn(async () => {}),
      dispatch: vi.fn(async () => {})
    }
  })

  it('renders task state and dispatches a Rust mutation on click', async () => {
    const controller = new MuyaRustInputController(container, bridge, renderer).attach()
    const checkbox = container.querySelector('[data-muya-rust-task-checkbox]')
    expect(checkbox).not.toBeNull()
    expect(checkbox.checked).toBe(false)

    checkbox.click()
    await controller.idle()

    expect(bridge.dispatch).toHaveBeenCalledWith({
      type: 'set_task_checked',
      item: 3,
      checked: true,
      auto_check: false
    })
  })

  it('forwards the Muya autoCheck option to Rust', async () => {
    const controller = new MuyaRustInputController(container, bridge, renderer, {
      autoCheck: true
    }).attach()
    container.querySelector('[data-muya-rust-task-checkbox]').click()
    await controller.idle()

    expect(bridge.dispatch).toHaveBeenCalledWith({
      type: 'set_task_checked',
      item: 3,
      checked: true,
      auto_check: true
    })
  })

  it('refreshes the checkbox shell without duplicating intrinsic controls', () => {
    renderer.applyPatches(
      [
        {
          type: 'set_block_kind',
          node: 3,
          kind: { type: 'list_item', checked: true }
        }
      ],
      { revision: 1 }
    )

    const boxes = container.querySelectorAll('[data-muya-rust-task-checkbox]')
    expect(boxes).toHaveLength(1)
    expect(boxes[0].checked).toBe(true)
    expect(container.textContent).toContain('alpha')
  })
})
