const TEXT_NODE = 3
const ELEMENT_NODE = 1

export const normalizedText = (value) => String(value || '').replace(/\r\n?/g, '\n')
export const elementTag = (node) => node?.nodeType === ELEMENT_NODE
  ? node.tagName.toLowerCase()
  : ''
export const isWhitespaceText = (node) => node?.nodeType === TEXT_NODE &&
  !String(node.data || '').trim()

const escapeLinkDestination = (value) => String(value || '').replace(/[()\\]/g, '\\$&')
const escapeImageAlt = (value) => String(value || '').replace(/[\\\]]/g, '\\$&')

const imageMarkdown = (node) => {
  const source = node.getAttribute('src') || ''
  if (!source) return ''
  const alt = escapeImageAlt(node.getAttribute('alt') || '')
  const title = node.getAttribute('title')
  const serializedTitle = title ? ` "${String(title).replace(/"/g, '\\"')}"` : ''
  return `![${alt}](${escapeLinkDestination(source)}${serializedTitle})`
}

export const inlineChildren = (node) => Array.from(node.childNodes)
  .map(inlineMarkdown)
  .join('')

export const inlineMarkdown = (node) => {
  if (node.nodeType === TEXT_NODE) return normalizedText(node.data)
  if (node.nodeType !== ELEMENT_NODE) return ''

  const tag = elementTag(node)
  if (tag === 'img') return imageMarkdown(node)
  const content = inlineChildren(node)
  switch (tag) {
    case 'strong':
    case 'b':
      return `**${content}**`
    case 'em':
    case 'i':
      return `*${content}*`
    case 'del':
    case 's':
    case 'strike':
      return `~~${content}~~`
    case 'code': {
      const delimiter = content.includes('`') ? '``' : '`'
      return `${delimiter}${content}${delimiter}`
    }
    case 'a': {
      const href = node.getAttribute('href')
      return href ? `[${content}](${escapeLinkDestination(href)})` : content
    }
    case 'br':
      return '\n'
    default:
      return content
  }
}
