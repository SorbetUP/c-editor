import { editorCommands } from '../bridge'
import { NODE_ATTRIBUTE } from '../domRenderer/helpers'

const TASK_SELECTOR = '[data-muya-rust-task-checkbox]'
const ITEM_SELECTOR = `[data-muya-rust-kind="list_item"][${NODE_ATTRIBUTE}]`

export const handleTaskClick = (controller, event) => {
  const checkbox = event.target?.closest?.(TASK_SELECTOR)
  if (!checkbox || !controller.container.contains(checkbox)) return false
  const item = checkbox.closest(ITEM_SELECTOR)
  const itemId = Number(item?.getAttribute(NODE_ATTRIBUTE))
  if (!Number.isSafeInteger(itemId)) return false

  event.stopPropagation?.()
  controller.schedule(() =>
    controller.bridge.dispatch(
      editorCommands.setTaskChecked(itemId, checkbox.checked, controller.autoCheck)
    )
  )
  return true
}
