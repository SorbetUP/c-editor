export const footnotePopupState = (markdown, cursor) => {
  const before = markdown.slice(0, cursor)
  const match = before.match(/\[\^([^\]]+)\]$/)
  if (!match) return { visible: false }
  const label = match[1]
  const line = markdown.split('\n').find((item) => item.startsWith(`[^${label}]:`))
  const text = line ? line.slice(line.indexOf(':') + 1).trim() : ''
  return { visible: true, label, text, actions: ['edit', 'jump', 'delete'] }
}

export const upsertFootnote = (markdown, label, text) => {
  const lines = markdown.split('\n')
  const next = lines.map((line) => line.startsWith(`[^${label}]:`) ? `[^${label}]: ${text}` : line)
  if (next.join('\n') !== markdown) return next.join('\n')
  return `${markdown.trim()}\n\n[^${label}]: ${text}\n`
}

export const slashCommands = (query = '') => {
  const commands = [
    { id: 'heading', label: 'Heading', markdown: '# ' },
    { id: 'task-list', label: 'Task list', markdown: '- [ ] ' },
    { id: 'table', label: 'Table', markdown: '| A | B |\n| - | - |\n|   |   |' },
    { id: 'image', label: 'Image', markdown: '![alt](url)' },
    { id: 'math', label: 'Math block', markdown: '$$\n\n$$' },
    { id: 'mermaid', label: 'Mermaid diagram', markdown: '```mermaid\ngraph TD;\n```' },
    { id: 'footnote', label: 'Footnote', markdown: '[^note]\n\n[^note]: ' },
    { id: 'code', label: 'Code block', markdown: '```\n\n```' }
  ]
  const normalized = query.toLowerCase().replace(/^\//, '')
  return commands.filter((command) => command.id.includes(normalized) || command.label.toLowerCase().includes(normalized))
}

export const floatingToolbarState = (selection, rect = null) => ({
  visible: Boolean(selection && !selection.collapsed),
  rect,
  actions: ['bold', 'italic', 'strike', 'code', 'link', 'footnote']
})

export const previewBlock = (block) => {
  if (block.type === 'math_block') return { type: 'katex', html: `<span class="katex-display" data-latex="${escapeHtml(block.text || '')}">${escapeHtml(block.text || '')}</span>` }
  if (block.type === 'code_fence' && ['mermaid', 'flowchart', 'sequence', 'vega', 'vega-lite', 'plantuml'].includes(block.language)) return { type: 'diagram', language: block.language, source: block.text }
  return { type: 'none' }
}

const escapeHtml = (text) => String(text).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;')
