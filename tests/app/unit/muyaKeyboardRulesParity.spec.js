import { describe, expect, it } from 'vitest'

import { applyKeyboardRuleToMarkdown, handleMuyaKeydown } from '../../../Elephant/frontend/src/renderer/src/muya/inputRulesRuntime.js'

describe('Muya keyboard rules parity', () => {
  it('continues bullets tasks and ordered lists on Enter', () => {
    expect(applyKeyboardRuleToMarkdown('- [x] done', 'Enter')).toBe('- [x] done\n- [ ] ')
    expect(applyKeyboardRuleToMarkdown('- item', 'Enter')).toBe('- item\n- ')
    expect(applyKeyboardRuleToMarkdown('3. item', 'Enter')).toBe('3. item\n4. ')
  })

  it('indents and outdents on Tab', () => {
    expect(applyKeyboardRuleToMarkdown('a\nb', 'Tab')).toBe('a\n  b')
    expect(applyKeyboardRuleToMarkdown('a\n  b', 'Tab', { shiftKey: true })).toBe('a\nb')
  })

  it('prevents default only when the runtime applies a rule', () => {
    const runtime = { markdown: '- item', setMarkdown(value) { this.markdown = value } }
    const event = { key: 'Enter', preventDefault() { this.prevented = true } }
    expect(handleMuyaKeydown(runtime, event)).toBe(true)
    expect(event.prevented).toBe(true)
    expect(runtime.markdown).toBe('- item\n- ')
  })
})
