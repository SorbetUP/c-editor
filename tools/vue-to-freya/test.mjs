import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { analyzeVueFile } from './ast.mjs'

const source = await readFile('Elephant/frontend/app/components/library/CreateEntryMenu.vue', 'utf8')
const report = analyzeVueFile('CreateEntryMenu.vue', source)

assert.equal(report.sourceBlocks.template, true)
assert.equal(report.sourceBlocks.script, true)
assert.ok(report.template.some((node) => node.kind === 'element' && node.sourceTag === 'div'))
assert.ok(report.script.state.some(({ code }) => code.includes('ref(')))
assert.ok(report.template.flatMap((node) => JSON.stringify(node)).some((value) => value.includes('@on:click')))
assert.ok(report.styles.mappings.some(({ declarations }) => declarations.some(({ freya }) => freya === 'with_corner_radius')))

const shellSource = await readFile('Elephant/frontend/app/components/shell/AppShell.vue', 'utf8')
const shell = analyzeVueFile('AppShell.vue', shellSource)
assert.ok(shell.unsupported.some(({ kind }) => kind === 'web-api:document'))
assert.ok(shell.unsupported.some(({ kind }) => kind === 'web-api:window'))
assert.ok(shell.template.some((node) => node.kind === 'element' && node.sourceTag === 'empty-vault-picker'))

console.log(JSON.stringify({
  passed: true,
  source: report.source,
  shellUnsupported: shell.unsupported.length,
  createEntryUnsupported: report.unsupported.length
}))
