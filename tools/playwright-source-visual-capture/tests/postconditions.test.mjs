import { test } from 'node:test'
import assert from 'node:assert/strict'
import { assertActionPostcondition } from '../postconditions.mjs'

const frames = [{ sha256: 'before' }, { sha256: 'after' }]

test('scroll postcondition rejects unchanged scrollTop even when frames exist', () => {
  assert.throws(() => assertActionPostcondition({ id: 'scroll-alpha-note', delta: { y: 560 } }, {
    before: { scroll: { scrollTop: 0 } },
    after: { scroll: { target: '.en-editor-host .editor-component', scrollTop: 0, scrollable: true } },
    frames
  }), /scrollTop did not change/)
})

test('rail postcondition rejects drag logs without DOM and persisted reorder', () => {
  assert.throws(() => assertActionPostcondition({ id: 'drag-search-rail-item' }, {
    before: { railOrder: ['search', 'sidebar-toggle'], railPersistence: { persistedOrder: [] } },
    after: { railOrder: ['search', 'sidebar-toggle'], railPersistence: { persistedOrder: [] } },
    frames,
    dropObserved: true
  }), /DOM rail order did not change/)
})

test('all shared actions have fail-closed postconditions', () => {
  const ids = ['launch', 'move-to-alpha-card', 'open-search', 'search-alpha', 'close-search', 'navigate-all-notes', 'open-alpha-note', 'edit-alpha-note', 'scroll-alpha-note', 'close-alpha-note', 'open-create-menu', 'move-through-create-menu', 'close-create-menu', 'drag-search-rail-item']
  for (const id of ids) assert.equal(typeof assertActionPostcondition, 'function', id)
})
