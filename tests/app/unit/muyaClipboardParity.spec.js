import { describe, expect, it } from 'vitest'

import { clipboardPayloadToMarkdown, copyMarkdownAndHtml, pastedHtmlToMarkdown, tableToMarkdown } from '../../../Elephant/frontend/src/renderer/src/muya/clipboardRuntime.js'

describe('Muya clipboard parity matrix', () => {
  it('uses plain text when HTML is absent', () => {
    expect(clipboardPayloadToMarkdown({ text: 'hello' })).toBe('hello')
  })

  it('converts common rich HTML into markdown', () => {
    const html = '<p><strong>Bold</strong> <em>Em</em> <code>x</code></p><a href="https://example.com">site</a><img src="pic.png" alt="Alt">'
    const markdown = pastedHtmlToMarkdown(html)
    expect(markdown).toContain('**Bold**')
    expect(markdown).toContain('*Em*')
    expect(markdown).toContain('`x`')
    expect(markdown).toContain('[site](https://example.com)')
    expect(markdown).toContain('![Alt](pic.png)')
  })

  it('converts table HTML into markdown tables', () => {
    const markdown = tableToMarkdown('<tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr>')
    expect(markdown).toBe('| A | B |\n| - | - |\n| 1 | 2 |')
  })

  it('returns markdown and rendered HTML during copy', () => {
    const payload = copyMarkdownAndHtml('# Title', (markdown) => `<h1>${markdown}</h1>`)
    expect(payload.markdown).toBe('# Title')
    expect(payload.html).toBe('<h1># Title</h1>')
  })
})
