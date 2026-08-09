export const ELEPHANTNOTE_ENTRY_DRAG_TYPE = 'application/x-elephantnote-entry'

let activeDraggedEntry = null

export const getEntryKind = (entry) => entry?.kind || entry?.type || ''

export const getEntryParentPath = (pathname = '') => {
  const parts = String(pathname || '').split('/').filter(Boolean)
  if (parts.length <= 1) return ''
  return parts.slice(0, -1).join('/')
}

export const normalizeDropDirectoryPath = (pathname = '') =>
  String(pathname || '').split('/').filter(Boolean).join('/')

export const serializeDraggedEntry = (entry) => JSON.stringify({
  kind: getEntryKind(entry),
  type: getEntryKind(entry),
  path: entry?.path || '',
  title: entry?.title || entry?.filename?.replace(/\.md$/i, '') || ''
})

const getTransfer = (value) => value?.dataTransfer || value || null

export const parseDraggedEntry = (eventOrTransfer) => {
  const transfer = getTransfer(eventOrTransfer)
  const raw = transfer?.getData?.(ELEPHANTNOTE_ENTRY_DRAG_TYPE)
  if (!raw) return activeDraggedEntry
  try {
    const parsed = JSON.parse(raw)
    return parsed?.path ? parsed : activeDraggedEntry
  } catch (error) {
    console.warn('[library:dnd] failed to parse dragged entry payload', { error: error?.message || String(error) })
    return activeDraggedEntry
  }
}

export const writeDraggedEntry = (event, entry) => {
  if (!event?.dataTransfer || !entry?.path) return
  activeDraggedEntry = {
    kind: getEntryKind(entry),
    type: getEntryKind(entry),
    path: entry.path,
    title: entry?.title || entry?.filename?.replace(/\.md$/i, '') || ''
  }
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.dropEffect = 'move'
  event.dataTransfer.setData(ELEPHANTNOTE_ENTRY_DRAG_TYPE, serializeDraggedEntry(activeDraggedEntry))
  console.info('[library:dnd] drag:start', activeDraggedEntry)
}

export const clearDraggedEntry = () => {
  console.info('[library:dnd] drag:clear', activeDraggedEntry)
  activeDraggedEntry = null
}

export const canDropEntryOnDirectory = (entry, targetDirectoryPath = '') => {
  const sourcePath = normalizeDropDirectoryPath(entry?.path)
  const targetPath = normalizeDropDirectoryPath(targetDirectoryPath)
  if (!sourcePath) return false
  if (getEntryParentPath(sourcePath) === targetPath) return false
  if (getEntryKind(entry) === 'folder') {
    if (targetPath === sourcePath) return false
    if (targetPath.startsWith(`${sourcePath}/`)) return false
  }
  return true
}
