import { editorCommands } from '../bridge'

export const commandForBeforeInput = (event) => {
  switch (event.inputType) {
    case 'insertText':
    case 'insertReplacementText':
      return typeof event.data === 'string' ? editorCommands.insertText(event.data) : null
    case 'insertParagraph':
    case 'insertLineBreak':
      return editorCommands.insertParagraph()
    case 'deleteContentBackward':
    case 'deleteContent':
    case 'deleteByCut':
    case 'deleteByDrag':
      return editorCommands.deleteBackward()
    case 'historyUndo':
      return editorCommands.undo()
    case 'historyRedo':
      return editorCommands.redo()
    case 'formatBold':
      return editorCommands.toggleStrong()
    case 'formatItalic':
      return editorCommands.toggleEmphasis()
    case 'formatStrikeThrough':
      return editorCommands.toggleStrike()
    default:
      return null
  }
}

export const commandForTableKey = (event, renderer, selection) => {
  if (event.key !== 'Tab' || !selection) return null
  if (!selectionInsideTable(renderer, selection)) return null
  return event.shiftKey ? editorCommands.previousTableCell() : editorCommands.nextTableCell()
}

const selectionInsideTable = (renderer, selection) => {
  return [selection.anchor, selection.focus].every((point) => hasTableAncestor(renderer, point.node))
}

const hasTableAncestor = (renderer, nodeId) => {
  let current = renderer.logical.node(nodeId)
  while (current) {
    if (current.kind?.layer === 'block' && current.kind?.value?.type === 'table') return true
    current = current.parent ? renderer.logical.node(current.parent) : null
  }
  return false
}
