import { NODE_ATTRIBUTE } from '../domRenderer/helpers'

const IMAGE_SELECTOR = `[data-muya-rust-kind="image"][${NODE_ATTRIBUTE}]`

export const handleImageClick = (controller, event) => {
  if (!controller.onImageClick) return false
  const element = event.target?.closest?.(IMAGE_SELECTOR)
  if (!element || !controller.container.contains(element)) return false
  const image = Number(element.getAttribute(NODE_ATTRIBUTE))
  if (!Number.isSafeInteger(image)) return false

  event.stopPropagation?.()
  controller.onImageClick({
    image,
    source: element.getAttribute('data-source') || '',
    alt: element.getAttribute('alt') || '',
    title: element.getAttribute('title') || '',
    rect: element.getBoundingClientRect()
  })
  return true
}
