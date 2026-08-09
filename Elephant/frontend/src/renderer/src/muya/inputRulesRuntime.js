export const applyLineInputRule = (line = '') => {
  const value = String(line)
  if (/^\s*#{1,6}\s+/.test(value)) return { type: 'heading', markdown: value }
  if (/^\s*>\s+/.test(value)) return { type: 'blockquote', markdown: value }
  if (/^\s*[-*+]\s+\[[ xX]\]\s+/.test(value)) return { type: 'task_list_item', markdown: value.replace(/^(\s*)[-*+]\s+\[X\]/, '$1- [x]') }
  if (/^\s*[-*+]\s+/.test(value)) return { type: 'list_item', markdown: value.replace(/^(\s*)[-*+]\s+/, '$1- ') }
  if (/^\s*\d+\.\s+/.test(value)) return { type: 'ordered_list_item', markdown: value }
  if (value.trim() === '$$') return { type: 'math_block', markdown: '$$\n\n$$' }
  if (value.trim().startsWith('```')) return { type: 'code_fence', markdown: `${value.trim()}\n\n\`\`\`` }
  return { type: 'paragraph', markdown: value }
}

export const indentLine = (line = '') => `  ${line}`
export const outdentLine = (line = '') => line.startsWith('  ') ? line.slice(2) : line

export const applyKeyboardRuleToMarkdown = (markdown = '', key = '', { shiftKey = false } = {}) => {
  const lines = String(markdown).split('\n')
  const index = Math.max(0, lines.length - 1)
  if (key === 'Tab') {
    lines[index] = shiftKey ? outdentLine(lines[index]) : indentLine(lines[index])
    return lines.join('\n')
  }
  if (key === 'Enter') {
    const current = lines[index]
    const task = current.match(/^(\s*)- \[[ xX]\] /)
    if (task) { lines.push(`${task[1]}- [ ] `); return lines.join('\n') }
    const bullet = current.match(/^(\s*)- /)
    if (bullet) { lines.push(`${bullet[1]}- `); return lines.join('\n') }
    const ordered = current.match(/^(\s*)(\d+)\. /)
    if (ordered) { lines.push(`${ordered[1]}${Number(ordered[2]) + 1}. `); return lines.join('\n') }
  }
  return markdown
}

export const handleMuyaKeydown = (runtime, event) => {
  if (!runtime || !event) return false
  if (event.key !== 'Tab' && event.key !== 'Enter') return false
  const next = applyKeyboardRuleToMarkdown(runtime.markdown, event.key, { shiftKey: event.shiftKey })
  if (next === runtime.markdown) return false
  event.preventDefault?.()
  runtime.setMarkdown(next, `key:${event.key}`)
  return true
}
