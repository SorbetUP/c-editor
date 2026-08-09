import { createSelectionSnapshot, restoreSelectionSnapshot } from './selectionRuntime.js'
import { jsonStateToHtml, markdownToJsonState } from './jsonStateRuntime.js'
import { domToMarkdown } from './liveRenderingRuntime.js'

export const findCurrentMuyaBlock = (root, doc = globalThis.document) => {
  const selection = doc?.getSelection?.() || globalThis.getSelection?.()
  if (!root || !selection || selection.rangeCount === 0) return null
  let node = selection.getRangeAt(0).startContainer
  if (node?.nodeType === 3) node = node.parentElement
  const markedBlock = node?.closest?.('[data-muya-block]')
  if (markedBlock) return markedBlock
  const browserBlock = node?.closest?.('p, h1, h2, h3, h4, h5, h6, blockquote, pre, table, li, div')
  if (browserBlock && browserBlock !== root) return browserBlock
  return null
}

export const blockToLiveMarkdown = (node) => {
  const type = node?.dataset?.muyaBlock || node?.tagName?.toLowerCase()
  const text = node?.textContent || ''
  if (type === 'heading' || /^h[1-6]$/.test(type)) {
    const level = Number(node.tagName?.slice(1)) || 1
    return `${'#'.repeat(level)} ${text.trim()}`
  }
  if (text.trim().startsWith('# ')) return text.trim()
  if (text.trim().startsWith('## ')) return text.trim()
  if (text.trim().startsWith('### ')) return text.trim()
  if (text.trim().startsWith('> ')) return text.trim()
  if (text.trim().startsWith('- [ ] ') || text.trim().startsWith('- [x] ')) return text.trim()
  if (text.trim().startsWith('- ')) return text.trim()
  return text.trim()
}

export const renderCurrentBlockNow = ({ root, setState, getDocument = () => globalThis.document } = {}) => {
  const doc = getDocument()
  const block = findCurrentMuyaBlock(root, doc)
  if (!root || !block || !doc) return null
  const selection = createSelectionSnapshot(root)
  const selectionStart = selection ? selectionOffsetWithinBlock(block, doc, 'start') : null
  const selectionEnd = selection ? selectionOffsetWithinBlock(block, doc, 'end') : null
  const blockMarkdown = blockToLiveMarkdown(block)
  const blockState = markdownToJsonState(blockMarkdown).blocks[0]
  if (!blockState) return null
  const currentType = block.dataset?.muyaBlock || tagNameToBlockType(block.tagName)
  const typeChanged = currentType !== blockState.type
  const needsBrowserBlockNormalization = !currentType && blockState.type !== 'paragraph'
  if (!typeChanged && !needsBrowserBlockNormalization) {
    const nextState = markdownToJsonState(domToMarkdown(root))
    setState(nextState)
    return { state: nextState, block }
  }
  const template = doc.createElement('template')
  template.innerHTML = jsonStateToHtml({ blocks: [blockState] })
  const replacement = template.content.firstElementChild
  block.replaceWith(replacement)
  const nextState = markdownToJsonState(domToMarkdown(root))
  setState(nextState)
  if (selection?.collapsed) {
    // A syntax promotion (paragraph -> heading/list) removes the Markdown
    // marker from the DOM. The old browser range is therefore not a safe
    // caret target; MarkText keeps typing at the end of the promoted block.
    restoreBlockSelection(replacement, replacement.textContent.length, replacement.textContent.length, doc)
  } else if (!restoreSelectionSnapshot(root, selection, doc)) {
    restoreBlockSelection(replacement, selectionStart, selectionEnd, doc)
  }
  return { state: nextState, block: replacement }
}

const tagNameToBlockType = (tagName = '') => {
  const normalized = String(tagName).toLowerCase()
  if (/^h[1-6]$/.test(normalized)) return 'heading'
  if (normalized === 'blockquote') return 'blockquote'
  if (normalized === 'pre') return 'code_fence'
  if (normalized === 'table') return 'table'
  if (normalized === 'li') return 'list_item'
  if (normalized === 'p') return 'paragraph'
  return null
}

const selectionOffsetWithinBlock = (block, doc, side) => {
  const currentSelection = doc?.getSelection?.() || globalThis.getSelection?.()
  if (!currentSelection?.rangeCount) return null
  const range = block.ownerDocument.createRange()
  range.selectNodeContents(block)
  const source = currentSelection.getRangeAt(0)
  const container = side === 'start' ? source.startContainer : source.endContainer
  const offset = side === 'start' ? source.startOffset : source.endOffset
  if (!block.contains(container)) return null
  range.setEnd(container, Math.min(offset, container.nodeType === 3 ? container.nodeValue.length : container.childNodes.length))
  return range.toString().length
}

const restoreBlockSelection = (block, start, end = start, doc = globalThis.document) => {
  if (!block || !doc?.createRange || !globalThis.getSelection || start === null || start === undefined) return false
  let textNode = block.firstChild
  if (!textNode || textNode.nodeType !== 3) {
    textNode = doc.createTextNode(block.textContent || '')
    block.replaceChildren(textNode)
  }
  const max = textNode.nodeValue.length
  const range = doc.createRange()
  range.setStart(textNode, Math.min(start, max))
  range.setEnd(textNode, Math.min(end ?? start, max))
  const currentSelection = globalThis.getSelection()
  currentSelection.removeAllRanges()
  currentSelection.addRange(range)
  return true
}
