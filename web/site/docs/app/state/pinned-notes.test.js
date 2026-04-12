import test from 'node:test';
import assert from 'node:assert/strict';
import {
  applyPinnedMeta,
  applyPinnedValueToCollection,
  getNextPinnedValue
} from './pinned-notes.js';

test('getNextPinnedValue toggles the current pin state', () => {
  assert.equal(getNextPinnedValue({ pinned: true }), false);
  assert.equal(getNextPinnedValue({ pinned: false }), true);
});

test('applyPinnedValueToCollection mutates only the targeted note', () => {
  const { changed, notes } = applyPinnedValueToCollection([
    { id: 'a', pinned: false },
    { id: 'b', pinned: true }
  ], 'a', true, '2026-04-12T00:00:00.000Z');

  assert.equal(changed, true);
  assert.deepEqual(notes, [
    { id: 'a', pinned: true, updatedAt: '2026-04-12T00:00:00.000Z' },
    { id: 'b', pinned: true }
  ]);
});

test('applyPinnedMeta updates directory metadata without dropping sibling keys', () => {
  const result = applyPinnedMeta({ 'foo.md': { pinned: false, extra: 1 } }, { relativePath: 'foo.md' }, true);

  assert.equal(result.changed, true);
  assert.deepEqual(result.meta, {
    'foo.md': { pinned: true, extra: 1 }
  });
});
