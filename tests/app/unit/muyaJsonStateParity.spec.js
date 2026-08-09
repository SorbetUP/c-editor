import { describe, expect, it } from 'vitest'
import { JSDOM } from 'jsdom'

import { jsonStateToHtml, jsonStateToMarkdown, markdownToJsonState, renderJsonStateIntoDom } from '../../../Elephant/frontend/src/renderer/src/muya/jsonStateRuntime.js'

describe('Muya JSON state parity matrix', () => {
  it('keeps empty documents writable', () => {
    const state = markdownToJsonState('')
    expect(state.blocks).toHaveLength(1)
    expect(state.blocks[0].type).toBe('paragraph')
    expect(jsonStateToHtml(state)).toContain('data-muya-block="paragraph"')
  })

  it('roundtrips common block types to markdown', () => {
    const markdown = ['# Title', '', '> Quote', '', '- [x] Done', '', '- Item', '', '1. Ordered'].join('\n')
    const state = markdownToJsonState(markdown)
    expect(state.blocks.map((block) => block.type)).toEqual(['heading', 'blockquote', 'task_list_item', 'list_item', 'list_item'])
    expect(jsonStateToMarkdown(state)).toContain('# Title')
    expect(jsonStateToMarkdown(state)).toContain('- [x] Done')
  })

  it('renders tables code fences and math blocks', () => {
    const markdown = ['| A | B |', '| - | - |', '| 1 | 2 |', '', '```js', 'x()', '```', '', '$$', 'x+1', '$$'].join('\n')
    const state = markdownToJsonState(markdown)
    const types = state.blocks.map((block) => block.type)
    expect(types).toContain('table')
    expect(types).toContain('code_fence')
    expect(types).toContain('math_block')
    expect(jsonStateToHtml(state)).toContain('data-muya-block="table"')
  })

  it('writes editable DOM roots', () => {
    const dom = new JSDOM('<div id="root"></div>')
    const root = dom.window.document.getElementById('root')
    renderJsonStateIntoDom(root, markdownToJsonState('# Title'), dom.window.document)
    expect(root.getAttribute('contenteditable')).toBe('true')
    expect(root.querySelector('h1')?.textContent).toBe('Title')
  })
})
