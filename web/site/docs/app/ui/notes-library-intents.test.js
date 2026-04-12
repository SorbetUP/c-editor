import test from 'node:test';
import assert from 'node:assert/strict';
import { resolveNotesLibraryIntent, resolveNotesLibraryKeyIntent } from './notes-library-intents.js';
import { UI_INTENT } from './ui-intents.js';

function createTarget(lookup = {}) {
  return {
    closest(selector) {
      return lookup[selector] || null;
    }
  };
}

test('explicit action controls win over note opening', () => {
  const target = createTarget({
    '[data-intent-action]': {
      dataset: {
        intentAction: UI_INTENT.TOGGLE_PIN,
        noteId: 'note-1'
      }
    },
    '[data-intent-open-note]': {
      dataset: {
        noteId: 'note-1'
      }
    }
  });

  assert.deepEqual(resolveNotesLibraryIntent(target), {
    type: UI_INTENT.TOGGLE_PIN,
    detail: { noteId: 'note-1' }
  });
});

test('open-note intent only comes from the explicit open surface', () => {
  const target = createTarget({
    '[data-intent-open-note]': {
      dataset: {
        noteId: 'note-2'
      }
    }
  });

  assert.deepEqual(resolveNotesLibraryIntent(target), {
    type: UI_INTENT.OPEN_NOTE,
    detail: { noteId: 'note-2' }
  });
});

test('keyboard activation is limited to enter and space on the open surface', () => {
  const target = createTarget({
    '[data-intent-open-note]': {
      dataset: {
        noteId: 'note-3'
      }
    }
  });

  assert.equal(resolveNotesLibraryKeyIntent(target, 'Escape'), null);
  assert.deepEqual(resolveNotesLibraryKeyIntent(target, 'Enter'), {
    type: UI_INTENT.OPEN_NOTE,
    detail: { noteId: 'note-3' }
  });
  assert.deepEqual(resolveNotesLibraryKeyIntent(target, ' '), {
    type: UI_INTENT.OPEN_NOTE,
    detail: { noteId: 'note-3' }
  });
});
