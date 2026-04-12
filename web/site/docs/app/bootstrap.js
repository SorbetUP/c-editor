import '../components/hybrid-note-editor.js';
import './ui/source-status.js';
import './ui/notes-library.js';
import { UI_INTENT } from './ui/ui-intents.js';
import { NotesStore } from './state/notes-store.js';
import { EditorEngineService } from './services/editor-engine-service.js';
import { MarkdownRenderer } from './services/markdown-renderer.js';
import { buildMarkdownDocument } from './models/markdown-document.js';

function registerServiceWorker() {
  if (!('serviceWorker' in navigator)) {
    return;
  }

  window.addEventListener('load', async () => {
    try {
      const registrations = await navigator.serviceWorker.getRegistrations();
      await Promise.all(registrations.map((registration) => registration.unregister()));
    } catch (error) {
      console.warn('Service worker cleanup failed:', error);
    }
  });
}

async function pickSingleMarkdownFile() {
  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.md,.markdown,.txt';
    input.style.display = 'none';

    input.addEventListener('change', () => {
      const [file] = Array.from(input.files || []);
      document.body.removeChild(input);
      resolve(file || null);
    }, { once: true });

    input.addEventListener('cancel', () => {
      document.body.removeChild(input);
      reject(new DOMException('Selection canceled', 'AbortError'));
    }, { once: true });

    document.body.appendChild(input);
    input.click();
  });
}

document.addEventListener('DOMContentLoaded', async () => {
  registerServiceWorker();

  const notesLibrary = document.getElementById('notesLibrary');
  const editorWorkspace = document.getElementById('editorWorkspace');
  const noteEditor = document.getElementById('noteEditor');
  const notesBrowser = document.getElementById('notesBrowser') || notesLibrary || document.querySelector('notes-library');

  const store = new NotesStore();
  const renderer = new MarkdownRenderer(new EditorEngineService(), { maxEntries: 80 });
  let lastLoadedNoteId = null;
  let lastLoadedSignature = null;
  let lastEditorOpen = false;

  const closeEditorIfPossible = async () => {
    const saved = await store.saveCurrentDraft();
    if (saved) {
      store.closeEditor();
      return true;
    }
    window.alert('Impossible de fermer avec des modifications non enregistrables.');
    return false;
  };

  const openNoteIfPossible = async (noteId) => {
    const state = store.getState();
    if (state.ui.editorOpen && state.ui.currentNoteId && state.ui.currentNoteId !== noteId) {
      const saved = await store.saveCurrentDraft();
      if (!saved) {
        window.alert('Impossible de changer de note tant que la source est en lecture seule.');
        return false;
      }
    }

    return store.openNote(noteId);
  };

  const openSourceIfPossible = async () => {
    if (store.getState().ui.editorOpen) {
      const saved = await store.saveCurrentDraft();
      if (!saved) {
        window.alert('Impossible de changer de source tant que des modifications ne peuvent pas etre enregistrees.');
        return false;
      }
    }

    await store.openBestDirectorySource();
    return true;
  };

  const syncUi = (state) => {
    document.body.classList.toggle('editor-open', Boolean(state.ui.editorOpen));

    if (typeof notesLibrary?.setState === 'function') {
      notesLibrary.setState(state);
    }

    const currentNote = state.notes.items.find((note) => note.id === state.ui.currentNoteId) || null;
    if (!editorWorkspace || !noteEditor) {
      return;
    }

    if (state.ui.editorOpen && currentNote && state.ui.currentDraft) {
      const draftSignature = `${state.ui.currentDraft.id || currentNote.id}:${buildMarkdownDocument(state.ui.currentDraft)}`;
      const shouldReload = lastLoadedNoteId !== currentNote.id || (!state.ui.dirty && lastLoadedSignature !== draftSignature);

      if (shouldReload) {
        noteEditor.loadDocument(state.ui.currentDraft);
        lastLoadedNoteId = currentNote.id;
        lastLoadedSignature = draftSignature;
      }

      editorWorkspace.hidden = false;
      if (notesBrowser) {
        notesBrowser.classList.add('is-blurred');
      }
      if (!lastEditorOpen) {
        noteEditor.focus();
      }
    } else {
      editorWorkspace.hidden = true;
      if (notesBrowser) {
        notesBrowser.classList.remove('is-blurred');
      }
      lastLoadedNoteId = null;
      lastLoadedSignature = null;
    }
    lastEditorOpen = state.ui.editorOpen;
  };

  if (!notesLibrary) {
    throw new Error('notesLibrary host not found');
  }

  if (typeof notesLibrary.setRenderer === 'function') {
    notesLibrary.setRenderer(renderer);
  }
  store.subscribe(syncUi);
  await store.initialize();
  syncUi(store.getState());

  if ('requestIdleCallback' in window) {
    window.requestIdleCallback(async () => {
      await renderer.warmUp();
      if (typeof notesLibrary.enableRichPreview === 'function') {
        notesLibrary.enableRichPreview();
      }
    });
  } else {
    setTimeout(async () => {
      await renderer.warmUp();
      if (typeof notesLibrary.enableRichPreview === 'function') {
        notesLibrary.enableRichPreview();
      }
    }, 1200);
  }

  notesLibrary.addEventListener(UI_INTENT.SEARCH_CHANGE, (event) => {
    store.setSearchQuery(event.detail.query);
  });

  notesLibrary.addEventListener(UI_INTENT.OPEN_NOTE, (event) => {
    void openNoteIfPossible(event.detail.noteId);
  });

  notesLibrary.addEventListener(UI_INTENT.TOGGLE_PIN, async (event) => {
    await store.togglePinned(event.detail.noteId);
  });

  notesLibrary.addEventListener(UI_INTENT.DELETE_NOTE, async (event) => {
    const success = await store.deleteNote(event.detail.noteId);
    if (!success) {
      window.alert('Suppression impossible dans ce mode.');
    }
  });

  notesLibrary.addEventListener(UI_INTENT.CREATE_NOTE, async () => {
    const success = await store.createNoteAndOpen();
    if (!success) {
      window.alert('Creation impossible dans cette source.');
    }
  });

  notesLibrary.addEventListener(UI_INTENT.OPEN_SOURCE, async () => {
    try {
      await openSourceIfPossible();
    } catch (error) {
      console.error('Unable to open source:', error);
      window.alert('Impossible d’ouvrir ce dossier.');
    }
  });

  notesLibrary.addEventListener(UI_INTENT.IMPORT_FILE, async () => {
    try {
      const file = await pickSingleMarkdownFile();
      if (!file) {
        return;
      }
      await store.importFile(file);
    } catch (error) {
      if (error && error.name === 'AbortError') {
        return;
      }
      console.error('Unable to import file:', error);
      window.alert('Impossible d’importer ce fichier.');
    }
  });

  noteEditor.addEventListener('change', (event) => {
    store.updateDraft(event.detail.document);
  });

  noteEditor.addEventListener(UI_INTENT.SAVE_NOTE, async () => {
    const saved = await store.saveCurrentDraft();
    if (!saved) {
      window.alert('Cette source est en lecture seule.');
    }
  });

  noteEditor.addEventListener('ready', () => {
    // no-op hook for future instrumentation
  });

  noteEditor.addEventListener(UI_INTENT.CLOSE_EDITOR, async () => {
    await closeEditorIfPossible();
  });

  editorWorkspace.addEventListener('click', async (event) => {
    if (event.target.closest('.editor-host-shell')) {
      return;
    }
    await closeEditorIfPossible();
  });

  document.addEventListener('keydown', async (event) => {
    if (event.key === 'Escape' && !editorWorkspace.hidden) {
      await closeEditorIfPossible();
    }
  });
});
