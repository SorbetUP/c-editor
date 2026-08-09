import { editorCommands } from '../bridge'
import { htmlToMarkdown } from './clipboard'

const MARKDOWN_TYPES = [
  'application/x-elephant-markdown',
  'application/x-muya-markdown',
  'application/x-markdown',
  'text/markdown'
]

const selectedHtml = (ownerDocument) => {
  const selection = ownerDocument.defaultView?.getSelection?.()
  if (!selection?.rangeCount) return ''
  const wrapper = ownerDocument.createElement('div')
  for (let index = 0; index < selection.rangeCount; index += 1) {
    wrapper.appendChild(selection.getRangeAt(index).cloneContents())
  }
  return wrapper.innerHTML
}

const escapeHtml = (value) => String(value || '')
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;')
  .replace(/'/g, '&#39;')

const wrapInlineMarkdown = (value, type) => {
  if (!value) return value
  switch (type) {
    case 'strong':
      return `**${value}**`
    case 'em':
      return `*${value}*`
    case 'del':
      return `~~${value}~~`
    case 'inline_code': {
      const delimiter = value.includes('`') ? '``' : '`'
      return `${delimiter}${value}${delimiter}`
    }
    default:
      return value
  }
}

const wrapInlineHtml = (value, type) => {
  if (!value) return value
  switch (type) {
    case 'strong':
      return `<strong>${value}</strong>`
    case 'em':
      return `<em>${value}</em>`
    case 'del':
      return `<del>${value}</del>`
    case 'inline_code':
      return `<code>${value}</code>`
    default:
      return value
  }
}

const logicalSelectionPayload = (controller, selection) => {
  const anchor = selection?.anchor
  const focus = selection?.focus
  if (!anchor || !focus || anchor.node !== focus.node) return null
  const logical = controller.renderer?.logical
  const node = logical?.node?.(anchor.node)
  const nodeKind = node?.kind?.value
  if (!node || node.kind?.layer !== 'inline' || nodeKind?.type !== 'text') return null
  const start = Math.min(Number(anchor.offset_utf16) || 0, Number(focus.offset_utf16) || 0)
  const end = Math.max(Number(anchor.offset_utf16) || 0, Number(focus.offset_utf16) || 0)
  if (start === end) return null
  const plain = String(nodeKind.value || '').slice(start, end)
  let markdown = plain
  let html = escapeHtml(plain)
  let parent = logical.node(node.parent)
  while (parent && parent.kind?.layer === 'inline') {
    const type = parent.kind?.value?.type
    markdown = wrapInlineMarkdown(markdown, type)
    html = wrapInlineHtml(html, type)
    parent = logical.node(parent.parent)
  }
  return { plain, markdown, html }
}

export const clipboardSelection = (controller) => {
  const ownerDocument = controller.container.ownerDocument
  const domSelection = ownerDocument.defaultView?.getSelection?.()
  if (!domSelection || domSelection.isCollapsed) return null
  const selection = controller.readSelection()
  const logicalPayload = logicalSelectionPayload(controller, selection)
  const plain = logicalPayload?.plain || domSelection.toString()
  const domHtml = selectedHtml(ownerDocument)
  const html = logicalPayload?.html || domHtml
  return {
    plain,
    html,
    markdown: logicalPayload?.markdown || htmlToMarkdown(ownerDocument, domHtml) || plain
  }
}

export const writeClipboardSelection = (event, payload) => {
  const clipboard = event.clipboardData
  if (!clipboard?.setData || !payload) return false
  for (const type of MARKDOWN_TYPES) clipboard.setData(type, payload.markdown)
  clipboard.setData('text/plain', payload.plain)
  if (payload.html) clipboard.setData('text/html', payload.html)
  event.preventDefault()
  event.stopPropagation?.()
  return true
}

export const handleCopy = (controller, event) =>
  writeClipboardSelection(event, clipboardSelection(controller))

export const handleCut = (controller, event) => {
  const selection = controller.readSelection()
  if (!selection) return false
  const payload = clipboardSelection(controller)
  if (!writeClipboardSelection(event, payload)) return false
  controller.schedule(async () => {
    await controller.bridge.setSelection(selection)
    await controller.bridge.dispatch(editorCommands.deleteBackward())
  })
  return true
}
