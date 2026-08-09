const normalizeLineEndings = (value) => {
  return String(value || '').replace(/\r\n/g, '\n').replace(/\r/g, '\n')
}

const stripInlineMarkdown = (line) => {
  return line
    .replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '$1')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '$1')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/\*\*([^*]+)\*\*/g, '$1')
    .replace(/\*([^*]+)\*/g, '$1')
    .replace(/__([^_]+)__/g, '$1')
    .replace(/_([^_]+)_/g, '$1')
    .replace(/~~([^~]+)~~/g, '$1')
    .replace(/<[^>]+>/g, ' ')
}

const parseFrontmatter = (lines) => {
  const text = []
  const frontmatter = []
  let index = 0

  if (lines[0] !== '---') {
    return { index: 0, text }
  }

  index = 1
  while (index < lines.length && lines[index] !== '---') {
    frontmatter.push(lines[index])
    index += 1
  }

  if (lines[index] === '---') {
    index += 1
  }

  const fields = new Map()
  for (const line of frontmatter) {
    const match = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/)
    if (!match) continue
    const [, key, rawValue] = match
    const value = rawValue.trim()
    fields.set(key.toLowerCase(), value)
  }

  const title = fields.get('title')
  if (title) {
    text.push(title.replace(/^"|"$/g, ''))
  }

  const tags = fields.get('tags')
  if (tags) {
    const tagValues = tags
      .replace(/^\[|\]$/g, '')
      .split(',')
      .map((item) => item.trim().replace(/^"|"$/g, ''))
      .filter(Boolean)
    if (tagValues.length) {
      text.push(`Tags: ${tagValues.join(', ')}`)
    }
  }

  return { index, text }
}

export const markdownToSearchText = (markdown) => {
  const normalized = normalizeLineEndings(markdown).trim()
  if (!normalized) return ''

  const lines = normalized.split('\n')
  const { index: startIndex, text: frontmatterText } = parseFrontmatter(lines)
  const output = [...frontmatterText]
  let inCodeBlock = false

  for (let i = startIndex; i < lines.length; i += 1) {
    const rawLine = lines[i]
    const line = rawLine.trimEnd()

    if (/^(```|~~~)/.test(line.trim())) {
      inCodeBlock = !inCodeBlock
      continue
    }

    if (inCodeBlock) {
      if (line.trim()) {
        output.push(line.replace(/\s+/g, ' ').trim())
      }
      continue
    }

    if (!line.trim()) {
      output.push('')
      continue
    }

    const cleaned = stripInlineMarkdown(line)
      .replace(/^\s{0,3}#{1,6}\s+/, '')
      .replace(/^\s{0,3}>\s?/, '')
      .replace(/^\s*[-*+]\s+/, '')
      .replace(/^\s*\d+[.)]\s+/, '')
      .replace(/\s+/g, ' ')
      .trim()

    if (cleaned) {
      output.push(cleaned)
    }
  }

  return output
    .join('\n')
    .replace(/\n{3,}/g, '\n\n')
    .trim()
}
