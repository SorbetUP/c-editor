export const NODE_ATTRIBUTE = 'data-muya-rust-id'

export const childElements = (element) =>
  Array.from(element.children).filter((child) => child.hasAttribute(NODE_ATTRIBUTE))

export const safeUrl = (value) => {
  const url = String(value || '').trim()
  if (!url) return ''
  if (/^(https?:|mailto:|tel:|#|\/|\.\/|\.\.\/)/i.test(url)) return url
  return ''
}

export const safeImageUrl = (value) => {
  const url = String(value || '').trim()
  if (!url) return ''
  if (safeUrl(url)) return url
  if (/^(data:image\/|blob:|file:|asset:)/i.test(url)) return url
  if (/^[a-z][a-z0-9+.-]*:/i.test(url)) return ''
  return url
}
