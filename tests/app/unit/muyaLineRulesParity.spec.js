import { describe, expect, it } from 'vitest'

import { applyLineInputRule } from '../../../Elephant/frontend/src/renderer/src/muya/inputRulesRuntime.js'

describe('Muya line rules parity', () => {
  it('classifies heading quote list task and paragraph lines', () => {
    expect(applyLineInputRule('# title').type).toBe('heading')
    expect(applyLineInputRule('### title').type).toBe('heading')
    expect(applyLineInputRule('> quote').type).toBe('blockquote')
    expect(applyLineInputRule('- item').type).toBe('list_item')
    expect(applyLineInputRule('1. item').type).toBe('ordered_list_item')
    expect(applyLineInputRule('- [ ] todo').type).toBe('task_list_item')
    expect(applyLineInputRule('plain').type).toBe('paragraph')
  })
})
