import { editorCommands } from '../bridge'
import { planDeleteForward } from './deleteForward'

export const DELETE_UNIT_INPUTS = new Set([
  'deleteWordBackward',
  'deleteWordForward',
  'deleteSoftLineBackward',
  'deleteSoftLineForward',
  'deleteHardLineBackward',
  'deleteHardLineForward',
  'deleteEntireSoftLine'
])

const isCollapsed = (selection) =>
  selection.anchor.node === selection.focus.node &&
  selection.anchor.offset_utf16 === selection.focus.offset_utf16

const textValue = (renderer, nodeId) => {
  const node = renderer.logical.node(nodeId)
  return node?.kind?.layer === 'inline' && node.kind?.value?.type === 'text'
    ? String(node.kind.value.value || '')
    : null
}

const range = (node, start, end) => ({
  anchor: { node, offset_utf16: start },
  focus: { node, offset_utf16: end }
})

const wordSegments = (value) => {
  if (typeof Intl?.Segmenter === 'function') {
    return Array.from(
      new Intl.Segmenter(undefined, { granularity: 'word' }).segment(value)
    )
  }
  return Array.from(value.matchAll(/\s+|[^\s]+/gu), (match) => ({
    index: match.index,
    segment: match[0],
    isWordLike: /\S/u.test(match[0])
  }))
}

const previousWordStart = (value, offset) => {
  const prefix = value.slice(0, offset)
  const segments = wordSegments(prefix)
  let start = offset
  for (let index = segments.length - 1; index >= 0; index -= 1) {
    const segment = segments[index]
    start = segment.index
    if (segment.isWordLike ?? /\S/u.test(segment.segment)) break
  }
  return start
}

const nextWordEnd = (value, offset) => {
  const segments = wordSegments(value)
  let seenWord = false
  for (const segment of segments) {
    const end = segment.index + segment.segment.length
    if (end <= offset) continue
    const isWord = segment.isWordLike ?? /\S/u.test(segment.segment)
    if (isWord) seenWord = true
    if (seenWord && end > offset) return end
  }
  return value.length
}

const lineStart = (value, offset) => value.lastIndexOf('\n', Math.max(0, offset - 1)) + 1

const lineEnd = (value, offset) => {
  const end = value.indexOf('\n', offset)
  return end < 0 ? value.length : end
}

export const planDeleteUnit = (renderer, selection, inputType) => {
  if (!selection) return null
  if (!isCollapsed(selection)) {
    return { selection, command: editorCommands.deleteBackward() }
  }

  const point = selection.focus
  const value = textValue(renderer, point.node)
  if (value === null) return null
  let start = point.offset_utf16
  let end = point.offset_utf16

  switch (inputType) {
    case 'deleteWordBackward':
      start = previousWordStart(value, start)
      break
    case 'deleteWordForward':
      end = nextWordEnd(value, end)
      break
    case 'deleteSoftLineBackward':
    case 'deleteHardLineBackward':
      start = lineStart(value, start)
      break
    case 'deleteSoftLineForward':
    case 'deleteHardLineForward':
      end = lineEnd(value, end)
      break
    case 'deleteEntireSoftLine':
      start = lineStart(value, start)
      end = lineEnd(value, end)
      break
    default:
      return null
  }

  if (start !== end) {
    return {
      selection: range(point.node, start, end),
      command: editorCommands.deleteBackward()
    }
  }
  if (/Forward$/.test(inputType)) return planDeleteForward(renderer, selection)
  return { selection, command: editorCommands.deleteBackward() }
}

export const handleDeleteUnit = (controller, event, selection) => {
  const plan = planDeleteUnit(controller.renderer, selection, event.inputType)
  if (!plan) return false
  event.preventDefault()
  controller.schedule(async () => {
    await controller.bridge.setSelection(plan.selection)
    await controller.bridge.dispatch(plan.command)
  })
  return true
}
