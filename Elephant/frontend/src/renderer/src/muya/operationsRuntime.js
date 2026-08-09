export const applyOperation = (text = '', operation = {}) => {
  const pos = clamp(operation.pos || 0, 0, text.length)
  if (operation.type === 'insert') return text.slice(0, pos) + (operation.text || '') + text.slice(pos)
  if (operation.type === 'delete') return text.slice(0, pos) + text.slice(pos + (operation.count || 0))
  if (operation.type === 'replace') return text.slice(0, pos) + (operation.text || '') + text.slice(pos + (operation.count || 0))
  return text
}

export const transformOperation = (operation = {}, against = {}) => {
  const next = { ...operation }
  if (against.type === 'insert' && against.pos <= next.pos) next.pos += String(against.text || '').length
  if (against.type === 'delete' && against.pos < next.pos) next.pos = Math.max(against.pos, next.pos - (against.count || 0))
  if (against.type === 'replace' && against.pos < next.pos) next.pos += String(against.text || '').length - (against.count || 0)
  return next
}

export const composeOperations = (text = '', operations = []) => operations.reduce((value, operation) => applyOperation(value, operation), text)

export const createGroupedHistory = () => ({ undo: [], redo: [], currentGroup: null })

export const pushGroupedHistory = (history, before, after, group = 'default') => {
  if (before === after) return history
  if (!history.currentGroup || history.currentGroup.name !== group) {
    history.currentGroup = { name: group, before, after, count: 1 }
    history.undo.push(history.currentGroup)
  } else {
    history.currentGroup.after = after
    history.currentGroup.count += 1
  }
  history.redo = []
  return history
}

export const closeHistoryGroup = (history) => {
  history.currentGroup = null
  return history
}

export const undoGroupedHistory = (history, current) => {
  const item = history.undo.pop()
  if (!item) return { markdown: current, history, changed: false }
  history.redo.push(item)
  history.currentGroup = null
  return { markdown: item.before, history, changed: true }
}

export const redoGroupedHistory = (history, current) => {
  const item = history.redo.pop()
  if (!item) return { markdown: current, history, changed: false }
  history.undo.push(item)
  history.currentGroup = null
  return { markdown: item.after, history, changed: true }
}

export const createOtTransaction = (before = '', operations = [], group = 'default') => ({
  before,
  operations,
  after: composeOperations(before, operations),
  group
})

const clamp = (value, min, max) => Math.max(min, Math.min(max, value))
