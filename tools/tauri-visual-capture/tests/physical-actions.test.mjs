import assert from 'node:assert/strict'
import test from 'node:test'

import { loadSharedScenario } from '../scenario.mjs'
import { resolvePhysicalEvent } from '../physical-actions.mjs'

test('maps the target-only shared hover action to a native pointer movement', async () => {
  const shared = await loadSharedScenario('migration/freya/differential-scenarios.json')
  assert.equal(resolvePhysicalEvent(shared.scenario.actions[1]), 'move-pointer')
  assert.equal(resolvePhysicalEvent(shared.scenario.actions[2]), 'click')
  assert.equal(resolvePhysicalEvent(shared.scenario.actions[0]), null)
})
