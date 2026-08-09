import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import vm from 'node:vm'

const root = path.resolve(process.argv[2] || '')
if (!process.argv[2]) {
  console.error('Usage: node build/scripts/test-popular-note-addons.mjs <catalogue-directory>')
  process.exit(2)
}

const loadAddon = async (slug) => {
  const source = await fs.readFile(path.join(root, 'addons', slug, 'main.js'), 'utf8')
  const sandbox = { self: {}, Intl }
  vm.runInNewContext(source, sandbox, { filename: `${slug}/main.js`, timeout: 1_000 })
  assert.equal(typeof sandbox.self.elephantAddon?.activate, 'function')
  return sandbox.self.elephantAddon
}

const activateCommands = async (definition, api) => {
  const commands = new Map()
  const wrappedApi = {
    ...api,
    commands: {
      register(command) {
        assert.ok(command?.id)
        assert.equal(typeof command.run, 'function')
        commands.set(command.id, command)
        return () => commands.delete(command.id)
      }
    }
  }
  const dispose = await definition.activate(wrappedApi)
  assert.ok(commands.size > 0)
  return { commands, dispose }
}

const createStorage = () => {
  const values = new Map()
  return {
    values,
    api: {
      get: async (key) => values.get(key) ?? null,
      set: async (key, value) => { values.set(key, value); return { ok: true } },
      remove: async (key) => values.delete(key),
      entries: async () => Object.fromEntries(values)
    }
  }
}

const createNotes = (source = new Map()) => {
  const written = new Map()
  let clock = 1_700_000_000_000
  const all = () => new Map([...source, ...written])
  return {
    source,
    written,
    api: {
      list: async (prefix = '.') => [...all().entries()]
        .filter(([notePath]) => prefix === '.' || notePath === prefix || notePath.startsWith(`${prefix}/`))
        .map(([notePath, content], index) => ({
          path: notePath,
          size: String(content).length,
          modifiedAt: clock + index
        })),
      read: async (notePath) => {
        const notes = all()
        if (!notes.has(notePath)) throw new Error(`missing note: ${notePath}`)
        return { path: notePath, content: notes.get(notePath) }
      },
      write: async (notePath, content) => {
        clock += 100
        written.set(notePath, content)
        return { ok: true, path: notePath }
      }
    }
  }
}

const baseApi = (notes, storage = createStorage()) => ({
  app: { info: async () => ({ name: 'ElephantNote', version: '0.1.0', addonApiVersion: 1 }) },
  notes: notes.api,
  storage: storage.api,
  http: { request: async () => { throw new Error('network not expected') } }
})

const testTemplateStudio = async () => {
  const notes = createNotes()
  const { commands } = await activateCommands(await loadAddon('template-studio'), baseApi(notes))
  const result = await commands.get('com.elephantnote.template-studio.create').run({ title: 'Research Plan' })
  assert.equal(result.templateCreated, true)
  assert.equal(result.template, 'Templates/Default.md')
  assert.match(result.path, /^Generated\/\d{4}-\d{2}-\d{2}\//)
  assert.match(notes.written.get(result.path), /# Research Plan/)
  assert.equal(result.unresolvedVariables.length, 0)
  console.log('[popular-addons] template-studio ok generated=1')
}

const testWeeklyReview = async () => {
  const notes = createNotes(new Map([
    ['Daily/2026-07-08.md', '# Wednesday\n\n- [x] Ship patch\n- Useful discovery\n'],
    ['Daily/2026-07-09.md', '# Thursday\n\n- [ ] Review logs\n- Keep tests deterministic\n'],
    ['Daily/ignore.md', '# Not dated\n']
  ]))
  const { commands } = await activateCommands(await loadAddon('weekly-review'), baseApi(notes))
  const result = await commands.get('com.elephantnote.weekly-review.generate').run()
  assert.equal(result.sourceNotes, 2)
  assert.equal(result.openTasks, 1)
  assert.equal(result.completedTasks, 1)
  assert.equal(result.highlights, 2)
  const report = notes.written.get(result.path)
  assert.match(report, /Review logs/)
  assert.match(report, /Ship patch/)
  assert.match(report, /\[\[Daily\/2026-07-09\|2026-07-09\]\]/)
  console.log('[popular-addons] weekly-review ok sources=2')
}

const testWritingStatistics = async () => {
  const storage = createStorage()
  const notes = createNotes(new Map([
    ['Notes/Long.md', '# Long\n\nOne two three four five six.\n'],
    ['Notes/Short.md', '# Short\n\nHello world.\n']
  ]))
  const { commands } = await activateCommands(await loadAddon('writing-statistics'), baseApi(notes, storage))
  const command = commands.get('com.elephantnote.writing-statistics.generate')
  const first = await command.run()
  assert.equal(first.notes, 2)
  assert.ok(first.words >= 8)
  assert.match(notes.written.get(first.path), /Largest notes/)
  const second = await command.run()
  assert.match(notes.written.get(second.path), /Change since previous run: 0 words, 0 notes/)
  assert.equal(storage.values.get('lastSummary').notes, 2)
  console.log(`[popular-addons] writing-statistics ok words=${first.words}`)
}

const testVaultChangelog = async () => {
  const storage = createStorage()
  const notes = createNotes(new Map([
    ['A.md', '# A\n'],
    ['B.md', '# B\n']
  ]))
  const { commands } = await activateCommands(await loadAddon('vault-changelog'), baseApi(notes, storage))
  const command = commands.get('com.elephantnote.vault-changelog.snapshot')
  const first = await command.run()
  assert.equal(first.baseline, true)

  notes.source.set('A.md', '# A changed\n')
  notes.source.delete('B.md')
  notes.source.set('C.md', '# C\n')
  const second = await command.run()
  assert.equal(second.baseline, false)
  assert.equal(second.created, 1)
  assert.equal(second.modified, 1)
  assert.equal(second.deleted, 1)
  const report = notes.written.get(second.path)
  assert.match(report, /\[\[C\|C\]\]/)
  assert.match(report, /`B\.md`/)
  console.log('[popular-addons] vault-changelog ok created=1 modified=1 deleted=1')
}

const testTagIndex = async () => {
  const notes = createNotes(new Map([
    ['One.md', '---\ntags: [Project, research]\n---\n\n# One\n\nWork on #Ideas.\n'],
    ['Two.md', '---\ntags:\n  - project\n---\n\n# Two\n\nMore #ideas and `#ignored`.\n']
  ]))
  const { commands } = await activateCommands(await loadAddon('tag-index'), baseApi(notes))
  const result = await commands.get('com.elephantnote.tag-index.generate').run()
  assert.equal(result.notes, 2)
  assert.equal(result.tags, 3)
  assert.equal(result.collisions, 2)
  const report = notes.written.get(result.path)
  assert.match(report, /#project/)
  assert.match(report, /`Project`, `project`/)
  assert.doesNotMatch(report, /#ignored/)
  console.log('[popular-addons] tag-index ok tags=3 collisions=2')
}

const testMarkdownLinter = async () => {
  const original = '# First  \r\n# Second\r\n##Malformed\r\n\r\n\r\n\r\nText   '
  const notes = createNotes(new Map([['Messy.md', original], ['Clean.md', '# Clean\n']]))
  const { commands } = await activateCommands(await loadAddon('markdown-linter'), baseApi(notes))
  assert.equal(commands.size, 2)

  const audit = await commands.get('com.elephantnote.markdown-linter.audit').run()
  assert.equal(audit.changed, 0)
  assert.equal(notes.source.get('Messy.md'), original)
  assert.match(notes.written.get(audit.path), /Malformed headings without a space: 1/)

  const applied = await commands.get('com.elephantnote.markdown-linter.apply-safe-fixes').run()
  assert.equal(applied.changed, 1)
  const fixed = notes.written.get('Messy.md')
  assert.doesNotMatch(fixed, /\r/)
  assert.doesNotMatch(fixed, / +$/m)
  assert.ok(fixed.endsWith('\n'))
  assert.match(fixed, /##Malformed/)
  assert.match(notes.written.get(applied.path), /Notes changed by safe fixes: 1/)
  console.log('[popular-addons] markdown-linter ok audit+apply')
}

await testTemplateStudio()
await testWeeklyReview()
await testWritingStatistics()
await testVaultChangelog()
await testTagIndex()
await testMarkdownLinter()
console.log('[popular-addons] all six functional pipelines passed')
