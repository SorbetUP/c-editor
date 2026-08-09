import { editorCommands } from '../../editor-rust/protocol'

const normalize = (value) => String(value || '').trim().toLowerCase()

export const rustFormatCommand = (type) => {
  switch (normalize(type)) {
    case 'strong':
    case 'bold':
      return editorCommands.toggleStrong()
    case 'em':
    case 'emphasis':
    case 'italic':
      return editorCommands.toggleEmphasis()
    case 'del':
    case 'strike':
    case 'strikethrough':
      return editorCommands.toggleStrike()
    default:
      return null
  }
}

export const rustParagraphCommand = (type) => {
  const value = normalize(type)
  if (value === 'paragraph' || value === 'p') return editorCommands.setParagraph()
  if (value === 'blockquote') return editorCommands.toggleBlockQuote()
  if (value === 'pre') return editorCommands.toggleCodeBlock()
  if (value === 'ul-bullet') return editorCommands.setListKind('unordered')
  if (value === 'ul-task') return editorCommands.setListKind('task')
  if (value === 'ol-order') return editorCommands.setListKind('ordered')
  const match = value.match(/^heading(?:\s+|[-_])?([1-6])$/)
  return match ? editorCommands.setHeading(Number(match[1])) : null
}

export const rustBusCommand = (event, payload) => {
  switch (event) {
    case 'undo':
      return editorCommands.undo()
    case 'redo':
      return editorCommands.redo()
    case 'format':
      return rustFormatCommand(payload)
    case 'paragraph':
      return rustParagraphCommand(payload)
    case 'duplicate':
      return editorCommands.duplicateBlock()
    case 'deleteParagraph':
      return editorCommands.deleteBlock()
    case 'createParagraph':
      return editorCommands.insertParagraphAfterBlock()
    case 'insertParagraph':
      return editorCommands.insertParagraph()
    case 'insert-horizontal-rule':
      return editorCommands.insertHorizontalRule()
    case 'createTable':
      return editorCommands.createTable(payload?.rows, payload?.columns)
    case 'insert-image': {
      const image = typeof payload === 'string' ? { source: payload } : payload || {}
      return editorCommands.insertImage(
        image.source || image.src || '',
        image.alt || '',
        image.title || null
      )
    }
    case 'replace-image':
      return editorCommands.replaceImage(
        payload?.image,
        payload?.source || payload?.src || '',
        payload?.alt || '',
        payload?.title || null
      )
    case 'delete-image':
      return editorCommands.deleteImage(payload?.image)
    default:
      return null
  }
}
