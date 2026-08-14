import assert from 'node:assert/strict'
import test from 'node:test'

import { loadSharedScenario } from '../scenario.mjs'
import { resolvePhysicalEvent, translateObservedRect } from '../physical-actions.mjs'

test('maps the target-only shared hover action to a native pointer movement', async () => {
  const shared = await loadSharedScenario('migration/freya/differential-scenarios.json')
  assert.equal(resolvePhysicalEvent(shared.scenario.actions[1]), 'move-pointer')
  assert.equal(resolvePhysicalEvent(shared.scenario.actions[2]), 'click')
  assert.equal(resolvePhysicalEvent(shared.scenario.actions[0]), null)
})

test('translates DOM viewport bounds from the native window origin', () => {
  const result = translateObservedRect({
    rect: { x: 10.5, y: 100, width: 34, height: 34 },
    windowBounds: { x: 80, y: 60, width: 1280, height: 840 },
    viewportRect: { x: 0, y: 0, width: 1280, height: 840 }
  })
  assert.deepEqual(result.rect, { x: 90.5, y: 160, width: 34, height: 34 })
  assert.deepEqual(result.scale, { x: 1, y: 1 })
})

test('scales DOM viewport bounds to logical native window coordinates', () => {
  const result = translateObservedRect({
    rect: { x: 10, y: 20, width: 30, height: 40 },
    windowBounds: { x: 100, y: 200, width: 1280, height: 840 },
    viewportRect: { x: 0, y: 0, width: 640, height: 420 }
  })
  assert.deepEqual(result.rect, { x: 120, y: 240, width: 60, height: 80 })
  assert.deepEqual(result.scale, { x: 2, y: 2 })
})
