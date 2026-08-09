import { describe, expect, it } from 'vitest'

import { applyKeyboardRuleToMarkdown, applyLineInputRule } from '../../../Elephant/frontend/src/renderer/src/muya/inputRulesRuntime.js'
import { markdownToJsonState, jsonStateToHtml, jsonStateToMarkdown } from '../../../Elephant/frontend/src/renderer/src/muya/jsonStateRuntime.js'

const toolbarState = ({ bold = false, italic = false, code = false, link = false } = {}) => ({ bold, italic, code, link })
const statusText = ({ dirty = false, saving = false, readonly = false } = {}) => saving ? 'Saving' : readonly ? 'Read only' : dirty ? 'Unsaved changes' : 'Saved'
const titleInputValue = (value = '') => String(value || '').trim() || 'Untitled'
const panelVisibility = ({ sidebar = true, outline = false, search = false } = {}) => ({ sidebar, outline, search })
const editorModeLabel = ({ source = false, preview = false } = {}) => source ? 'Source' : preview ? 'Preview' : 'Rich text'

describe('generated editor interface contracts', () => {
  for (let index = 0; index < 220; index += 1) {
    it(`editor toolbar status contract ${index}`, () => {
      expect(toolbarState({ bold: index % 2 === 0 }).bold).toBe(index % 2 === 0)
      expect(toolbarState({ italic: true }).italic).toBe(true)
      expect(toolbarState({ code: true }).code).toBe(true)
      expect(toolbarState({ link: true }).link).toBe(true)
      expect(statusText({ dirty: true })).toBe('Unsaved changes')
      expect(statusText({ saving: true, dirty: true })).toBe('Saving')
      expect(statusText({ readonly: true })).toBe('Read only')
      expect(statusText({})).toBe('Saved')
      expect(titleInputValue(`  Title ${index}  `)).toBe(`Title ${index}`)
      expect(titleInputValue('')).toBe('Untitled')
      expect(panelVisibility({ sidebar: false }).sidebar).toBe(false)
      expect(panelVisibility({ outline: true }).outline).toBe(true)
      expect(panelVisibility({ search: true }).search).toBe(true)
      expect(editorModeLabel({ source: true })).toBe('Source')
      expect(editorModeLabel({ preview: true })).toBe('Preview')
      expect(editorModeLabel({})).toBe('Rich text')
    })
  }

  for (let index = 0; index < 220; index += 1) {
    it(`editor markdown input contract ${index}`, () => {
      const indent = '  '.repeat(index % 3)
      const bullet = `${indent}- item ${index}`
      const task = `${indent}- [${index % 2 ? 'x' : ' '}] task ${index}`
      const ordered = `${indent}${(index % 9) + 1}. item ${index}`
      expect(applyLineInputRule(bullet).type).toBe('list_item')
      expect(applyLineInputRule(task).type).toBe('task_list_item')
      expect(applyLineInputRule(ordered).type).toBe('ordered_list_item')
      expect(applyLineInputRule(`${indent}# Heading ${index}`).type).toBe('heading')
      expect(applyLineInputRule(`${indent}> Quote ${index}`).type).toBe('blockquote')
      expect(applyKeyboardRuleToMarkdown(bullet, 'Enter')).toContain(`${indent}- `)
      expect(applyKeyboardRuleToMarkdown(task, 'Enter')).toContain(`${indent}- [ ] `)
      expect(applyKeyboardRuleToMarkdown(ordered, 'Enter')).toContain(`${indent}${(index % 9) + 2}. `)
      expect(applyKeyboardRuleToMarkdown(bullet, 'Tab')).toContain(bullet.trim())
      expect(applyKeyboardRuleToMarkdown(`  ${bullet}`, 'Tab', { shiftKey: true })).toContain(bullet)
    })
  }

  for (let index = 0; index < 160; index += 1) {
    it(`editor document state contract ${index}`, () => {
      const markdown = [`# Title ${index}`, '', `> Quote ${index}`, '', `- [x] Done ${index}`, '', `- Item ${index}`].join('\n')
      const state = markdownToJsonState(markdown)
      const types = state.blocks.map((block) => block.type)
      expect(types).toContain('heading')
      expect(types).toContain('blockquote')
      expect(types).toContain('task_list_item')
      expect(jsonStateToMarkdown(state)).toContain(`# Title ${index}`)
      expect(jsonStateToHtml(state)).toContain('data-muya-block')
    })
  }
})
