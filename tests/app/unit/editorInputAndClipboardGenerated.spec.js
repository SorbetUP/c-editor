import { describe, expect, it } from 'vitest'

import { applyKeyboardRuleToMarkdown, applyLineInputRule } from '../../../Elephant/frontend/src/renderer/src/muya/inputRulesRuntime.js'
import { clipboardPayloadToMarkdown, copyMarkdownAndHtml, pastedHtmlToMarkdown, sanitizePastedHtml, tableToMarkdown } from '../../../Elephant/frontend/src/renderer/src/muya/clipboardRuntime.js'

describe('generated editor input and clipboard contracts', () => {
  for (let index = 0; index < 160; index += 1) {
    it(`editor input contract ${index}`, () => {
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

  for (let index = 0; index < 120; index += 1) {
    it(`clipboard rich conversion contract ${index}`, () => {
      const html = `<p><strong>Bold ${index}</strong> <em>Em ${index}</em> <code>x${index}</code></p><a href="https://example.com/${index}">site ${index}</a><img src="pic-${index}.png" alt="Alt ${index}">`
      const markdown = pastedHtmlToMarkdown(html)
      expect(markdown).toContain('**Bold')
      expect(markdown).toContain('*Em')
      expect(markdown).toContain('`x')
      expect(markdown).toContain('[site')
      expect(markdown).toContain(`![Alt ${index}]`)
      expect(tableToMarkdown(`<tr><th>A${index}</th><th>B${index}</th></tr><tr><td>1</td><td>2</td></tr>`)).toContain(`| A${index} | B${index} |`)
      expect(clipboardPayloadToMarkdown({ text: `plain ${index}` })).toBe(`plain ${index}`)
      expect(copyMarkdownAndHtml(`# T${index}`, (value) => `<h1>${value}</h1>`).html).toContain(`# T${index}`)
    })
  }

  for (let index = 0; index < 80; index += 1) {
    it(`clipboard cleanup contract ${index}`, () => {
      const html = `<p class="MsoNormal" data-block-id="${index}"><span>Text ${index}</span></p><meta charset="utf-8"><div contenteditable="true">Edit ${index}</div>`
      const clean = sanitizePastedHtml(html)
      expect(clean).not.toContain('MsoNormal')
      expect(clean).not.toContain('data-block-id')
      expect(clean).not.toContain('<meta')
      expect(clean).not.toContain('contenteditable')
      expect(pastedHtmlToMarkdown(clean)).toContain(`Text ${index}`)
    })
  }
})
