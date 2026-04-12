import { UI_INTENT } from './ui-intents.js';

export const NOTES_LIBRARY_SELECTOR = Object.freeze({
  ACTION: '[data-intent-action]',
  OPEN: '[data-intent-open-note]'
});

function extractNoteId(element) {
  return element?.dataset?.noteId || null;
}

export function resolveNotesLibraryIntent(target) {
  const actionElement = target?.closest?.(NOTES_LIBRARY_SELECTOR.ACTION) || null;
  if (actionElement) {
    return {
      type: actionElement.dataset.intentAction,
      detail: { noteId: extractNoteId(actionElement) }
    };
  }

  const openElement = target?.closest?.(NOTES_LIBRARY_SELECTOR.OPEN) || null;
  if (openElement) {
    return {
      type: UI_INTENT.OPEN_NOTE,
      detail: { noteId: extractNoteId(openElement) }
    };
  }

  return null;
}

export function resolveNotesLibraryKeyIntent(target, key) {
  const normalizedKey = String(key || '');
  if (normalizedKey !== 'Enter' && normalizedKey !== ' ') {
    return null;
  }

  const openElement = target?.closest?.(NOTES_LIBRARY_SELECTOR.OPEN) || null;
  if (!openElement) {
    return null;
  }

  return {
    type: UI_INTENT.OPEN_NOTE,
    detail: { noteId: extractNoteId(openElement) }
  };
}
