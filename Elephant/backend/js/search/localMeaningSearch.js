import fs from 'fs-extra'
import path from 'path'
import { markdownToSearchText } from './markdownToSearchText'

const ALIASES = Object.freeze({
  ai: ['agent', 'agents', 'llm', 'model', 'models', 'embedding', 'chat'],
  atomic: ['graph', 'wiki', 'canvas', 'knowledge', 'citation', 'citations', 'source', 'sources'],
  calendar: ['event', 'events', 'meeting', 'meetings', 'schedule', 'agenda'],
  image: ['photo', 'picture', 'draw', 'drawing', 'excalidraw', 'edit'],
  import: ['source', 'sources', 'rss', 'keep', 'calendar', 'migration'],
  note: ['notes', 'markdown', 'document', 'documents', 'folder', 'vault'],
  sync: ['git', 'history', 'replication', 'offline', 'merge'],
  task: ['automation', 'automations', 'programmatic', 'scheduled', 'job']
})

const tokenize = (value = '') =>
  String(value || '')
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .split(/[^a-z0-9]+/)
    .filter((token) => token.length > 1)

const expandQueryTokens = (tokens = []) => {
  const expanded = new Set(tokens)
  for (const token of tokens) {
    for (const [root, aliases] of Object.entries(ALIASES)) {
      if (token === root || aliases.includes(token)) {
        expanded.add(root)
        for (const alias of aliases) expanded.add(alias)
      }
    }
  }
  return expanded
}

const frontmatterTags = (markdown = '') => {
  const match = markdown.match(/^---\r?\n([\s\S]*?)\r?\n---/)
  if (!match) return []
  const inlineTags = match[1].match(/^\s*tags:\s*\[(.*?)\]\s*$/m)
  if (inlineTags) {
    return inlineTags[1]
      .split(',')
      .map((tag) => tag.trim().replace(/^["']|["']$/g, '').replace(/^#/, ''))
      .filter(Boolean)
  }
  return []
}

const titleFromMarkdown = (markdown = '', fallback = '') => {
  const frontmatterTitle = markdown.match(/^---\r?\n[\s\S]*?^\s*title:\s*["']?(.+?)["']?\s*$/m)
  if (frontmatterTitle?.[1]) return frontmatterTitle[1].trim()
  const heading = markdown.match(/^#\s+(.+)$/m)
  return heading?.[1]?.trim() || fallback
}

const buildSnippet = (text = '', tokens = new Set()) => {
  const normalized = text.replace(/\s+/g, ' ').trim()
  const lowered = normalized.toLowerCase()
  let index = -1
  for (const token of tokens) {
    index = lowered.indexOf(token)
    if (index !== -1) break
  }
  if (index === -1) return normalized.slice(0, 180)
  const start = Math.max(0, index - 70)
  return normalized.slice(start, index + 120)
}

export const localMeaningSearchMarkdownFiles = async({ vaultRoot, files, query, limit = 20 }) => {
  const queryTokens = tokenize(query)
  if (!queryTokens.length) return []

  const expandedTokens = expandQueryTokens(queryTokens)
  const maxResults = Math.max(1, Math.min(50, Number(limit) || 20))
  const matches = []

  for (const absolutePath of files) {
    const relativePath = path.relative(vaultRoot, absolutePath).split(path.sep).join('/')
    const markdown = await fs.readFile(absolutePath, 'utf8').catch(() => '')
    const text = markdownToSearchText(markdown)
    const title = titleFromMarkdown(markdown, path.basename(relativePath, path.extname(relativePath)))
    const tags = frontmatterTags(markdown)
    const titleTokens = new Set(tokenize(title))
    const pathTokens = new Set(tokenize(relativePath))
    const tagTokens = new Set(tags.flatMap(tokenize))
    const bodyTokens = new Set(tokenize(text))

    let score = 0
    for (const token of expandedTokens) {
      if (titleTokens.has(token)) score += 4
      if (tagTokens.has(token)) score += 3
      if (pathTokens.has(token)) score += 2
      if (bodyTokens.has(token)) score += 1
    }
    if (!score) continue

    matches.push({
      id: `meaning:${relativePath}`,
      uri: `elephantnote://vault/${encodeURI(relativePath)}`,
      title,
      relativePath,
      score,
      matchType: 'semantic',
      snippets: [{ text: buildSnippet(text, expandedTokens), score }]
    })
  }

  return matches
    .sort((a, b) => b.score - a.score || a.relativePath.localeCompare(b.relativePath))
    .slice(0, maxResults)
}
