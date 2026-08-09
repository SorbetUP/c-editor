import { beforeEach, describe, expect, it, vi } from 'vitest'

import { MuyaRustDomRenderer } from '../../../../../Elephant/frontend/src/muya/lib/rust/domRenderer'
import { MuyaRustInputController } from '../../../../../Elephant/frontend/src/muya/lib/rust/inputController'

const snapshot = () => ({
  markdown: 'alpha',
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
        children: [],
        kind: { layer: 'inline', value: { type: 'text', value: 'alpha' } },
        source: null
      }
    ]
  },
  revision: 0,
  selection: {
    anchor: { node: 3, offset_utf16: 0 },
    focus: { node: 3, offset_utf16: 0 }
  },
  can_undo: false,
  can_redo: false,
  composition_active: false
})

const browserEvent = (type, values = {}) => {
  const event = new Event(type, { bubbles: true, cancelable: true })
  for (const [key, value] of Object.entries(values)) {
    Object.defineProperty(event, key, { configurable: true, value })
  }
  return event
}

const clipboardData = (text, html = '') => ({
  getData: (type) => {
    if (type === 'text/plain') return text
    if (type === 'text/html') return html
    return ''
  }
})

const fakeBridge = () => ({
  setSelection: vi.fn(async () => {}),
  dispatch: vi.fn(async () => {})
})

describe('MuyaRustInputController', () => {
  let container
  let renderer
  let bridge
  let controller

  beforeEach(() => {
    container = document.createElement('section')
    document.body.replaceChildren(container)
    renderer = new MuyaRustDomRenderer(container)
    renderer.applySnapshot(snapshot())
    renderer.restoreSelection({
      anchor: { node: 3, offset_utf16: 2 },
      focus: { node: 3, offset_utf16: 2 }
    })
    bridge = fakeBridge()
    controller = new MuyaRustInputController(container, bridge, renderer).attach()
  })

  it('intercepts beforeinput and synchronizes selection before dispatch', async () => {
    const event = browserEvent('beforeinput', { inputType: 'insertText', data: 'X' })
    container.dispatchEvent(event)
    await controller.idle()

    expect(event.defaultPrevented).toBe(true)
    expect(bridge.setSelection).toHaveBeenCalledWith({
      anchor: { node: 3, offset_utf16: 2 },
      focus: { node: 3, offset_utf16: 2 }
    })
    expect(bridge.dispatch).toHaveBeenCalledWith({ type: 'insert_text', text: 'X' })
    expect(bridge.setSelection.mock.invocationCallOrder[0]).toBeLessThan(
      bridge.dispatch.mock.invocationCallOrder[0]
    )
  })

  it('owns plain Return on keydown when WebKit omits beforeinput', async () => {
    const event = browserEvent('keydown', { key: 'Enter', isComposing: false })
    container.dispatchEvent(event)
    await controller.idle()

    expect(event.defaultPrevented).toBe(true)
    expect(bridge.setSelection).toHaveBeenCalledWith({
      anchor: { node: 3, offset_utf16: 2 },
      focus: { node: 3, offset_utf16: 2 }
    })
    expect(bridge.dispatch).toHaveBeenCalledWith({ type: 'insert_paragraph' })
  })

  it('does not turn an IME confirmation or modified Return into a paragraph', async () => {
    const composing = browserEvent('keydown', {
      key: 'Enter',
      isComposing: true
    })
    const modified = browserEvent('keydown', {
      key: 'Enter',
      metaKey: true,
      isComposing: false
    })

    container.dispatchEvent(composing)
    container.dispatchEvent(modified)
    await controller.idle()

    expect(composing.defaultPrevented).toBe(false)
    expect(modified.defaultPrevented).toBe(false)
    expect(bridge.dispatch).not.toHaveBeenCalled()
  })

  it('routes plain clipboard Markdown through one paste command', async () => {
    const event = browserEvent('paste', {
      clipboardData: clipboardData('one\n\ntwo')
    })
    container.dispatchEvent(event)
    await controller.idle()

    expect(event.defaultPrevented).toBe(true)
    expect(bridge.setSelection).toHaveBeenCalledWith({
      anchor: { node: 3, offset_utf16: 2 },
      focus: { node: 3, offset_utf16: 2 }
    })
    expect(bridge.dispatch).toHaveBeenCalledWith({
      type: 'paste_markdown',
      markdown: 'one\n\ntwo'
    })
  })

  it('normalizes semantic clipboard HTML before paste', async () => {
    const event = browserEvent('paste', {
      clipboardData: clipboardData(
        'bold and soft',
        '<p><strong>bold</strong> and <em>soft</em></p>'
      )
    })
    container.dispatchEvent(event)
    await controller.idle()

    expect(bridge.dispatch).toHaveBeenCalledWith({
      type: 'paste_markdown',
      markdown: '**bold** and *soft*'
    })
  })

  it('normalizes unordered and ordered HTML lists', async () => {
    container.dispatchEvent(browserEvent('paste', {
      clipboardData: clipboardData('one\ntwo', '<ul><li>one</li><li>two</li></ul>')
    }))
    await controller.idle()
    expect(bridge.dispatch).toHaveBeenLastCalledWith({
      type: 'paste_markdown',
      markdown: '- one\n- two'
    })

    container.dispatchEvent(browserEvent('paste', {
      clipboardData: clipboardData('one\ntwo', '<ol><li>one</li><li>two</li></ol>')
    }))
    await controller.idle()
    expect(bridge.dispatch).toHaveBeenLastCalledWith({
      type: 'paste_markdown',
      markdown: '1. one\n2. two'
    })
  })

  it('normalizes a language-tagged preformatted code block', async () => {
    container.dispatchEvent(browserEvent('paste', {
      clipboardData: clipboardData(
        'console.log(1)',
        '<pre><code class="language-js">console.log(1)</code></pre>'
      )
    }))
    await controller.idle()

    expect(bridge.dispatch).toHaveBeenLastCalledWith({
      type: 'paste_markdown',
      markdown: '```js\nconsole.log(1)\n```'
    })
  })

  it('normalizes an HTML table to minimal GFM Markdown', async () => {
    container.dispatchEvent(browserEvent('paste', {
      clipboardData: clipboardData(
        'A\tB\none\ttwo',
        '<table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>one</td><td>two</td></tr></tbody></table>'
      )
    }))
    await controller.idle()

    expect(bridge.dispatch).toHaveBeenLastCalledWith({
      type: 'paste_markdown',
      markdown: '| A | B |\n| --- | --- |\n| one | two |'
    })
  })

  it('normalizes an HTML image to Markdown', async () => {
    container.dispatchEvent(browserEvent('paste', {
      clipboardData: clipboardData('alt', '<p><img src="image.png" alt="alt"></p>')
    }))
    await controller.idle()

    expect(bridge.dispatch).toHaveBeenLastCalledWith({
      type: 'paste_markdown',
      markdown: '![alt](image.png)'
    })
  })

  it('replaces the active IME range when composition text changes', async () => {
    container.dispatchEvent(browserEvent('compositionstart'))
    container.dispatchEvent(
      browserEvent('beforeinput', { inputType: 'insertCompositionText', data: 'に' })
    )
    container.dispatchEvent(
      browserEvent('beforeinput', { inputType: 'insertCompositionText', data: '日本' })
    )
    container.dispatchEvent(browserEvent('compositionend', { data: '日本' }))
    await controller.idle()

    expect(bridge.dispatch.mock.calls.map(([command]) => command)).toEqual([
      { type: 'begin_composition' },
      { type: 'update_composition', text: 'に' },
      { type: 'update_composition', text: '日本' },
      { type: 'commit_composition' }
    ])
    expect(bridge.setSelection.mock.calls).toEqual([
      [
        {
          anchor: { node: 3, offset_utf16: 2 },
          focus: { node: 3, offset_utf16: 2 }
        }
      ],
      [
        {
          anchor: { node: 3, offset_utf16: 2 },
          focus: { node: 3, offset_utf16: 2 }
        }
      ],
      [
        {
          anchor: { node: 3, offset_utf16: 2 },
          focus: { node: 3, offset_utf16: 3 }
        }
      ]
    ])
  })

  it('cancels an active composition with Escape', async () => {
    container.dispatchEvent(browserEvent('compositionstart'))
    const event = browserEvent('keydown', { key: 'Escape' })
    container.dispatchEvent(event)
    await controller.idle()

    expect(event.defaultPrevented).toBe(true)
    expect(bridge.dispatch).toHaveBeenLastCalledWith({ type: 'cancel_composition' })
  })
})
