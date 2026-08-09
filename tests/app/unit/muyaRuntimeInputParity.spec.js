import { describe, expect, it } from 'vitest'
import { JSDOM } from 'jsdom'

import { createMuyaFullEditorRuntime, domToMarkdown, readMuyaRuntimeMode } from '../../../Elephant/frontend/src/renderer/src/muya/index.js'

describe('Muya runtime input parity', () => {
  it('uses the Rust editor by default in Tauri', () => {
    expect(readMuyaRuntimeMode({ __MARKTEXT_RUNTIME__: 'tauri' })).toBe('rust')
  })

  it('renders empty documents as writable content', () => {
    const dom = new JSDOM('<div id="root"></div>')
    globalThis.document = dom.window.document
    globalThis.getSelection = dom.window.getSelection.bind(dom.window)
    const root = dom.window.document.getElementById('root')
    createMuyaFullEditorRuntime(root, '')
    expect(root.getAttribute('contenteditable')).toBe('true')
    expect(root.textContent).toBe('')
    expect(root.querySelector('[data-muya-block="paragraph"]')).toBeTruthy()
  })

  it('preserves text nodes created by browser editing', () => {
    const dom = new JSDOM('<div id="root">hello</div>')
    const root = dom.window.document.getElementById('root')
    expect(domToMarkdown(root)).toBe('hello')
  })
})
