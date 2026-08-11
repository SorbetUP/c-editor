import test from 'node:test'
import assert from 'node:assert/strict'

import { loadSharedScenario } from '../scenario.mjs'

test('capture adapter consumes the shared 14-action scenario without substitution', async () => {
  const loaded = await loadSharedScenario()
  assert.equal(loaded.actions.length, 14)
  assert.deepEqual(loaded.actions.map((action) => action.id), [
    'launch', 'move-to-alpha-card', 'open-search', 'search-alpha', 'close-search',
    'navigate-all-notes', 'open-alpha-note', 'edit-alpha-note', 'scroll-alpha-note',
    'close-alpha-note', 'open-create-menu', 'move-through-create-menu',
    'close-create-menu', 'drag-search-rail-item'
  ])
  assert.ok(loaded.actions.every((action) => action.frameTimes.length >= 3))
  assert.ok(loaded.scenario.capture.beforeAndAfterEveryAction)
})
