import { describe, expect, it } from 'vitest'

import { formatShortDate } from '../../../Elephant/frontend/app/services/markdownMetaService.js'
import { getNoteCardExcerpt, getNoteCardTitle, getNoteCardTypeLabel, getNoteCardUpdatedLabel } from '../../../Elephant/frontend/app/utils/noteCardView.js'
import {
  applyCatalogFilters,
  dedupeModelsById,
  formatCompactCount,
  formatRelativeDate,
  getModelCapabilities,
  getModelFormat,
  getModelQuantization,
  getModelSource,
  getModelUpdatedDate,
  isLocalModel,
  isRemoteModel
} from '../../../Elephant/frontend/app/components/views/modelsViewHelpers.js'
import { applyKeyboardRuleToMarkdown, applyLineInputRule } from '../../../Elephant/frontend/src/renderer/src/muya/inputRulesRuntime.js'
import { clipboardPayloadToMarkdown, pastedHtmlToMarkdown, tableToMarkdown } from '../../../Elephant/frontend/src/renderer/src/muya/clipboardRuntime.js'
import { jsonStateToHtml, jsonStateToMarkdown, markdownToJsonState } from '../../../Elephant/frontend/src/renderer/src/muya/jsonStateRuntime.js'
import { readMuyaRuntimeMode } from '../../../Elephant/frontend/src/renderer/src/muya/runtimeFlags.js'

const fence = String.fromCharCode(45, 45, 45)

const noteFixtures = Array.from({ length: 130 }, (_, index) => ({
  title: `Note ${index}`,
  filename: `note-${index}.md`,
  body: `Visible body ${index}`,
  tag: `meta-${index}`
}))

const dateFixtures = Array.from({ length: 70 }, (_, index) => ({
  valid: `2026-06-${String((index % 28) + 1).padStart(2, '0')}T10:00:00.000Z`,
  invalid: `invalid-date-${index}`
}))

const modelFixtures = Array.from({ length: 120 }, (_, index) => ({
  remote: {
    id: `remote-${index}`,
    repoId: index % 2 === 0 ? `sentence-transformers/model-${index}` : `org/chat-model-${index}`,
    fileName: index % 3 === 0 ? `model-${index}.Q4_K_M.gguf` : `model-${index}.gguf`,
    pipelineTag: index % 2 === 0 ? 'feature-extraction' : 'text-generation',
    task: index % 5 === 0 ? 'ocr' : '',
    downloads: index * 10,
    likes: index
  },
  local: {
    id: `local-${index}`,
    repoId: `org/local-${index}`,
    path: `/models/local-${index}.gguf`,
    fileName: `local-${index}.Q5_K_M.gguf`
  }
}))

const keyboardFixtures = Array.from({ length: 90 }, (_, index) => ({
  bullet: `${'  '.repeat(index % 3)}- item ${index}`,
  ordered: `${(index % 9) + 1}. item ${index}`,
  task: `${'  '.repeat(index % 2)}- [${index % 2 ? 'x' : ' '}] task ${index}`
}))

const clipboardFixtures = Array.from({ length: 80 }, (_, index) => ({
  html: `<p><strong>Bold ${index}</strong> <em>Em ${index}</em> <code>x${index}</code></p><a href="https://example.com/${index}">site ${index}</a><img src="pic-${index}.png" alt="Alt ${index}">`,
  table: `<tr><th>A${index}</th><th>B${index}</th></tr><tr><td>1</td><td>2</td></tr>`
}))

const jsonStateFixtures = Array.from({ length: 90 }, (_, index) => [
  `# Title ${index}`,
  '',
  `> Quote ${index}`,
  '',
  `- [${index % 2 ? 'x' : ' '}] Task ${index}`,
  '',
  `- Item ${index}`,
  '',
  `${(index % 9) + 1}. Ordered ${index}`
].join('\n'))

describe('generated interface parity suite', () => {
  for (const fixture of noteFixtures) {
    it(`note card metadata preview parity ${fixture.filename}`, () => {
      const entry = {
        title: fixture.title,
        filename: fixture.filename,
        type: 'note',
        updatedAt: '2026-06-22T10:00:00.000Z',
        excerpt: [fence, fixture.tag, fence, '', `# ${fixture.title}`, '', fixture.body].join('\n')
      }
      const excerpt = getNoteCardExcerpt(entry)
      expect(getNoteCardTitle(entry)).toBe(fixture.title)
      expect(getNoteCardTypeLabel(entry)).toBe('note')
      expect(getNoteCardUpdatedLabel(entry)).toMatch(/^2026-06-22$/)
      expect(excerpt).toContain(fixture.body)
      expect(excerpt).not.toContain(fence)
      expect(excerpt).not.toContain(fixture.tag)
    })
  }

  for (const fixture of dateFixtures) {
    it(`date formatting parity ${fixture.invalid}`, () => {
      expect(formatShortDate(fixture.invalid)).toBe('')
      expect(getModelUpdatedDate({ updatedAt: fixture.invalid })).toBe('')
      expect(formatRelativeDate(fixture.invalid)).toBe('')
      expect(formatShortDate(fixture.valid)).toMatch(/^2026-06-/)
    })
  }

  for (const fixture of modelFixtures) {
    it(`models catalog parity ${fixture.remote.id}`, () => {
      expect(isRemoteModel(fixture.remote)).toBe(true)
      expect(isLocalModel(fixture.local)).toBe(true)
      expect(getModelSource(fixture.remote)).toBe('Hugging Face')
      expect(getModelSource(fixture.local)).toBe('Local')
      expect(getModelFormat(fixture.remote)).toBe('GGUF')
      expect(getModelQuantization(fixture.local)).toBe('Q5_K_M')
      expect(formatCompactCount(fixture.remote.downloads)).not.toBe('')
      expect(getModelCapabilities(fixture.remote).length).toBeGreaterThan(0)
      const deduped = dedupeModelsById([fixture.remote, { ...fixture.remote, path: `/models/${fixture.remote.id}.gguf` }])
      expect(deduped).toHaveLength(1)
      const filtered = applyCatalogFilters({ models: [fixture.remote, fixture.local], source: 'remote', format: 'gguf' })
      expect(filtered.every((model) => isRemoteModel(model))).toBe(true)
    })
  }

  for (const fixture of keyboardFixtures) {
    it(`keyboard input parity ${fixture.ordered}`, () => {
      expect(applyLineInputRule(fixture.bullet).type).toBe('list_item')
      expect(applyLineInputRule(fixture.ordered).type).toBe('ordered_list_item')
      expect(applyLineInputRule(fixture.task).type).toBe('task_list_item')
      expect(applyKeyboardRuleToMarkdown(fixture.bullet, 'Enter')).toContain('- ')
      expect(applyKeyboardRuleToMarkdown(fixture.ordered, 'Enter')).toContain('.')
      expect(applyKeyboardRuleToMarkdown(fixture.task, 'Enter')).toContain('- [ ] ')
      expect(applyKeyboardRuleToMarkdown(fixture.bullet, 'Tab')).toContain(fixture.bullet.trim())
    })
  }

  for (const fixture of clipboardFixtures) {
    it(`clipboard paste parity ${fixture.html.length}`, () => {
      const markdown = pastedHtmlToMarkdown(fixture.html)
      expect(markdown).toContain('**Bold')
      expect(markdown).toContain('*Em')
      expect(markdown).toContain('`x')
      expect(markdown).toContain('[site')
      expect(markdown).toContain('![')
      expect(tableToMarkdown(fixture.table)).toContain('| A')
      expect(clipboardPayloadToMarkdown({ text: 'plain' })).toBe('plain')
    })
  }

  for (const markdown of jsonStateFixtures) {
    it(`json state editor parity ${markdown.slice(0, 16)}`, () => {
      const state = markdownToJsonState(markdown)
      const types = state.blocks.map((block) => block.type)
      expect(types).toContain('heading')
      expect(types).toContain('blockquote')
      expect(types).toContain('task_list_item')
      expect(types).toContain('list_item')
      expect(jsonStateToMarkdown(state)).toContain('# Title')
      expect(jsonStateToHtml(state)).toContain('data-muya-block')
    })
  }

  it('keeps the Rust editor active in production runtimes', () => {
    expect(readMuyaRuntimeMode({ __MARKTEXT_RUNTIME__: 'tauri' })).toBe('rust')
    expect(readMuyaRuntimeMode({ __MARKTEXT_RUNTIME__: 'electron' })).toBe('rust')
  })
})
