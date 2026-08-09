export const isOutsideElement = (target, element) => {
  if (!element || !target) return true
  if (typeof element.contains !== 'function') return true
  return !element.contains(target)
}
