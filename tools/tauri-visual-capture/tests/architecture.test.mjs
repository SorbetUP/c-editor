import { readFileSync } from 'node:fs'
import path from 'node:path'
import test from 'node:test'
import assert from 'node:assert/strict'

const toolsRoot = path.resolve(import.meta.dirname, '..')

test('capture entrypoint stays focal and delegates runtime responsibilities', () => {
  const index = readFileSync(path.join(toolsRoot, 'index.mjs'), 'utf8')
  const lines = index.split('\n').length - 1
  assert.ok(lines <= 200, `index.mjs grew to ${lines} lines; split capture responsibilities into focused modules`)
  assert.match(index, /runSharedActions/)
  assert.match(index, /createFrameCapture/)
  assert.match(index, /proveProfileIsolation/)
})
