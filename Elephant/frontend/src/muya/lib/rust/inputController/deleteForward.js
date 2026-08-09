import { editorCommands } from '../bridge'

const collapsed = (point) => ({ anchor: point, focus: point })

const isCollapsed = (selection) =>
  selection.anchor.node === selection.focus.node &&
  selection.anchor.offset_utf16 === selection.focus.offset_utf16

const textValue = (node) =>
  node?.kind?.layer === 'inline' && node.kind?.value?.type === 'text'
    ? String(node.kind.value.value || '')
    : null

const nextGraphemeEnd = (value, offset) => {
  if (offset >= value.length) return null
  if (typeof Intl?.Segmenter === 'function') {
    const segments = new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(value)
    for (const segment of segments) {
      const end = segment.index + segment.segment.length
      if (segment.index <= offset && offset < end) return end
      if (segment.index > offset) return end
    }
  }
  const next = Array.from(value.slice(offset))[0]
  return next ? offset + next.length : null
}

const firstTextDescendant = (logical, rootId) => {
  const root = logical.node(rootId)
  if (!root) return null
  if (textValue(root) !== null) return root
  for (const child of root.children || []) {
    const text = firstTextDescendant(logical, child)
    if (text) return text
  }
  return null
}

const nextParagraphText = (logical, textNode) => {
  const block = logical.node(textNode.parent)
  if (block?.kind?.layer !== 'block' || block.kind?.value?.type !== 'paragraph') return null
  const parent = logical.node(block.parent)
  if (!parent) return null
  const index = parent.children.indexOf(block.id)
  if (index < 0) return null
  const nextBlock = logical.node(parent.children[index + 1])
  if (nextBlock?.kind?.layer !== 'block' || nextBlock.kind?.value?.type !== 'paragraph') {
    return null
  }
  return firstTextDescendant(logical, nextBlock.id)
}

export const planDeleteForward = (renderer, selection) => {
  if (!selection) return null
  if (!isCollapsed(selection)) {
    return { selection, command: editorCommands.deleteBackward() }
  }

  const point = selection.focus
  const node = renderer.logical.node(point.node)
  const value = textValue(node)
  if (value === null) return null
  const end = nextGraphemeEnd(value, point.offset_utf16)
  if (end !== null) {
    return {
      selection: collapsed({ node: point.node, offset_utf16: end }),
      command: editorCommands.deleteBackward()
    }
  }

  const next = nextParagraphText(renderer.logical, node)
  if (!next) return null
  return {
    selection: collapsed({ node: next.id, offset_utf16: 0 }),
    command: editorCommands.deleteBackward()
  }
}

export const handleDeleteForward = (controller, event, selection) => {
  event.preventDefault()
  const plan = planDeleteForward(controller.renderer, selection)
  if (!plan) return false
  controller.schedule(async () => {
    await controller.bridge.setSelection(plan.selection)
    await controller.bridge.dispatch(plan.command)
  })
  return true
}
