import { editorCommands } from '../bridge'

const utf16Length = (value) => String(value || '').length

const orderedSameNodeSelection = (selection) => {
  if (selection.anchor.node !== selection.focus.node) return selection
  const start = Math.min(selection.anchor.offset_utf16, selection.focus.offset_utf16)
  const end = Math.max(selection.anchor.offset_utf16, selection.focus.offset_utf16)
  return {
    anchor: { node: selection.anchor.node, offset_utf16: start },
    focus: { node: selection.anchor.node, offset_utf16: end }
  }
}

export const startComposition = (controller) => {
  if (controller.composition) return
  const selection = controller.readSelection()
  if (!selection) return
  const range = orderedSameNodeSelection(selection)
  controller.composition = { range, text: '' }
  controller.schedule(async () => {
    await controller.bridge.setSelection(range)
    await controller.bridge.dispatch(editorCommands.beginComposition())
  })
}

export const replaceComposition = (controller, text) => {
  const composition = controller.composition
  if (!composition) return
  const range = composition.range
  const start = range.anchor
  composition.text = text
  composition.range = {
    anchor: start,
    focus: { node: start.node, offset_utf16: start.offset_utf16 + utf16Length(text) }
  }
  controller.schedule(async () => {
    await controller.bridge.setSelection(range)
    await controller.bridge.dispatch(editorCommands.updateComposition(text))
  })
}

export const finishComposition = (controller, event) => {
  if (!controller.composition) return
  const finalText = event.data || ''
  if (finalText !== controller.composition.text) replaceComposition(controller, finalText)
  controller.composition = null
  controller.schedule(() => controller.bridge.dispatch(editorCommands.commitComposition()))
}

export const cancelComposition = (controller) => {
  if (!controller.composition) return false
  controller.composition = null
  controller.schedule(() => controller.bridge.dispatch(editorCommands.cancelComposition()))
  return true
}
