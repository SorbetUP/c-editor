import { createSelectionSnapshot, restoreSelectionSnapshot } from './selectionRuntime.js'
import { jsonStateToMarkdown, markdownToJsonState, renderJsonStateIntoDom } from './jsonStateRuntime.js'

export const domToMarkdown = (root) => {
  if (!root) return ''
  const nodes = [...root.childNodes]
  if (!nodes.length) return ''
  return nodes.map((node) => blockNodeToMarkdown(node)).filter((value) => value !== null && value !== undefined).join('\n\n').trim()
}

export const blockNodeToMarkdown = (node) => {
  if (!node) return ''
  if (node.nodeType === 3) return node.nodeValue || ''
  const type = node?.dataset?.muyaBlock || node?.tagName?.toLowerCase()
  const text = node?.textContent || ''
  if (type === 'heading' || /^h[1-6]$/.test(type)) {
    const level = Number(node.tagName?.slice(1)) || 1
    return `${'#'.repeat(level)} ${text.trim()}`
  }
  if (type === 'blockquote') return `> ${text.trim()}`
  if (type === 'code_fence' || type === 'pre') {
    const code = node.querySelector?.('code')
    const language = [...(code?.classList || [])].find((item) => item.startsWith('language-'))?.replace('language-', '') || ''
    return `\`\`\`${language}\n${code?.textContent || text}\n\`\`\``
  }
  if (type === 'math_block') return `$$\n${text}\n$$`
  if (type === 'table') return tableNodeToMarkdown(node)
  return text.trim()
}

export const tableNodeToMarkdown = (table) => {
  const headers = [...table.querySelectorAll('thead th')].map((item) => item.textContent.trim())
  const rows = [...table.querySelectorAll('tbody tr')].map((row) => [...row.querySelectorAll('td')].map((cell) => cell.textContent.trim()))
  if (!headers.length) return ''
  return [`| ${headers.join(' | ')} |`, `| ${headers.map(() => '-').join(' | ')} |`, ...rows.map((row) => `| ${row.join(' | ')} |`)].join('\n')
}

export const createLiveRenderScheduler = ({ root, getState, setState, getDocument = () => globalThis.document, delay = 0 } = {}) => {
  let frame = null
  const renderNow = () => {
    if (!root) return null
    const doc = getDocument()
    const selection = createSelectionSnapshot(root)
    const nextMarkdown = domToMarkdown(root)
    const nextState = markdownToJsonState(nextMarkdown)
    setState(nextState)
    renderJsonStateIntoDom(root, nextState, doc)
    restoreSelectionSnapshot(root, selection, doc)
    return { markdown: jsonStateToMarkdown(nextState), state: nextState }
  }
  const schedule = () => {
    if (frame) return frame
    const runner = () => { frame = null; renderNow() }
    frame = delay > 0 ? setTimeout(runner, delay) : requestAnimationFrameSafe(runner)
    return frame
  }
  const cancel = () => {
    if (!frame) return
    if (typeof cancelAnimationFrame === 'function') cancelAnimationFrame(frame)
    else clearTimeout(frame)
    frame = null
  }
  return { schedule, renderNow, cancel, getState }
}

const requestAnimationFrameSafe = (callback) => {
  if (typeof requestAnimationFrame === 'function') return requestAnimationFrame(callback)
  return setTimeout(callback, 0)
}
