import { editorCommands } from '../bridge'
import { markdownFromClipboard, TEXT_TRANSFER_TYPES } from './clipboard'

const transferTypes = (transfer) => Array.from(transfer?.types || [])

const transferHasFiles = (transfer) =>
  Boolean(transfer?.files?.length) || transferTypes(transfer).includes('Files')

const transferHasUri = (transfer) => transferTypes(transfer).includes('text/uri-list')

const transferHasText = (transfer) => {
  const types = transferTypes(transfer)
  return TEXT_TRANSFER_TYPES.some((type) => types.includes(type))
}

const caretRangeFromPoint = (ownerDocument, x, y) => {
  if (typeof ownerDocument.caretRangeFromPoint === 'function') {
    return ownerDocument.caretRangeFromPoint(x, y)
  }
  if (typeof ownerDocument.caretPositionFromPoint !== 'function') return null
  const position = ownerDocument.caretPositionFromPoint(x, y)
  if (!position?.offsetNode) return null
  const range = ownerDocument.createRange()
  range.setStart(position.offsetNode, position.offset)
  range.collapse(true)
  return range
}

const placeDropCaret = (controller, event) => {
  const ownerDocument = controller.container.ownerDocument
  const range = caretRangeFromPoint(ownerDocument, event.clientX || 0, event.clientY || 0)
  if (!range || !controller.container.contains(range.startContainer)) return
  const selection = ownerDocument.defaultView?.getSelection?.()
  if (!selection) return
  selection.removeAllRanges()
  selection.addRange(range)
}

const claimDrop = (event) => {
  event.preventDefault()
  event.stopPropagation?.()
  try {
    event.dataTransfer.dropEffect = 'copy'
  } catch {
    // Some DataTransfer implementations expose a read-only dropEffect.
  }
}

const droppedUri = (transfer) => String(transfer.getData?.('text/uri-list') || '')
  .split(/\r?\n/)
  .map((line) => line.trim())
  .find((line) => line && !line.startsWith('#')) || ''

export const handleDragOver = (controller, event) => {
  const transfer = event.dataTransfer
  if (!transfer) return false
  if (transferHasFiles(transfer)) {
    if (!controller.onFileDrop) return false
    claimDrop(event)
    return true
  }
  if (transferHasUri(transfer)) {
    if (!controller.onUriDrop) return false
    claimDrop(event)
    return true
  }
  if (!transferHasText(transfer)) return false
  claimDrop(event)
  return true
}

export const handleDrop = (controller, event) => {
  const transfer = event.dataTransfer
  if (!transfer) return false
  if (transferHasFiles(transfer)) return handleFileDrop(controller, event)
  if (transferHasUri(transfer)) return handleUriDrop(controller, event)
  if (!transferHasText(transfer)) return false

  placeDropCaret(controller, event)
  const selection = controller.readSelection()
  if (!selection) return false
  const markdown = markdownFromClipboard(event, controller.container.ownerDocument)
  if (markdown === null) return false
  claimDrop(event)
  controller.schedule(async () => {
    await controller.bridge.setSelection(selection)
    await controller.bridge.dispatch(editorCommands.pasteMarkdown(markdown))
  })
  return true
}

const handleFileDrop = (controller, event) => {
  if (!controller.onFileDrop) return false
  const files = Array.from(event.dataTransfer?.files || [])
  if (!files.length) return false
  return delegateDrop(controller, event, () => controller.onFileDrop(files))
}

const handleUriDrop = (controller, event) => {
  if (!controller.onUriDrop) return false
  const uri = droppedUri(event.dataTransfer)
  if (!uri) return false
  return delegateDrop(controller, event, () => controller.onUriDrop(uri))
}

const delegateDrop = (controller, event, callback) => {
  placeDropCaret(controller, event)
  const selection = controller.readSelection()
  if (!selection) return false
  claimDrop(event)
  controller.schedule(async () => {
    await controller.bridge.setSelection(selection)
    await callback()
  })
  return true
}
