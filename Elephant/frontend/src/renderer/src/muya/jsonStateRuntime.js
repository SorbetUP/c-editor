export const markdownToJsonState = (markdown = '') => {
  const blocks = []
  const lines = String(markdown).split(/\r?\n/)
  let i = 0
  while (i < lines.length) {
    const line = lines[i]
    const trimmed = line.trim()
    if (!trimmed) { i += 1; continue }
    const heading = trimmed.match(/^(#{1,6})\s+(.*)$/)
    if (heading) { blocks.push(block('heading', { level: heading[1].length, text: heading[2] })); i += 1; continue }
    if (trimmed === '$$') {
      const body = []
      i += 1
      while (i < lines.length && lines[i].trim() !== '$$') { body.push(lines[i]); i += 1 }
      if (i < lines.length) i += 1
      blocks.push(block('math_block', { text: body.join('\n') }))
      continue
    }
    if (trimmed.startsWith('```') || trimmed.startsWith('~~~')) {
      const marker = trimmed.startsWith('```') ? '```' : '~~~'
      const info = trimmed.slice(3).trim()
      const body = []
      i += 1
      while (i < lines.length && !lines[i].trim().startsWith(marker)) { body.push(lines[i]); i += 1 }
      if (i < lines.length) i += 1
      blocks.push(block('code_fence', { marker, info, language: info.split(/\s+/)[0] || '', text: body.join('\n') }))
      continue
    }
    if (isTableStart(lines, i)) {
      const tableLines = [lines[i], lines[i + 1]]
      i += 2
      while (i < lines.length && /^\s*\|.*\|\s*$/.test(lines[i])) { tableLines.push(lines[i]); i += 1 }
      blocks.push(parseTable(tableLines))
      continue
    }
    const list = parseListLine(line)
    if (list) { blocks.push(list); i += 1; continue }
    if (trimmed.startsWith('>')) { blocks.push(block('blockquote', { text: trimmed.replace(/^>\s?/, '') })); i += 1; continue }
    blocks.push(block('paragraph', { text: line }))
    i += 1
  }
  if (!blocks.length) blocks.push(block('paragraph', { text: '' }))
  return { version: 1, type: 'muya-json-state', blocks }
}

export const jsonStateToMarkdown = (state) => (state?.blocks || []).map((item) => {
  if (item.type === 'heading') return `${'#'.repeat(item.level)} ${item.text}`
  if (item.type === 'paragraph') return item.text || ''
  if (item.type === 'blockquote') return `> ${item.text || ''}`
  if (item.type === 'task_list_item') return `${'  '.repeat(item.depth || 0)}- [${item.checked ? 'x' : ' '}] ${item.text || ''}`
  if (item.type === 'list_item') return `${'  '.repeat(item.depth || 0)}${item.ordered ? `${item.index || 1}.` : '-'} ${item.text || ''}`
  if (item.type === 'code_fence') return `${item.marker || '```'}${item.info || ''}\n${item.text || ''}\n${item.marker || '```'}`
  if (item.type === 'math_block') return `$$\n${item.text || ''}\n$$`
  if (item.type === 'table') return tableToMarkdown(item)
  return item.text || ''
}).join('\n\n')

export const jsonStateToHtml = (state) => (state?.blocks || []).map((item) => {
  if (item.type === 'heading') return `<h${item.level} data-muya-block="heading">${escapeHtml(item.text)}</h${item.level}>`
  if (item.type === 'blockquote') return `<blockquote data-muya-block="blockquote">${escapeHtml(item.text)}</blockquote>`
  if (item.type === 'code_fence') return `<pre data-muya-block="code_fence"><code class="language-${escapeAttr(item.language)}">${escapeHtml(item.text)}</code></pre>`
  if (item.type === 'math_block') return `<div data-muya-block="math_block" class="math-block katex-display">${escapeHtml(item.text)}</div>`
  if (item.type === 'table') return tableToHtml(item)
  return `<p data-muya-block="paragraph">${item.text ? escapeHtml(item.text) : '<br>'}</p>`
}).join('\n')

export const renderJsonStateIntoDom = (root, state, doc = globalThis.document) => {
  if (!root || !doc) return null
  root.setAttribute('contenteditable', 'true')
  root.setAttribute('data-muya-editor', 'true')
  root.innerHTML = jsonStateToHtml(state)
  return root
}

const block = (type, values = {}) => ({ type, ...values, children: inlineState(values.text || '') })
const inlineState = (text) => [{ type: 'text', text }]

const parseListLine = (line) => {
  const depth = Math.floor((line.match(/^\s*/)?.[0].length || 0) / 2)
  const trimmed = line.trim()
  const task = trimmed.match(/^[-*+]\s+\[([ xX])]\s+(.*)$/)
  if (task) return block('task_list_item', { depth, checked: task[1].toLowerCase() === 'x', text: task[2] })
  const ordered = trimmed.match(/^(\d+)\.\s+(.*)$/)
  if (ordered) return block('list_item', { depth, ordered: true, index: Number(ordered[1]), text: ordered[2] })
  const bullet = trimmed.match(/^[-*+]\s+(.*)$/)
  if (bullet) return block('list_item', { depth, ordered: false, text: bullet[1] })
  return null
}

const isTableStart = (lines, i) => /^\s*\|.*\|\s*$/.test(lines[i] || '') && /^\s*\|?[\s:|-]+\|\s*$/.test(lines[i + 1] || '')
const parseTable = (lines) => ({ type: 'table', headers: splitRow(lines[0]), alignments: splitRow(lines[1]).map(alignment), rows: lines.slice(2).map(splitRow) })
const splitRow = (line) => line.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map((cell) => cell.trim())
const alignment = (cell) => cell.startsWith(':') && cell.endsWith(':') ? 'center' : cell.startsWith(':') ? 'left' : cell.endsWith(':') ? 'right' : 'default'
const tableToMarkdown = (item) => [`| ${item.headers.join(' | ')} |`, `| ${item.alignments.map((a) => a === 'left' ? ':-' : a === 'center' ? ':-:' : a === 'right' ? '-:' : '-').join(' | ')} |`, ...item.rows.map((row) => `| ${row.join(' | ')} |`)].join('\n')
const tableToHtml = (item) => `<table data-muya-block="table"><thead><tr>${item.headers.map((h) => `<th>${escapeHtml(h)}</th>`).join('')}</tr></thead><tbody>${item.rows.map((row) => `<tr>${row.map((cell) => `<td>${escapeHtml(cell)}</td>`).join('')}</tr>`).join('')}</tbody></table>`
const escapeHtml = (text) => String(text).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;')
const escapeAttr = (text) => String(text).replace(/[^a-z0-9_-]/gi, '-')
