import { describe, expect, it } from 'vitest'

import {
  applyKeyboardRuleToMarkdown,
  defaultMuyaRuntimeMode,
  handleMuyaKeydown,
  pastedHtmlToMarkdown,
  readMuyaRuntimeMode,
  renderPreviewBlock,
  tableToMarkdown
} from '../../../Elephant/frontend/src/renderer/src/muya/index.js'

describe('advanced Muya runtime behavior', () => {
  it('uses the Rust editor by default in production runtimes', () => {
    expect(defaultMuyaRuntimeMode({ __MARKTEXT_RUNTIME__: 'tauri' })).toBe('rust')
    expect(defaultMuyaRuntimeMode({ __MARKTEXT_RUNTIME__: 'electron' })).toBe('rust')
    expect(readMuyaRuntimeMode({ __MARKTEXT_RUNTIME__: 'tauri' })).toBe('rust')
  })

  it('applies keyboard input rules for list continuation and indentation', () => {
    expect(applyKeyboardRuleToMarkdown('- [x] done', 'Enter')).toBe('- [x] done\n- [ ] ')
    expect(applyKeyboardRuleToMarkdown('- item', 'Enter')).toBe('- item\n- ')
    expect(applyKeyboardRuleToMarkdown('1. item', 'Enter')).toBe('1. item\n2. ')
    expect(applyKeyboardRuleToMarkdown('- item', 'Tab')).toBe('  - item')
    expect(applyKeyboardRuleToMarkdown('  - item', 'Tab', { shiftKey: true })).toBe('- item')
  })

  it('handles keyboard rules through a runtime facade', () => {
    const runtime = {
      markdown: '- item',
      setMarkdown(value) { this.markdown = value }
    }
    const event = { key: 'Enter', preventDefault() { this.prevented = true } }
    expect(handleMuyaKeydown(runtime, event)).toBe(true)
    expect(event.prevented).toBe(true)
    expect(runtime.markdown).toBe('- item\n- ')
  })

  it('converts stronger rich paste cases from Word Notion and web HTML', () => {
    const html = '<table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table><p><strong>Bold</strong> <em>Em</em> <code>x</code></p><img src="pic.png" alt="Alt"><div data-notion-block-id="x">Notion</div>'
    const markdown = pastedHtmlToMarkdown(html)
    expect(markdown).toContain('| A | B |')
    expect(markdown).toContain('| 1 | 2 |')
    expect(markdown).toContain('**Bold**')
    expect(markdown).toContain('*Em*')
    expect(markdown).toContain('`x`')
    expect(markdown).toContain('![Alt](pic.png)')
    expect(markdown).toContain('Notion')
  })

  it('converts table fragments to markdown', () => {
    expect(tableToMarkdown('<tr><th>A</th></tr><tr><td>1</td></tr>')).toBe('| A |\n| - |\n| 1 |')
  })

  it('provides safe preview renderer fallbacks or real renderer outputs', async() => {
    const katex = await renderPreviewBlock({ type: 'math_block', text: 'x+1' })
    expect(katex.type).toBe('katex')
    expect(katex.html).toContain('x+1')

    const mermaid = await renderPreviewBlock({ type: 'code_fence', language: 'mermaid', text: 'graph TD;' })
    expect(mermaid.type).toBe('mermaid')
    expect(mermaid.html.length).toBeGreaterThan(0)
  })
})
