import { NODE_ATTRIBUTE, safeImageUrl, safeUrl } from './helpers'

export const createNodeElement = (renderer, node) => {
  if (!node) throw new Error('Cannot render an absent Elephant Rust node.')
  const element = createElementForKind(renderer.ownerDocument, node.kind)
  element.setAttribute(NODE_ATTRIBUTE, String(node.id))
  element.setAttribute('data-muya-rust-layer', node.kind.layer)
  if (node.kind.value?.type) element.setAttribute('data-muya-rust-kind', node.kind.value.type)
  element.setAttribute('data-elephant-editor-node', String(node.id))
  element.setAttribute('data-elephant-editor-layer', node.kind.layer)
  if (node.kind.value?.type) element.setAttribute('data-elephant-editor-kind', node.kind.value.type)
  applyIntrinsicContent(element, node)
  renderer.elements.set(node.id, element)
  return element
}

const createElementForKind = (ownerDocument, kind) => {
  if (kind.layer === 'document') return ownerDocument.createElement('div')
  if (kind.layer === 'block') return createBlockElement(ownerDocument, kind.value || {})
  return createInlineElement(ownerDocument, kind.value || {})
}

const createBlockElement = (ownerDocument, value) => {
  switch (value.type) {
    case 'paragraph':
      return ownerDocument.createElement('p')
    case 'heading':
      return ownerDocument.createElement(`h${Math.min(6, Math.max(1, value.level || 1))}`)
    case 'block_quote':
      return ownerDocument.createElement('blockquote')
    case 'list':
      return ownerDocument.createElement(value.kind === 'ordered' ? 'ol' : 'ul')
    case 'list_item':
      return ownerDocument.createElement('li')
    case 'table':
      return ownerDocument.createElement('table')
    case 'table_row':
      return ownerDocument.createElement('tr')
    case 'table_cell':
      return ownerDocument.createElement(value.header ? 'th' : 'td')
    case 'code_block':
    case 'front_matter':
    case 'diagram':
      return ownerDocument.createElement('pre')
    case 'thematic_break':
      return ownerDocument.createElement('hr')
    default:
      return ownerDocument.createElement('div')
  }
}

const createInlineElement = (ownerDocument, value) => {
  switch (value.type) {
    case 'emphasis':
      return ownerDocument.createElement('em')
    case 'strong':
      return ownerDocument.createElement('strong')
    case 'strike':
      return ownerDocument.createElement('del')
    case 'mark_fragment':
      return createMarkFragmentElement(ownerDocument, value.mark)
    case 'code_span':
      return ownerDocument.createElement('code')
    case 'link':
    case 'auto_link':
      return ownerDocument.createElement('a')
    case 'image':
      return ownerDocument.createElement('img')
    case 'superscript':
    case 'footnote_reference':
      return ownerDocument.createElement('sup')
    case 'subscript':
      return ownerDocument.createElement('sub')
    case 'hard_break':
      return ownerDocument.createElement('br')
    default:
      return ownerDocument.createElement('span')
  }
}

const createMarkFragmentElement = (ownerDocument, mark) => {
  switch (mark) {
    case 'emphasis':
      return ownerDocument.createElement('em')
    case 'strong':
      return ownerDocument.createElement('strong')
    case 'strike':
      return ownerDocument.createElement('del')
    default:
      return ownerDocument.createElement('span')
  }
}

const applyIntrinsicContent = (element, node) => {
  const kind = node.kind.value || {}
  applyInlineContent(element, kind)
  if (node.kind.layer === 'block') applyBlockAttributes(element, kind)
}

const applyInlineContent = (element, kind) => {
  switch (kind.type) {
    case 'text':
      element.setAttribute('data-muya-rust-text', '')
      element.appendChild(element.ownerDocument.createTextNode(kind.value || ''))
      break
    case 'escaped':
      element.textContent = kind.value || ''
      break
    case 'mark_fragment':
      element.setAttribute('data-muya-rust-mark', kind.mark || '')
      element.setAttribute('data-muya-rust-mark-group', String(kind.group ?? ''))
      element.setAttribute('data-muya-rust-mark-edge', kind.edge || '')
      break
    case 'code_span':
      element.textContent = kind.code || ''
      break
    case 'image': {
      const source = safeImageUrl(kind.source)
      element.setAttribute('alt', kind.alt || '')
      element.setAttribute('data-source', kind.source || '')
      if (source) element.setAttribute('src', source)
      if (kind.title) element.setAttribute('title', kind.title)
      break
    }
    case 'link':
    case 'auto_link': {
      const destination = safeUrl(kind.destination)
      element.setAttribute('data-destination', kind.destination || '')
      if (destination) element.setAttribute('href', destination)
      if (kind.title) element.setAttribute('title', kind.title)
      break
    }
    case 'inline_html':
      element.textContent = kind.raw || ''
      break
    case 'inline_math':
      element.textContent = kind.source || ''
      break
    case 'emoji':
      element.textContent = kind.value || kind.shortcode || ''
      break
    case 'footnote_reference':
      element.textContent = kind.label || ''
      break
    case 'soft_break':
      element.textContent = '\n'
      break
    default:
      break
  }
}

const applyBlockAttributes = (element, kind) => {
  if (kind.type === 'list' && kind.kind === 'ordered' && kind.start) {
    element.setAttribute('start', String(kind.start))
  }
  if (kind.type === 'list' && kind.kind === 'task') element.setAttribute('data-task-list', '')
  if (kind.type === 'list_item' && kind.checked !== null && kind.checked !== undefined) {
    const checked = Boolean(kind.checked)
    element.setAttribute('data-checked', String(checked))
    const checkbox = element.ownerDocument.createElement('input')
    checkbox.type = 'checkbox'
    checkbox.checked = checked
    checkbox.tabIndex = -1
    checkbox.setAttribute('data-muya-rust-task-checkbox', '')
    element.appendChild(checkbox)
  }
  if (kind.type === 'table_cell') {
    element.setAttribute('data-alignment', kind.alignment || 'default')
  }
  if (kind.type === 'code_block' && kind.language) {
    element.setAttribute('data-language', kind.language)
  }
}
