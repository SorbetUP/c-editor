export const UI_INTENT = Object.freeze({
  OPEN_NOTE: 'open-note',
  TOGGLE_PIN: 'toggle-pin',
  DELETE_NOTE: 'delete-note',
  CREATE_NOTE: 'create-note',
  CLOSE_EDITOR: 'close-editor',
  SAVE_NOTE: 'save-note',
  OPEN_SOURCE: 'open-source',
  IMPORT_FILE: 'import-file',
  SEARCH_CHANGE: 'search-change'
});

export function createIntentEvent(type, detail = {}) {
  return new CustomEvent(type, {
    bubbles: true,
    composed: true,
    detail
  });
}

export function dispatchIntent(target, type, detail = {}) {
  target.dispatchEvent(createIntentEvent(type, detail));
}
