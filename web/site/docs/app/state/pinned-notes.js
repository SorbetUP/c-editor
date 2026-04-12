export function getNextPinnedValue(note) {
  if (!note) {
    return null;
  }

  return !Boolean(note.pinned);
}

export function applyPinnedValue(note, pinned, updatedAt = new Date().toISOString()) {
  return {
    ...note,
    pinned: Boolean(pinned),
    updatedAt
  };
}

export function applyPinnedValueToCollection(notes, noteId, pinned, updatedAt = new Date().toISOString()) {
  let changed = false;
  const updatedNotes = notes.map((note) => {
    if (note.id !== noteId) {
      return note;
    }
    changed = true;
    return applyPinnedValue(note, pinned, updatedAt);
  });

  return { changed, notes: updatedNotes };
}

export function applyPinnedMeta(meta, note, pinned) {
  if (!note?.relativePath) {
    return { changed: false, meta };
  }

  return {
    changed: true,
    meta: {
      ...meta,
      [note.relativePath]: {
        ...(meta[note.relativePath] || {}),
        pinned: Boolean(pinned)
      }
    }
  };
}
