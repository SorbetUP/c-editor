import { readFile } from 'node:fs/promises'
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { loadScenario } from '../../freya-differential/lib/scenario.mjs'

test('source adapter binds the exact shared fourteen-action protocol', async () => {
  const scenario = await loadScenario('migration/freya/differential-scenarios.json')
  assert.equal(scenario.actions.length, 14)
  assert.deepEqual(scenario.actions.map((action) => action.id), [
    'launch', 'move-to-alpha-card', 'open-search', 'search-alpha', 'close-search',
    'navigate-all-notes', 'open-alpha-note', 'edit-alpha-note', 'scroll-alpha-note',
    'close-alpha-note', 'open-create-menu', 'move-through-create-menu',
    'close-create-menu', 'drag-search-rail-item'
  ])
  assert.equal(scenario.actions.reduce((count, action) => count + action.frames.length, 0), 120)
  assert.deepEqual(scenario.viewport, { width: 1280, height: 840, scaleFactor: 1, deviceScaleFactor: 1, fullPage: false, colorScheme: 'light', locale: 'en-US' })
})

test('action module contains no evaluate-based action or success injection', async () => {
  const source = await readFile(new URL('../actions.mjs', import.meta.url), 'utf8')
  assert.equal(/\bpage\.evaluate\b|\bevaluateHandle\b|\bevaluateOnNewDocument\b/.test(source), false)
  assert.equal(source.includes('window.__'), false)
  assert.equal(source.includes('.dispatchEvent('), false)
  assert.match(source, /page\.mouse\.down\(\)/)
  assert.match(source, /page\.mouse\.move\(middle\.x, middle\.y, \{ steps: 8 \}\)/)
  assert.match(source, /target\.drop\(\{ data: \{ 'text\/plain': 'search' \} \}\)/)
  assert.match(source, /page\.mouse\.up\(\)/)
})
