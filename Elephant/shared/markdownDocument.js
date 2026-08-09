const frontmatterPattern = /^---\r?\n([\s\S]*?)\r?\n---\r?\n?/

const escapeRegExp = (value) => String(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
const generatedUntitledTitlePattern = /^Untitled(?:[ -]\d+)?$/i

const normalizeDocumentTitle = (value = '') => String(value ?? '').trim()
const isGeneratedUntitledTitle = (value = '') =>
  generatedUntitledTitlePattern.test(normalizeDocumentTitle(value))

const normalizeTag = (tag) =>
  String(tag || '')
    .trim()
    .replace(/^["']|["']$/g, '')
    .replace(/^#+/, '')
    .replace(/\s+/g, ' ')

const serializeTag = (tag) => `"${String(tag).replace(/"/g, '\\"')}"`

const splitInlineList = (value = '') => {
  const items = []
  let current = ''
  let quote = ''
  let escaped = false

  for (const char of String(value || '')) {
    if (escaped) {
      current += char
      escaped = false
      continue
    }
    if (char === '\\') {
      current += char
      escaped = true
      continue
    }
    if ((char === '"' || char === "'") && !quote) {
      quote = char
      current += char
      continue
    }
    if (char === quote) {
      quote = ''
      current += char
      continue
    }
    if (char === ',' && !quote) {
      items.push(current)
      current = ''
      continue
    }
    current += char
  }

  items.push(current)
  return items
}

const parseTagList = (value = '') => {
  const content = String(value || '').trim()
  if (!content) return []
  const list = content.startsWith('[') && content.endsWith(']')
    ? content.slice(1, -1)
    : content

  return splitInlineList(list)
    .map((item) => normalizeTag(item.replace(/^"|"$/g, '')))
    .filter(Boolean)
}

const normalizeTagInput = (nextTags = []) => {
  if (Array.isArray(nextTags)) return nextTags
  if (nextTags instanceof Set) return [...nextTags]
  if (typeof nextTags === 'string') {
    const value = nextTags.trim()
    if (!value) return []
    if (value.startsWith('[') || value.includes(',')) return parseTagList(value)
    return [value]
  }
  return []
}

export const serializeFrontmatterValue = (value) => {
  if (Array.isArray(value)) {
    return `[${value.map((item) => `"${String(item).replace(/"/g, '\\"')}"`).join(', ')}]`
  }
  return `"${String(value).replace(/"/g, '\\"')}"`
}

export const parseFrontmatter = (markdown = '') => {
  const content = String(markdown || '')
  const match = content.match(frontmatterPattern)
  if (!match) {
    return {
      raw: '',
      fields: {},
      body: content
    }
  }

  const fields = {}
  for (const line of match[1].split(/\r?\n/)) {
    const fieldMatch = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/)
    if (!fieldMatch) continue
    fields[fieldMatch[1]] = fieldMatch[2].trim().replace(/^"|"$/g, '')
  }

  return {
    raw: match[0].trimEnd(),
    fields,
    body: content.slice(match[0].length)
  }
}

export const getDocumentTitle = (markdown = '', fallback = 'Untitled') => {
  const { fields, body } = parseFrontmatter(markdown)
  const headingTitle = body.match(/^#\s+(.+)$/m)?.[1]
  const explicitTitle = normalizeDocumentTitle(fields.title || headingTitle || '')
  if (explicitTitle) return explicitTitle

  const fallbackTitle = normalizeDocumentTitle(fallback)
  return isGeneratedUntitledTitle(fallbackTitle) ? '' : fallbackTitle
}

export const stripDisplayedTitle = (markdown = '', title = '') => {
  const escapedTitle = escapeRegExp(title || '')
  if (!escapedTitle) return String(markdown || '').replace(/^\s+/, '')
  return String(markdown || '')
    .replace(new RegExp(`^\\s*#\\s+${escapedTitle}\\s*\\n+`, 'i'), '')
    .replace(/^\s+/, '')
}

export const toEditorMarkdown = (markdown = '', fallbackTitle = 'Untitled') => {
  const title = getDocumentTitle(markdown, fallbackTitle)
  const { body } = parseFrontmatter(markdown)
  return stripDisplayedTitle(body, title)
}

export const getEditorMarkdownStats = (markdown = '') => {
  const content = String(markdown || '').trim()
  const words = content.length === 0 ? [] : content.split(/\s+/).filter(Boolean)
  return {
    word: words.length,
    character: Array.from(String(markdown || '')).length
  }
}

const composeNoteDocument = (rawFrontmatter, title, body = '') => {
  const normalizedBody = stripDisplayedTitle(body, title).trim()
  if (!normalizedBody) return rawFrontmatter
  return [rawFrontmatter, title ? `# ${title}` : '', normalizedBody]
    .filter(Boolean)
    .join('\n\n')
    .trimEnd()
}

export const ensureNoteDocument = (markdown = '', title = 'Untitled') => {
  const content = String(markdown || '')
  const normalizedTitle = normalizeDocumentTitle(title)
  if (!normalizedTitle && !content.startsWith('---\n')) return content
  if (content.startsWith('---\n')) {
    return content.replace(/^type:\s*.*$/m, `type: ${serializeFrontmatterValue('note')}`)
  }

  const now = new Date().toISOString()
  const rawFrontmatter = [
    '---',
    `title: ${serializeFrontmatterValue(normalizedTitle)}`,
    `type: ${serializeFrontmatterValue('note')}`,
    'tags: []',
    `createdAt: ${serializeFrontmatterValue(now)}`,
    `updatedAt: ${serializeFrontmatterValue(now)}`,
    '---'
  ].join('\n')

  return composeNoteDocument(rawFrontmatter, normalizedTitle, content)
}

export const mergeEditorMarkdown = (currentDocument = '', editorMarkdown = '', fallbackTitle = 'Untitled') => {
  const title = getDocumentTitle(currentDocument, fallbackTitle)
  const base = ensureNoteDocument(currentDocument, title)
  const { raw } = parseFrontmatter(base)
  return composeNoteDocument(raw, title, editorMarkdown)
}

export const renameDocumentTitle = (markdown = '', nextTitle = 'Untitled') => {
  const title = String(nextTitle || '').trim() || 'Untitled'
  const base = ensureNoteDocument(markdown, title)
  const { raw, body } = parseFrontmatter(base)
  const nextFrontmatter = raw.replace(/^title:\s*.*$/m, `title: ${serializeFrontmatterValue(title)}`)
  const nextBody = stripDisplayedTitle(body, getDocumentTitle(markdown, title))
  return composeNoteDocument(nextFrontmatter, title, nextBody)
}

export const getDocumentCreatedAt = (markdown = '') => parseFrontmatter(markdown).fields.createdAt || ''

export const parseMarkdownTags = (markdown = '') => {
  const frontmatterMatch = String(markdown || '').match(frontmatterPattern)
  if (!frontmatterMatch) return []

  const lines = frontmatterMatch[1].split(/\r?\n/)
  const tagsIndex = lines.findIndex((line) => /^\s*tags:\s*/.test(line))
  if (tagsIndex < 0) return []

  const inlineValue = lines[tagsIndex].replace(/^\s*tags:\s*/, '')
  if (inlineValue.trim()) return parseTagList(inlineValue)

  const blockTags = []
  for (let index = tagsIndex + 1; index < lines.length; index += 1) {
    const line = lines[index]
    if (/^\s*[A-Za-z0-9_-]+:\s*/.test(line)) break
    const match = line.match(/^\s*-\s*(.+?)\s*$/)
    if (!match) continue
    const tag = normalizeTag(match[1])
    if (tag) blockTags.push(tag)
  }
  return blockTags
}

export const updateMarkdownTags = (markdown = '', nextTags = [], title = 'Untitled') => {
  const normalizedTitle = normalizeDocumentTitle(title)
  const tagsInput = normalizeTagInput(nextTags)
  const uniqueTags = [...new Set(tagsInput.map(normalizeTag).filter(Boolean))]
  const tagsLine = `tags: [${uniqueTags.map(serializeTag).join(', ')}]`
  const content = String(markdown || '')
  const frontmatterMatch = content.match(frontmatterPattern)

  if (!frontmatterMatch) {
    const body = content.replace(/^\s+/, '')
    const frontmatter = ['---']
    if (normalizedTitle) {
      frontmatter.push(`title: "${normalizedTitle.replace(/"/g, '\\"')}"`)
    }
    frontmatter.push('type: "note"', tagsLine, '---')

    const visibleBody = []
    if (normalizedTitle) visibleBody.push(`# ${normalizedTitle}`)
    if (body) visibleBody.push(body)

    return [...frontmatter, '', ...visibleBody].join('\n').trimEnd()
  }

  const frontmatterBody = frontmatterMatch[1]
  const lines = frontmatterBody.split(/\r?\n/)
  const tagsIndex = lines.findIndex((line) => /^\s*tags:\s*/.test(line))

  if (tagsIndex >= 0) {
    const hadBlockTags = !lines[tagsIndex].replace(/^\s*tags:\s*/, '').trim()
    lines[tagsIndex] = tagsLine
    if (hadBlockTags) {
      let deleteCount = 0
      for (let index = tagsIndex + 1; index < lines.length; index += 1) {
        if (/^\s*[A-Za-z0-9_-]+:\s*/.test(lines[index])) break
        if (!/^\s*-\s*/.test(lines[index]) && lines[index].trim()) break
        deleteCount += 1
      }
      if (deleteCount) lines.splice(tagsIndex + 1, deleteCount)
    }
  } else {
    const insertAfter = lines.findIndex((line) => /^(title|type|createdAt|updatedAt):\s*/.test(line))
    if (insertAfter >= 0) {
      lines.splice(insertAfter + 1, 0, tagsLine)
    } else {
      lines.unshift(tagsLine)
    }
  }

  return `---\n${lines.join('\n')}\n---\n${content.slice(frontmatterMatch[0].length).replace(/^\n+/, '')}`.trimEnd()
}

export const renameMarkdownTag = (markdown = '', fromTag = '', toTag = '', title = 'Untitled') => {
  const currentTags = parseMarkdownTags(markdown)
  const source = normalizeTag(fromTag)
  const target = normalizeTag(toTag)
  if (!source || !target) return markdown
  const nextTags = currentTags.map((tag) => (tag === source ? target : tag))
  return updateMarkdownTags(markdown, nextTags, title)
}

export const deleteMarkdownTag = (markdown = '', tag = '', title = 'Untitled') => {
  const currentTags = parseMarkdownTags(markdown)
  const target = normalizeTag(tag)
  if (!target) return markdown
  return updateMarkdownTags(
    markdown,
    currentTags.filter((currentTag) => currentTag !== target),
    title
  )
}
