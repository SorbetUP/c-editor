import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import { analyzeSfc, skeletonFor } from './index.mjs'

const fixturePath = new URL('./fixtures/Example.vue', import.meta.url)
const source = await fs.readFile(fixturePath, 'utf8')
const analysis = analyzeSfc(source, 'Example.vue')

assert.equal(analysis.component, 'Example')
assert.equal(analysis.template.elements.find(item => item.tag === 'button').mapping, 'Button')
assert.equal(analysis.template.elements.find(item => item.tag === 'section').mapping, 'rect()')
assert.equal(analysis.template.directives.find(item => item.argument === 'click').mapping, 'on_mouse_up')
assert.equal(analysis.script.signals.refs, true)
assert.ok(analysis.unsupported.some(item => item.token === 'querySelector'))
assert.ok(analysis.unsupported.some(item => item.property === 'filter'))
assert.match(skeletonFor(analysis), /unsupported constructs are not dropped/)

console.log('vue-to-freya: structured SFC analysis passed')
