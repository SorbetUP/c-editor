import test from 'node:test';
import assert from 'node:assert/strict';
import { NotesStore } from './notes-store.js';

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

test('NotesStore.togglePinned persists and reloads the resulting state', async () => {
  globalThis.localStorage = createLocalStorageMock();

  const store = new NotesStore();
  let persistedPinnedValue = false;
  const adapter = {
    capabilities() {
      return {
        type: 'test',
        label: 'Test',
        canRead: true,
        canWrite: true,
        canDelete: true,
        canCreate: true
      };
    },
    async togglePinned(noteId, value) {
      assert.equal(noteId, 'note-1');
      persistedPinnedValue = value;
      return true;
    },
    async listNotes() {
      return [{
        id: 'note-1',
        content: '# Test\n',
        pinned: persistedPinnedValue,
        writable: true,
        sourceKind: 'test',
        createdAt: '2026-04-12T00:00:00.000Z',
        updatedAt: '2026-04-12T00:00:00.000Z'
      }];
    }
  };

  store.store.setState((state) => ({
    ...state,
    source: {
      ...state.source,
      adapter,
      capabilities: adapter.capabilities()
    },
    notes: {
      items: [{
        id: 'note-1',
        content: '# Test\n',
        pinned: false,
        writable: true,
        sourceKind: 'test',
        createdAt: '2026-04-12T00:00:00.000Z',
        updatedAt: '2026-04-12T00:00:00.000Z'
      }],
      filteredItems: [{
        id: 'note-1',
        content: '# Test\n',
        pinned: false,
        writable: true,
        sourceKind: 'test',
        createdAt: '2026-04-12T00:00:00.000Z',
        updatedAt: '2026-04-12T00:00:00.000Z'
      }]
    }
  }));

  const result = await store.togglePinned('note-1');

  assert.equal(result, true);
  assert.equal(store.getState().notes.items[0].pinned, true);
});
