import {
  buildMarkdownDocument,
  buildMarkdownFilename,
  getMarkdownDocumentTitle
} from '../models/markdown-document.js';
import { applyPinnedMeta, applyPinnedValueToCollection } from '../state/pinned-notes.js';

const NOTES_STORAGE_KEY = 'elephantnote-notes-v2';
const DIRECTORY_META_KEY = 'elephantnote-directory-meta-v2';

const DEFAULT_NOTES = [
  {
    title: 'Nouvelle page',
    body: [
      'Paragraphe simple pour verifier le rendu de base et la stabilite generale du moteur.',
      '',
      '## Titres',
      '### Niveau 3',
      '#### Niveau 4',
      '',
      '## Tableau',
      '| Bloc | Etat | Note |',
      '| --- | --- | --- |',
      '| Rendu | OK | Verification visuelle |',
      '| Edition | OK | Iteration locale |'
    ].join('\n'),
    pinned: true
  },
  {
    title: 'Checklist',
    body: [
      '- [ ] Ouvrir un dossier',
      '- [ ] Modifier une note',
      '- [ ] Creer une note',
      '- [ ] Verifier la sauvegarde'
    ].join('\n')
  },
  {
    title: 'Rendu complet',
    body: [
      'Paragraphe avec **gras**, *italique*, `code`, ==surlignage== et [lien](https://example.com).',
      '',
      '> Citation de test',
      '',
      '![Image de test](https://placehold.co/640x240/png)'
    ].join('\n')
  }
];

function readJsonStorage(key, fallback) {
  try {
    const raw = localStorage.getItem(key);
    return raw ? JSON.parse(raw) : fallback;
  } catch (error) {
    console.warn(`Unable to read ${key}:`, error);
    return fallback;
  }
}

function writeJsonStorage(key, value) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch (error) {
    console.warn(`Unable to write ${key}:`, error);
  }
}

function createNoteRecord(overrides = {}) {
  const content = overrides.content || buildMarkdownDocument({
    title: overrides.title || 'Nouvelle page',
    body: overrides.body || ''
  });

  const now = new Date().toISOString();
  return {
    id: overrides.id || `note-${crypto.randomUUID()}`,
    content,
    pinned: overrides.pinned === true,
    fileName: overrides.fileName || null,
    relativePath: overrides.relativePath || '',
    parentPath: overrides.parentPath || '',
    fileHandle: overrides.fileHandle || null,
    parentHandle: overrides.parentHandle || null,
    writable: overrides.writable !== false,
    sourceKind: overrides.sourceKind || 'local-storage',
    createdAt: overrides.createdAt || now,
    updatedAt: overrides.updatedAt || now
  };
}

function loadDirectoryMeta() {
  return readJsonStorage(DIRECTORY_META_KEY, {});
}

function saveDirectoryMeta(meta) {
  writeJsonStorage(DIRECTORY_META_KEY, meta);
}

async function writeFile(fileHandle, content) {
  const writable = await fileHandle.createWritable();
  await writable.write(content);
  await writable.close();
}

async function fileExists(directoryHandle, name) {
  try {
    await directoryHandle.getFileHandle(name);
    return true;
  } catch (error) {
    if (error && error.name === 'NotFoundError') {
      return false;
    }
    throw error;
  }
}

async function resolveUniqueFilename(directoryHandle, desiredFilename, currentFilename = null) {
  const dotIndex = desiredFilename.lastIndexOf('.');
  const stem = dotIndex >= 0 ? desiredFilename.slice(0, dotIndex) : desiredFilename;
  const extension = dotIndex >= 0 ? desiredFilename.slice(dotIndex) : '.md';
  let candidate = desiredFilename;
  let iteration = 2;

  while (candidate !== currentFilename && await fileExists(directoryHandle, candidate)) {
    candidate = `${stem} (${iteration})${extension}`;
    iteration += 1;
  }

  return candidate;
}

async function collectWritableDirectoryNotes(directoryHandle, meta, parentPath = '') {
  const entries = [];
  for await (const [name, handle] of directoryHandle.entries()) {
    entries.push({ name, handle });
  }

  entries.sort((a, b) => {
    if (a.handle.kind !== b.handle.kind) {
      return a.handle.kind === 'directory' ? -1 : 1;
    }
    return a.name.localeCompare(b.name, 'fr', { sensitivity: 'base' });
  });

  const notes = [];

  for (const entry of entries) {
    if (entry.handle.kind === 'directory') {
      const nextParentPath = parentPath ? `${parentPath}/${entry.name}` : entry.name;
      notes.push(...await collectWritableDirectoryNotes(entry.handle, meta, nextParentPath));
      continue;
    }

    if (!/\.(md|markdown|txt)$/i.test(entry.name)) {
      continue;
    }

    const relativePath = parentPath ? `${parentPath}/${entry.name}` : entry.name;
    const file = await entry.handle.getFile();
    notes.push(createNoteRecord({
      content: await file.text(),
      pinned: meta[relativePath]?.pinned === true,
      fileName: entry.name,
      relativePath,
      parentPath,
      fileHandle: entry.handle,
      parentHandle: directoryHandle,
      writable: true,
      sourceKind: 'web-directory-readwrite',
      createdAt: new Date(file.lastModified).toISOString(),
      updatedAt: new Date(file.lastModified).toISOString()
    }));
  }

  return notes;
}

async function pickReadonlyDirectoryFiles() {
  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true;
    input.setAttribute('webkitdirectory', '');
    input.setAttribute('directory', '');
    input.accept = '.md,.markdown,.txt';
    input.style.display = 'none';

    input.addEventListener('change', () => {
      const files = Array.from(input.files || []);
      document.body.removeChild(input);
      resolve(files);
    }, { once: true });

    input.addEventListener('cancel', () => {
      document.body.removeChild(input);
      reject(new DOMException('Selection canceled', 'AbortError'));
    }, { once: true });

    document.body.appendChild(input);
    input.click();
  });
}

async function collectReadonlyDirectoryNotes(files, meta) {
  const markdownFiles = Array.from(files || [])
    .filter((file) => /\.(md|markdown|txt)$/i.test(file.name))
    .sort((a, b) => (a.webkitRelativePath || a.name).localeCompare(b.webkitRelativePath || b.name, 'fr', { sensitivity: 'base' }));

  if (markdownFiles.length === 0) {
    return { rootName: 'Dossier', notes: [] };
  }

  const rootName = (markdownFiles[0].webkitRelativePath || '').split('/')[0] || 'Dossier';
  const notes = [];

  for (const file of markdownFiles) {
    const rawRelativePath = file.webkitRelativePath || file.name;
    const relativePath = rawRelativePath.startsWith(`${rootName}/`)
      ? rawRelativePath.slice(rootName.length + 1)
      : rawRelativePath;
    const segments = relativePath.split('/');
    notes.push(createNoteRecord({
      content: await file.text(),
      pinned: meta[relativePath]?.pinned === true,
      fileName: segments[segments.length - 1],
      relativePath,
      parentPath: segments.slice(0, -1).join('/'),
      writable: false,
      sourceKind: 'web-directory-readonly',
      createdAt: new Date(file.lastModified).toISOString(),
      updatedAt: new Date(file.lastModified).toISOString()
    }));
  }

  return { rootName, notes };
}

export class LocalStorageVaultAdapter {
  constructor() {
    this.key = NOTES_STORAGE_KEY;
  }

  capabilities() {
    return {
      type: 'local-storage',
      label: 'Local',
      canRead: true,
      canWrite: true,
      canDelete: true,
      canCreate: true
    };
  }

  async listNotes() {
    const stored = readJsonStorage(this.key, null);
    if (Array.isArray(stored) && stored.length > 0) {
      return stored.map((note) => createNoteRecord({
        ...note,
        pinned: note.pinned === true,
        writable: true,
        sourceKind: 'local-storage'
      }));
    }

    const seeded = DEFAULT_NOTES.map((note, index) => createNoteRecord({
      title: note.title,
      body: note.body,
      pinned: note.pinned === true || index === 0,
      writable: true,
      sourceKind: 'local-storage'
    }));
    await this.#persist(seeded);
    return seeded;
  }

  async createNote(note) {
    const notes = await this.listNotes();
    const created = createNoteRecord({
      ...note,
      writable: true,
      sourceKind: 'local-storage'
    });
    notes.unshift(created);
    await this.#persist(notes);
    return created;
  }

  async updateNote(note) {
    const notes = await this.listNotes();
    const updated = notes.map((entry) => (
      entry.id === note.id
        ? { ...entry, ...note, writable: true, sourceKind: 'local-storage', updatedAt: new Date().toISOString() }
        : entry
    ));
    await this.#persist(updated);
    return updated.find((entry) => entry.id === note.id);
  }

  async deleteNote(noteId) {
    const notes = await this.listNotes();
    const filtered = notes.filter((entry) => entry.id !== noteId);
    await this.#persist(filtered);
  }

  async togglePinned(noteId, value) {
    const notes = await this.listNotes();
    const { changed, notes: updated } = applyPinnedValueToCollection(notes, noteId, value);
    if (!changed) {
      return false;
    }
    await this.#persist(updated);
    return true;
  }

  async importFile(file) {
    const content = await file.text();
    return this.createNote({
      content,
      fileName: file.name,
      writable: true,
      sourceKind: 'local-storage'
    });
  }

  async #persist(notes) {
    writeJsonStorage(this.key, notes.map((note) => ({
      id: note.id,
      content: note.content,
      pinned: note.pinned,
      createdAt: note.createdAt,
      updatedAt: note.updatedAt
    })));
  }
}

export class WebDirectoryAdapter {
  static async open() {
    if (typeof window.showDirectoryPicker !== 'function') {
      return null;
    }

    const handle = await window.showDirectoryPicker({ mode: 'readwrite' });
    return new WebDirectoryAdapter(handle);
  }

  constructor(directoryHandle) {
    this.directoryHandle = directoryHandle;
    this.meta = loadDirectoryMeta();
  }

  capabilities() {
    return {
      type: 'web-directory-readwrite',
      label: this.directoryHandle.name,
      canRead: true,
      canWrite: true,
      canDelete: true,
      canCreate: true
    };
  }

  async listNotes() {
    this.meta = loadDirectoryMeta();
    return collectWritableDirectoryNotes(this.directoryHandle, this.meta);
  }

  async createNote(note) {
    const content = note.content || buildMarkdownDocument({ title: note.title, body: note.body });
    const desiredFilename = buildMarkdownFilename(getMarkdownDocumentTitle(content, 'Nouvelle page'));
    const finalFilename = await resolveUniqueFilename(this.directoryHandle, desiredFilename);
    const fileHandle = await this.directoryHandle.getFileHandle(finalFilename, { create: true });
    await writeFile(fileHandle, content);

    const created = createNoteRecord({
      ...note,
      content,
      fileHandle,
      fileName: finalFilename,
      parentHandle: this.directoryHandle,
      relativePath: finalFilename,
      parentPath: '',
      writable: true,
      sourceKind: 'web-directory-readwrite'
    });

    if (created.relativePath) {
      this.meta[created.relativePath] = { pinned: created.pinned === true };
      saveDirectoryMeta(this.meta);
    }

    return created;
  }

  async updateNote(note) {
    const content = note.content || buildMarkdownDocument({ title: note.title, body: note.body });
    const currentFilename = note.fileName || buildMarkdownFilename(getMarkdownDocumentTitle(content, 'Nouvelle page'));
    const desiredFilename = buildMarkdownFilename(getMarkdownDocumentTitle(content, 'Nouvelle page'));
    const parentHandle = note.parentHandle || this.directoryHandle;
    const finalFilename = await resolveUniqueFilename(parentHandle, desiredFilename, currentFilename);
    const oldRelativePath = note.relativePath || currentFilename;

    if (note.fileHandle && currentFilename === finalFilename) {
      await writeFile(note.fileHandle, content);
    } else {
      const nextHandle = await parentHandle.getFileHandle(finalFilename, { create: true });
      await writeFile(nextHandle, content);
      if (note.fileName) {
        await parentHandle.removeEntry(note.fileName);
      }
      note.fileHandle = nextHandle;
    }

    const updated = {
      ...note,
      content,
      fileName: finalFilename,
      relativePath: note.parentPath ? `${note.parentPath}/${finalFilename}` : finalFilename,
      parentPath: note.parentPath || '',
      fileHandle: note.fileHandle || await parentHandle.getFileHandle(finalFilename),
      parentHandle,
      writable: true,
      sourceKind: 'web-directory-readwrite',
      updatedAt: new Date().toISOString()
    };

    if (oldRelativePath && oldRelativePath !== updated.relativePath) {
      delete this.meta[oldRelativePath];
    }
    this.meta[updated.relativePath] = { pinned: updated.pinned === true };
    saveDirectoryMeta(this.meta);

    return updated;
  }

  async deleteNote(noteId, notes) {
    const note = notes.find((entry) => entry.id === noteId);
    if (!note || !note.fileName) {
      return;
    }
    const parentHandle = note.parentHandle || this.directoryHandle;
    await parentHandle.removeEntry(note.fileName);
    delete this.meta[note.relativePath];
    saveDirectoryMeta(this.meta);
  }

  async togglePinned(noteId, value, notes) {
    const note = notes.find((entry) => entry.id === noteId);
    const { changed, meta } = applyPinnedMeta(this.meta, note, value);
    if (!changed) {
      return false;
    }
    this.meta = meta;
    saveDirectoryMeta(this.meta);
    return true;
  }

  async importFile(file) {
    const note = createNoteRecord({
      content: await file.text(),
      writable: true,
      sourceKind: 'web-directory-readwrite'
    });
    return this.createNote(note);
  }
}

export class WebDirectoryReadonlyAdapter {
  static async open() {
    const files = await pickReadonlyDirectoryFiles();
    return new WebDirectoryReadonlyAdapter(files);
  }

  constructor(files) {
    this.files = files;
    this.meta = loadDirectoryMeta();
    this.rootName = 'Dossier';
  }

  capabilities() {
    return {
      type: 'web-directory-readonly',
      label: this.rootName,
      canRead: true,
      canWrite: false,
      canDelete: false,
      canCreate: false
    };
  }

  async listNotes() {
    this.meta = loadDirectoryMeta();
    const payload = await collectReadonlyDirectoryNotes(this.files, this.meta);
    this.rootName = payload.rootName;
    return payload.notes;
  }

  async createNote() {
    throw new Error('Readonly source');
  }

  async updateNote() {
    throw new Error('Readonly source');
  }

  async deleteNote() {
    throw new Error('Readonly source');
  }

  async togglePinned(noteId, value, notes) {
    const note = notes.find((entry) => entry.id === noteId);
    const { changed, meta } = applyPinnedMeta(this.meta, note, value);
    if (!changed) {
      return false;
    }
    this.meta = meta;
    saveDirectoryMeta(this.meta);
    return true;
  }

  async importFile() {
    throw new Error('Readonly source');
  }
}

export class NativeFilesystemAdapter {
  capabilities() {
    return {
      type: 'native-directory',
      label: 'Natif',
      canRead: false,
      canWrite: false,
      canDelete: false,
      canCreate: false
    };
  }

  async listNotes() {
    throw new Error('Native adapter not implemented');
  }
}
