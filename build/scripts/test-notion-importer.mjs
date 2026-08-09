import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import vm from 'node:vm'

const root = path.resolve(process.argv[2] || '')
if (!process.argv[2]) {
  console.error('Usage: node build/scripts/test-notion-importer.mjs <catalogue-directory>')
  process.exit(2)
}

const source = await fs.readFile(path.join(root, 'addons/notion-markdown-importer/main.js'), 'utf8')
const sandbox = { self: {} }
vm.runInNewContext(source, sandbox, { filename: 'notion-markdown-importer/main.js', timeout: 1_000 })
const definition = sandbox.self.elephantAddon
assert.equal(typeof definition?.activate, 'function')

let command
const sourceNotes = new Map([
  [
    'Imports/Notion/Projects 0123456789abcdef0123456789abcdef/Alpha aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.md',
    '# Alpha\n\nSee [Beta](Beta%20bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.md).\n'
  ],
  [
    'Imports/Notion/Projects 0123456789abcdef0123456789abcdef/Beta bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.md',
    'Beta body without a heading.\n'
  ]
])
const written = new Map()
const api = {
  notes: {
    list: async (prefix) => {
      assert.equal(prefix, 'Imports/Notion')
      return [...sourceNotes.entries()].map(([notePath, content], index) => ({
        path: notePath,
        size: content.length,
        modifiedAt: 1_700_000_000_000 + index
      }))
    },
    read: async (notePath) => ({ path: notePath, content: sourceNotes.get(notePath) }),
    write: async (notePath, content) => {
      written.set(notePath, content)
      return { ok: true, path: notePath }
    }
  },
  commands: {
    register(value) {
      command = value
      return () => { command = null }
    }
  }
}

await definition.activate(api)
assert.ok(command?.id)
const result = await command.run()

assert.equal(result.imported, 2)
assert.equal(result.rewritten, 1)
assert.equal(result.unresolved, 0)
assert.ok(written.has('Imported/Notion/Projects/Alpha.md'))
assert.ok(written.has('Imported/Notion/Projects/Beta.md'))
assert.match(written.get('Imported/Notion/Projects/Alpha.md'), /\[\[Imported\/Notion\/Projects\/Beta\|Beta\]\]/)
assert.match(written.get('Imported/Notion/Projects/Beta.md'), /^# Beta/m)
assert.match(written.get('Reports/Notion Import.md'), /Markdown notes imported: 2/)

console.log('[official-addons] notion-markdown-importer ok imported=2 rewritten=1')
