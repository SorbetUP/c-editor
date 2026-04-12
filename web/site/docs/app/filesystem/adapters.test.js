import test from 'node:test';
import assert from 'node:assert/strict';
import {
  LocalStorageVaultAdapter,
  WebDirectoryAdapter,
  WebDirectoryReadonlyAdapter
} from './adapters.js';

function createLocalStorageMock() {
  const backing = new Map();
  return {
    getItem(key) {
      return backing.has(key) ? backing.get(key) : null;
    },
    setItem(key, value) {
      backing.set(key, String(value));
    },
    removeItem(key) {
      backing.delete(key);
    },
    clear() {
      backing.clear();
    }
  };
}

test('LocalStorageVaultAdapter.togglePinned persists the targeted note state', async () => {
  globalThis.localStorage = createLocalStorageMock();

  const adapter = new LocalStorageVaultAdapter();
  const notesBefore = await adapter.listNotes();
  const note = notesBefore.find((entry) => !entry.pinned) || notesBefore[1];
  const result = await adapter.togglePinned(note.id, true);
  const notes = await adapter.listNotes();

  assert.equal(result, true);
  assert.equal(notes.find((entry) => entry.id === note.id)?.pinned, true);
});

test('WebDirectoryAdapter.togglePinned updates directory meta and reports success', async () => {
  globalThis.localStorage = createLocalStorageMock();

  const adapter = new WebDirectoryAdapter({ name: 'Test' });
  const result = await adapter.togglePinned('note-1', true, [{
    id: 'note-1',
    relativePath: 'folder/test.md'
  }]);

  assert.equal(result, true);
  assert.match(globalThis.localStorage.getItem('elephantnote-directory-meta-v2'), /"folder\/test\.md"/);
});

test('WebDirectoryReadonlyAdapter.togglePinned mirrors the writable meta contract', async () => {
  globalThis.localStorage = createLocalStorageMock();

  const adapter = new WebDirectoryReadonlyAdapter([]);
  const result = await adapter.togglePinned('note-1', true, [{
    id: 'note-1',
    relativePath: 'readonly/test.md'
  }]);

  assert.equal(result, true);
  assert.match(globalThis.localStorage.getItem('elephantnote-directory-meta-v2'), /"readonly\/test\.md"/);
});
