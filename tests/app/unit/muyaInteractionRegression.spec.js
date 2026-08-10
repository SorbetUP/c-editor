import { describe, expect, it } from 'vitest'
import { JSDOM } from 'jsdom'
import {
  clearPaddingOnlySelection,
  isPaddingOnlySelection
} from '../../../Elephant/frontend/src/renderer/src/muya/selectionRuntime.js'

describe('Muya editor interaction regressions', () => {
  it('rejects a collapsed browser range whose boundary is editor padding', () => {
    const dom = new JSDOM('<div id="editor"><p>Text</p></div>')
    const root = dom.window.document.getElementById('editor')
    const selection = dom.window.getSelection()
    const range = dom.window.document.createRange()
    range.setStart(root, 0)
    range.collapse(true)
    selection.addRange(range)

    expect(isPaddingOnlySelection(selection, root)).toBe(true)
    expect(clearPaddingOnlySelection(selection, root)).toBe(true)
    expect(selection.rangeCount).toBe(0)
  })

  it('keeps a real text selection intact', () => {
    const dom = new JSDOM('<div id="editor"><p>Text</p></div>')
    const root = dom.window.document.getElementById('editor')
    const text = root.querySelector('p').firstChild
    const selection = dom.window.getSelection()
    const range = dom.window.document.createRange()
    range.setStart(text, 0)
    range.setEnd(text, 4)
    selection.addRange(range)

    expect(isPaddingOnlySelection(selection, root)).toBe(false)
    expect(clearPaddingOnlySelection(selection, root)).toBe(false)
    expect(selection.toString()).toBe('Text')
  })
})
