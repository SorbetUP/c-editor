import { createStore } from '../core/store.js';
import {
  buildMarkdownDocument,
  createDocumentFromNote,
  documentToNoteContent
} from '../models/markdown-document.js';
import {
  LocalStorageVaultAdapter,
  WebDirectoryAdapter,
  WebDirectoryReadonlyAdapter
} from '../filesystem/adapters.js';
import { getNextPinnedValue } from './pinned-notes.js';

function createInitialState() {
  return {
    ui: {
      searchQuery: '',
      editorOpen: false,
      currentNoteId: null,
      currentDraft: null,
      dirty: false,
      activeTheme: 'dark'
    },
    notes: {
      items: [],
      filteredItems: []
    },
    source: {
      adapter: null,
      type: 'local-storage',
      label: 'Local',
      capabilities: {
        canRead: true,
        canWrite: true,
        canDelete: true,
        canCreate: true
      }
    }
  };
}

export class NotesStore {
  constructor() {
    this.store = createStore(createInitialState());
  }

  subscribe(listener) {
    return this.store.subscribe(listener);
  }

  getState() {
    return this.store.getState();
  }

  async initialize() {
    const adapter = new LocalStorageVaultAdapter();
    const notes = await adapter.listNotes();
    this.#setSource(adapter);
    this.#setNotes(notes);
  }

  setSearchQuery(query) {
    const nextQuery = String(query || '');
    this.store.setState((state) => ({
      ...state,
      ui: {
        ...state.ui,
        searchQuery: nextQuery
      },
      notes: {
        ...state.notes,
        filteredItems: this.#filterNotes(state.notes.items, nextQuery)
      }
    }));
  }

  async openBestDirectorySource() {
    let adapter = null;

    try {
      adapter = await WebDirectoryAdapter.open();
    } catch (error) {
      if (error && error.name === 'AbortError') {
        return false;
      }
      if (!error || error.name !== 'AbortError') {
        console.warn('Writable directory access unavailable, falling back to readonly:', error);
      }
    }

    if (!adapter) {
      try {
        adapter = await WebDirectoryReadonlyAdapter.open();
      } catch (error) {
        if (error && error.name === 'AbortError') {
          return false;
        }
        throw error;
      }
    }

    const notes = await adapter.listNotes();
    this.#setSource(adapter);
    this.#setNotes(notes);
    this.closeEditor();
    return true;
  }

  async importFile(file) {
    const adapter = this.getState().source.adapter;
    if (!adapter || typeof adapter.importFile !== 'function') {
      return null;
    }

    try {
      const note = await adapter.importFile(file);
      await this.reloadNotes();
      this.openNote(note.id);
      return note;
    } catch (error) {
      console.warn('Unable to import file into current source:', error);
      const fallbackAdapter = new LocalStorageVaultAdapter();
      const importedNote = await fallbackAdapter.importFile(file);
      this.#setSource(fallbackAdapter);
      this.#setNotes(await fallbackAdapter.listNotes());
      this.openNote(importedNote.id);
      return importedNote;
    }
  }

  openNote(noteId) {
    const note = this.#findNote(noteId);
    if (!note) {
      return false;
    }

    this.store.setState((state) => ({
      ...state,
      ui: {
        ...state.ui,
        editorOpen: true,
        currentNoteId: noteId,
        currentDraft: createDocumentFromNote(note),
        dirty: false
      }
    }));
    return true;
  }

  closeEditor() {
    this.store.setState((state) => ({
      ...state,
      ui: {
        ...state.ui,
        editorOpen: false,
        currentNoteId: null,
        currentDraft: null,
        dirty: false
      }
    }));
  }

  updateDraft(document) {
    const note = this.#findCurrentNote();
    if (!note) {
      return;
    }

    const content = documentToNoteContent(document);
    this.store.setState((state) => ({
      ...state,
      ui: {
        ...state.ui,
        currentDraft: {
          ...document,
          id: note.id,
          content
        },
        dirty: content !== note.content
      }
    }));
  }

  async saveCurrentDraft() {
    const state = this.getState();
    const note = this.#findCurrentNote();
    const draft = state.ui.currentDraft;
    const adapter = state.source.adapter;

    if (!note || !draft || !adapter) {
      return true;
    }

    if (!state.source.capabilities.canWrite) {
      return false;
    }

    const payload = {
      ...note,
      content: buildMarkdownDocument(draft),
      updatedAt: new Date().toISOString()
    };

    const updated = note.id.startsWith('draft-')
      ? await adapter.createNote({ ...payload, id: undefined })
      : await adapter.updateNote(payload);

    await this.reloadNotes();
    if (updated && updated.id) {
      this.openNote(updated.id);
    }
    this.store.setState((currentState) => ({
      ...currentState,
      ui: {
        ...currentState.ui,
        dirty: false
      }
    }));
    return true;
  }

  async createNoteAndOpen() {
    const { source } = this.getState();
    if (!source.capabilities.canCreate) {
      return false;
    }

    const draftId = `draft-${crypto.randomUUID()}`;
    const draftNote = {
      id: draftId,
      content: '# Nouvelle page\n',
      pinned: false,
      writable: source.capabilities.canWrite,
      sourceKind: source.type,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    };

    this.store.setState((state) => {
      const items = [draftNote, ...state.notes.items];
      return {
        ...state,
        notes: {
          items,
          filteredItems: this.#filterNotes(items, state.ui.searchQuery)
        }
      };
    });

    this.openNote(draftId);
    return true;
  }

  async deleteNote(noteId) {
    const { source } = this.getState();
    if (!source.capabilities.canDelete) {
      return false;
    }

    await source.adapter.deleteNote(noteId, this.getState().notes.items);
    await this.reloadNotes();

    if (this.getState().ui.currentNoteId === noteId) {
      this.closeEditor();
    }
    return true;
  }

  async togglePinned(noteId) {
    const note = this.#findNote(noteId);
    if (!note) {
      return false;
    }

    const nextPinned = getNextPinnedValue(note);
    const applied = await this.getState().source.adapter.togglePinned(noteId, nextPinned, this.getState().notes.items);
    if (!applied) {
      return false;
    }
    await this.reloadNotes();
    const refreshed = this.#findNote(noteId);
    return refreshed?.pinned === nextPinned;
  }

  async reloadNotes() {
    const adapter = this.getState().source.adapter;
    const notes = await adapter.listNotes();
    this.#setNotes(notes);
  }

  #setSource(adapter) {
    const capabilities = adapter.capabilities();
    this.store.setState((state) => ({
      ...state,
      source: {
        adapter,
        type: capabilities.type,
        label: capabilities.label,
        capabilities
      }
    }));
  }

  #setNotes(notes) {
    this.store.setState((state) => ({
      ...state,
      notes: {
        items: notes,
        filteredItems: this.#filterNotes(notes, state.ui.searchQuery)
      }
    }));
  }

  #filterNotes(items, query) {
    const normalizedQuery = String(query || '').trim().toLowerCase();
    if (!normalizedQuery) {
      return [...items];
    }

    return items.filter((note) => {
      const haystack = `${note.content} ${note.relativePath || ''}`.toLowerCase();
      return haystack.includes(normalizedQuery);
    });
  }

  #findCurrentNote() {
    const { currentNoteId } = this.getState().ui;
    return this.#findNote(currentNoteId);
  }

  #findNote(noteId) {
    if (!noteId) {
      return null;
    }
    return this.getState().notes.items.find((note) => note.id === noteId) || null;
  }
}
