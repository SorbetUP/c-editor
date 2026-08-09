import { describe, expect, it } from 'vitest'
import { JSDOM } from 'jsdom'

import { createMuyaFullEditorRuntime, domToMarkdown } from '../../../Elephant/frontend/src/renderer/src/muya/index.js'

describe('Muya live rendering runtime', () => {
  it('converts live DOM text edits back into Markdown and rerenders block structure', () => {
    const dom = new JSDOM('<div id="editor"></div>')
    globalThis.document = dom.window.document
    globalThis.getSelection = dom.window.getSelection.bind(dom.window)
    const root = dom.window.document.getElementById('editor')
    const runtime = createMuyaFullEditorRuntime(root, 'plain')
    const paragraph = root.querySelector('p')
    paragraph.textContent = '# Live title'
    runtime.renderLiveNow()
    expect(runtime.markdown).toBe('# Live title')
    expect(root.querySelector('h1')?.textContent).toBe('Live title')
  })

  it('keeps normal paragraph edits live without source textarea interaction', () => {
    const dom = new JSDOM('<div id="editor"></div>')
    globalThis.document = dom.window.document
    globalThis.getSelection = dom.window.getSelection.bind(dom.window)
    const root = dom.window.document.getElementById('editor')
    const runtime = createMuyaFullEditorRuntime(root, 'hello')
    root.querySelector('p').textContent = 'hello live world'
    runtime.renderLiveNow()
    expect(runtime.markdown).toBe('hello live world')
    expect(runtime.html).toContain('hello live world')
  })

  it('extracts Markdown from the rendered DOM tree', () => {
    const dom = new JSDOM('<div id="editor"><h2 data-muya-block="heading">Title</h2><p data-muya-block="paragraph">Body</p></div>')
    const root = dom.window.document.getElementById('editor')
    expect(domToMarkdown(root)).toBe('## Title\n\nBody')
  })
})
