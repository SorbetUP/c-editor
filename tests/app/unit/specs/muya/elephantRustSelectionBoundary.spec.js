import { afterEach, describe, expect, it, vi } from 'vitest'

import { ElephantRustInputController } from '../../../../../Elephant/frontend/src/renderer/src/editor-rust/inputController'

afterEach(() => {
  globalThis.window?.getSelection?.()?.removeAllRanges()
})

describe('Elephant Rust selection boundary', () => {
  it('ignores a browser selection owned by toolbar UI outside the Rust document', () => {
    const container = document.createElement('section')
    const toolbarButton = document.createElement('button')
    toolbarButton.textContent = 'New note'
    document.body.replaceChildren(container, toolbarButton)

    const range = document.createRange()
    range.selectNodeContents(toolbarButton)
    const browserSelection = window.getSelection()
    browserSelection.removeAllRanges()
    browserSelection.addRange(range)

    const onError = vi.fn()
    const controller = new ElephantRustInputController(
      container,
      { dispatch: vi.fn(), setSelection: vi.fn() },
      { logical: {}, ownerDocument: document },
      { onError }
    )

    expect(controller.readSelection()).toBeNull()
    expect(onError).not.toHaveBeenCalled()
  })
})
