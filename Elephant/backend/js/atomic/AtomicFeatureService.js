import fs from 'fs-extra'
import path from 'path'
import { execFile } from 'child_process'
import { promisify } from 'util'

import { getSearchService } from '../search/searchIpc'
import { markdownToSearchText } from '../search/markdownToSearchText'
import { isIgnoredPath, isMarkdownFile } from '../search/pathSafety'
import { ATOMIC_MODEL_CATALOG, MODEL_GROUPS } from 'common/elephantnote/atomicWorkspace'

const execFileAsync = promisify(execFile)

const DEFAULT_GRAPH_OPTIONS = Object.freeze({
  semanticThreshold: 0.28,
  lexicalThreshold: 0.24,
  maxNotes: 5000,
  maxLinksPerNote: 8,
  maxEdges: 12000,
  maxInvertedBucketSize: 120
})

const PROVIDERS = Object.freeze([
  { id: 'node-llama-cpp', name: 'node-llama-cpp', type: 'local-main-process', defaultEndpoint: 'node-llama-cpp://local', supportsModelPull: true, notes: 'Default local text runtime. Downloads GGUF files and runs chat plus embeddings in the Electron main process.' },
  { id: 'local-ocr', name: 'Local OCR', type: 'local-main-process', defaultEndpoint: 'tesseract://local', supportsModelPull: false, notes: 'Default local image-to-text runtime backed by Tesseract.' },
  { id: 'lm-studio', name: 'LM Studio', type: 'local', defaultEndpoint: 'http://127.0.0.1:1234/v1/chat/completions', supportsModelPull: false, notes: 'OpenAI-compatible local server from LM Studio.' },
  { id: 'openai-compatible', name: 'OpenAI-compatible API', type: 'remote-or-local', defaultEndpoint: 'https://api.openai.com/v1/chat/completions', supportsModelPull: false, notes: 'OpenRouter, OpenAI-compatible gateways, self-hosted vLLM, llama.cpp server, etc.' },
  { id: 'codex-compatible', name: 'Codex-compatible bridge', type: 'remote', defaultEndpoint: 'http://127.0.0.1:1455/v1/chat/completions', supportsModelPull: false, notes: 'Use an OpenAI-compatible bridge or endpoint.' }
])

const RECOMMENDED_MODELS = ATOMIC_MODEL_CATALOG

const normalizeSlashPath = (value = '') => String(value || '').split(path.sep).join('/')
const normalizeVaultRoot = (vaultRoot = '') => path.resolve(String(vaultRoot || ''))

const assertVaultRoot = (vaultRoot) => {
  const root = normalizeVaultRoot(vaultRoot)
  if (!root || root === path.parse(root).root) throw new Error('A valid ElephantNote vault root is required.')
  return root
}

const toRelativePath = (vaultRoot, absolutePath) => normalizeSlashPath(path.relative(vaultRoot, absolutePath))

const assertRelativeNotePath = (vaultRoot, relativePath) => {
  const root = assertVaultRoot(vaultRoot)
  const normalized = normalizeSlashPath(relativePath)
  if (!normalized || normalized.startsWith('../') || path.isAbsolute(normalized)) throw new Error('Invalid note path.')
  const fullPath = path.resolve(root, normalized)
  if (!fullPath.startsWith(root + path.sep)) throw new Error('Note path is outside the active vault.')
  if (!isMarkdownFile(fullPath)) throw new Error('Only markdown notes can be processed.')
  return { root, normalized, fullPath }
}

const getVaultModelDir = async(vaultRoot) => {
  const root = assertVaultRoot(vaultRoot)
  const modelDir = path.join(root, '.elephantnote', 'models')
  await fs.ensureDir(modelDir)
  return modelDir
}

const slugify = (value = '') => String(value || 'topic')
  .normalize('NFKD')
  .replace(/[\u0300-\u036f]/g, '')
  .toLowerCase()
  .replace(/[^a-z0-9]+/g, '-')
  .replace(/^-+|-+$/g, '') || 'topic'

const safeFilename = (value = 'Untitled') => String(value || 'Untitled')
  .replace(/[\\/]/g, '-')
  .replace(/[<>:"|?*]/g, '')
  .replace(/\s+/g, ' ')
  .trim()
  .slice(0, 90) || 'Untitled'

const nextAvailableName = async(directory, baseName, currentPath = '') => {
  const ext = path.extname(baseName)
  const stem = path.basename(baseName, ext)
  let candidate = baseName
  let counter = 2
  while (await fs.pathExists(path.join(directory, candidate))) {
    if (currentPath && path.resolve(directory, candidate) === path.resolve(currentPath)) return candidate
    candidate = `${stem} ${counter}${ext}`
    counter += 1
  }
  return candidate
}

const parseFrontmatter = (markdown = '') => {
  const match = String(markdown || '').match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n?/)
  if (!match) return {}
  const meta = {}
  for (const line of match[1].split(/\r?\n/)) {
    const parts = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/)
    if (!parts) continue
    const key = parts[1]
    let value = parts[2].trim()
    if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) value = value.slice(1, -1)
    if (value.startsWith('[') && value.endsWith(']')) {
      meta[key] = value.slice(1, -1).split(',').map((item) => item.trim().replace(/^['"]|['"]$/g, '')).filter(Boolean)
    } else {
      meta[key] = value
    }
  }
  return meta
}

const stripFrontmatter = (markdown = '') => String(markdown || '').replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/, '')

const extractTitle = (markdown, relativePath, meta = {}) => {
  if (meta.title) return String(meta.title)
  const heading = stripFrontmatter(markdown).match(/^#\s+(.+)$/m)
  if (heading) return heading[1].trim()
  return path.basename(relativePath, path.extname(relativePath))
}

const extractTags = (meta = {}, markdown = '') => {
  const tags = new Set()
  const frontmatterTags = Array.isArray(meta.tags) ? meta.tags : []
  for (const tag of frontmatterTags) {
    const normalized = String(tag || '').trim().replace(/^#/, '')
    if (normalized) tags.add(normalized)
  }
  for (const match of String(markdown || '').matchAll(/(?:^|\s)#([\p{L}\p{N}_/-]{2,64})/gu)) tags.add(match[1])
  return [...tags].slice(0, 24)
}

const extractHeadings = (markdown = '') => {
  const headings = []
  const lines = String(markdown || '').split(/\r?\n/)
  for (let index = 0; index < lines.length; index += 1) {
    const match = lines[index].match(/^(#{1,6})\s+(.+)$/)
    if (!match) continue
    headings.push({ level: match[1].length, title: match[2].replace(/[#`*_]/g, '').trim(), line: index + 1 })
  }
  return headings
}

const extractLinks = (markdown = '') => {
  const links = []
  for (const match of String(markdown || '').matchAll(/\[\[([^\]]+)\]\]/g)) links.push({ label: match[1].trim(), type: 'wikilink' })
  for (const match of String(markdown || '').matchAll(/\[[^\]]+\]\(([^)]+\.md(?:#[^)]+)?)\)/gi)) links.push({ label: decodeURIComponent(match[1]).replace(/#.*/, ''), type: 'markdown' })
  return links
}

const STOP_WORDS = new Set(['avec', 'dans', 'pour', 'plus', 'moins', 'comme', 'donc', 'mais', 'cette', 'cela', 'fait', 'faire', 'etre', 'être', 'the', 'and', 'that', 'this', 'with', 'from', 'have', 'will', 'your', 'note', 'notes', 'todo', 'done', 'untitled', 'sans', 'titre'])

const tokenize = (text = '') => String(text || '')
  .toLowerCase()
  .normalize('NFKD')
  .replace(/[\u0300-\u036f]/g, '')
  .replace(/[^\p{L}\p{N}_-]+/gu, ' ')
  .split(/\s+/)
  .map((word) => word.trim())
  .filter((word) => word.length >= 3 && !STOP_WORDS.has(word))

const topTerms = (text = '', limit = 12) => {
  const counts = new Map()
  for (const token of tokenize(text)) counts.set(token, (counts.get(token) || 0) + 1)
  return [...counts.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0])).slice(0, limit).map(([term]) => term)
}

const jaccard = (left = [], right = []) => {
  const a = new Set(left)
  const b = new Set(right)
  if (!a.size || !b.size) return 0
  let intersection = 0
  for (const value of a) if (b.has(value)) intersection += 1
  return intersection / (a.size + b.size - intersection)
}

const summarizeLocally = (text = '', maxLength = 340) => {
  const cleaned = String(text || '').replace(/\s+/g, ' ').trim()
  if (!cleaned) return 'No readable content yet.'
  const sentences = cleaned.split(/(?<=[.!?])\s+/).filter(Boolean)
  const summary = sentences.slice(0, 3).join(' ') || cleaned
  return summary.length > maxLength ? `${summary.slice(0, maxLength - 1).trim()}…` : summary
}

const extractActionItems = (markdown = '') => String(markdown || '').split(/\r?\n/).map((line, index) => ({ line: index + 1, text: line.trim() })).filter((item) => /^[-*]\s+\[[ xX]\]/.test(item.text) || /\b(todo|à faire|a faire|fixme|next|prochaine étape)\b/i.test(item.text)).slice(0, 12)

const resolveRelativeReference = (reference = '', knownPaths = new Set()) => {
  const normalized = normalizeSlashPath(reference).replace(/^\.\//, '')
  if (!normalized) return ''
  const direct = normalized.endsWith('.md') ? normalized : `${normalized}.md`
  if (knownPaths.has(direct)) return direct
  const basename = path.basename(direct).toLowerCase()
  for (const candidate of knownPaths) if (path.basename(candidate).toLowerCase() === basename) return candidate
  return ''
}

const edgeKey = (source, target) => [source, target].sort().join('::')

const addScoredEdge = (edges, edge) => {
  if (!edge?.source || !edge?.target || edge.source === edge.target) return
  const key = edgeKey(edge.source, edge.target)
  const current = edges.get(key)
  const normalized = {
    id: key,
    source: edge.source,
    target: edge.target,
    type: edge.type || 'related',
    reason: edge.reason || edge.type || 'related',
    weight: Math.max(0, Math.min(1, Number(edge.weight || 0.1)))
  }
  if (!current || normalized.weight > current.weight) edges.set(key, normalized)
}

const addLimitedCliqueEdges = ({ notes, getter, type, reasonPrefix, baseWeight, edges, maxBucketSize, maxLinksPerNote }) => {
  const buckets = new Map()
  for (const note of notes) {
    for (const value of getter(note) || []) {
      if (!value) continue
      if (!buckets.has(value)) buckets.set(value, [])
      buckets.get(value).push(note)
    }
  }
  for (const [value, bucket] of buckets.entries()) {
    if (bucket.length < 2 || bucket.length > maxBucketSize) continue
    const sorted = bucket.slice().sort((a, b) => String(b.updatedAt || '').localeCompare(String(a.updatedAt || '')))
    for (let index = 0; index < sorted.length; index += 1) {
      const source = sorted[index]
      const neighbors = sorted.filter((note) => note.path !== source.path).slice(0, maxLinksPerNote)
      for (const target of neighbors) addScoredEdge(edges, { source: source.path, target: target.path, type, reason: `${reasonPrefix}${value}`, weight: baseWeight })
    }
  }
}

const normalizeProviderConfig = (config = {}) => ({
  enabled: Boolean(config.enabled),
  provider: String(config.provider || config.preset || 'node-llama-cpp').trim(),
  endpoint: String(config.endpoint || '').trim(),
  model: String(config.model || '').trim(),
  apiKey: String(config.apiKey || '').trim(),
  temperature: Number.isFinite(Number(config.temperature)) ? Number(config.temperature) : 0.2
})

const getProviderDefaultEndpoint = (providerId) => PROVIDERS.find((provider) => provider.id === providerId)?.defaultEndpoint || ''

const callChatProvider = async(messages = [], rawConfig = {}) => {
  const config = normalizeProviderConfig(rawConfig)
  if (!config.enabled || !config.model) return ''
  if (config.provider === 'ollama') {
    const base = (config.endpoint || getProviderDefaultEndpoint('ollama')).replace(/\/+$/, '')
    const url = base.endsWith('/api/chat') ? base : `${base}/api/chat`
    const response = await fetch(url, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ model: config.model, messages, stream: false, options: { temperature: config.temperature } }) })
    const data = await response.json().catch(() => ({}))
    if (!response.ok) throw new Error(data?.error || `Local chat endpoint returned HTTP ${response.status}.`)
    return data?.message?.content || data?.response || ''
  }
  const endpoint = config.endpoint || getProviderDefaultEndpoint(config.provider) || getProviderDefaultEndpoint('openai-compatible')
  const response = await fetch(endpoint, { method: 'POST', headers: { 'content-type': 'application/json', ...(config.apiKey ? { authorization: `Bearer ${config.apiKey}` } : {}) }, body: JSON.stringify({ model: config.model, messages, temperature: config.temperature, stream: false }) })
  const data = await response.json().catch(() => ({}))
  if (!response.ok) throw new Error(data?.error?.message || data?.message || `AI endpoint returned HTTP ${response.status}.`)
  return data?.choices?.[0]?.message?.content || data?.choices?.[0]?.text || data?.message?.content || ''
}

const parseMarkdownNote = async(vaultRoot, absolutePath) => {
  const markdown = await fs.readFile(absolutePath, 'utf8')
  const relativePath = toRelativePath(vaultRoot, absolutePath)
  const stats = await fs.stat(absolutePath).catch(() => null)
  const meta = parseFrontmatter(markdown)
  const searchable = markdownToSearchText(markdown)
  const title = extractTitle(markdown, relativePath, meta)
  const tags = extractTags(meta, markdown)
  const folder = path.dirname(relativePath) === '.' ? '' : path.dirname(relativePath)
  const terms = topTerms(`${title} ${tags.join(' ')} ${searchable}`, 16)
  return { id: relativePath, path: relativePath, relativePath, absolutePath, title, folder, tags, headings: extractHeadings(markdown), links: extractLinks(markdown), summary: summarizeLocally(searchable), keyTerms: terms, actionItems: extractActionItems(markdown), text: searchable, markdown, updatedAt: stats?.mtime?.toISOString?.() || meta.updatedAt || '', createdAt: meta.createdAt || '' }
}

const createCitation = (note, index = 0) => ({ id: note.path, index: index + 1, title: note.title, path: note.path, snippet: note.summary, tags: note.tags || [] })

const isWeakTitle = (note) => {
  const stem = path.basename(note.path, path.extname(note.path)).toLowerCase()
  return /^(untitled|sans titre|new note|nouvelle note|note|draft|brouillon)([-_\s]?\d+)?$/.test(stem) || /^\d{4}-\d{2}-\d{2}/.test(stem)
}

const inferNoteTitle = (note) => {
  const heading = note.headings.find((item) => item.level === 1)?.title || note.headings[0]?.title
  if (heading && heading.length >= 3) return safeFilename(heading)
  const terms = note.keyTerms.slice(0, 5).join(' ')
  if (terms) return safeFilename(terms.replace(/\b\w/g, (char) => char.toUpperCase()))
  const firstSentence = String(note.summary || '').split(/[.!?]/)[0]
  return safeFilename(firstSentence || 'Untitled note')
}

export class AtomicFeatureService {
  constructor({ executor = execFileAsync } = {}) {
    this.executor = executor
  }

  describeApi() {
    return {
      version: '2026-05-31',
      namespace: 'atomic.features',
      database: {
        kind: 'vault-local-files',
        root: '<vault>/.elephantnote',
        readableByAgents: true,
        writableByAgents: 'through approved API actions only'
      },
      actions: [
        { name: 'providers', description: 'List supported AI providers and recommended local models.' },
        { name: 'overview', description: 'Inspect local Atomic/AI capabilities for the active vault.' },
        { name: 'graph', description: 'Build the scalable Note Graph.' },
        { name: 'wiki', description: 'Generate cited wiki proposals.' },
        { name: 'wiki.createPage', description: 'Create a markdown wiki page from a proposal.' },
        { name: 'summarize', description: 'Summarize one note with local fallback.' },
        { name: 'structure', description: 'Suggest headings, tags and actions for one note.' },
        { name: 'notes.autoName', description: 'Infer and optionally apply a better filename for a note.' },
        { name: 'models.listLocal', description: 'List installed node-llama-cpp models and vault model directory metadata.' },
        { name: 'models.pull', description: 'Prepare one recommended node-llama-cpp model and mirror metadata into the vault model directory.' }
      ]
    }
  }

  async callApi({ action, arguments: args = {} } = {}) {
    if (action === 'providers') return this.providers()
    if (action === 'overview') return this.overview(args)
    if (action === 'graph') return this.graph(args)
    if (action === 'wiki') return this.wiki(args)
    if (action === 'wiki.createPage') return this.createWikiPage(args)
    if (action === 'summarize') return this.summarize(args)
    if (action === 'structure') return this.structure(args)
    if (action === 'notes.autoName') return this.autoNameNote(args)
    if (action === 'models.listLocal') return this.listLocalModels(args)
    if (action === 'models.pull') return this.pullModel(args)
    throw new Error(`Unknown Atomic feature action: ${action}`)
  }

  providers() {
    return { providers: PROVIDERS, recommendedModels: RECOMMENDED_MODELS, modelGroups: MODEL_GROUPS, graphDefaults: DEFAULT_GRAPH_OPTIONS }
  }

  async overview({ vaultRoot, windowId = null } = {}) {
    const root = assertVaultRoot(vaultRoot)
    const notes = await this.listNotes(root)
    const status = await getSearchService().getStatus(windowId).catch(() => null)
    return { vaultRoot: root, modelDir: await getVaultModelDir(root), notes: notes.length, providers: PROVIDERS, recommendedModels: RECOMMENDED_MODELS, modelGroups: MODEL_GROUPS, api: this.describeApi(), capabilities: ['semantic-search', 'hybrid-search', 'cited-rag', 'automatic-wiki', 'automatic-summaries', 'automatic-structure', 'automatic-note-naming', 'note-graph', 'vault-model-directory', 'extension-api', 'agent-database-access'], searchStatus: status }
  }

  async listNotes(vaultRoot, { maxNotes = DEFAULT_GRAPH_OPTIONS.maxNotes } = {}) {
    const root = assertVaultRoot(vaultRoot)
    const files = []
    const walk = async(directory) => {
      const entries = await fs.readdir(directory, { withFileTypes: true })
      for (const entry of entries) {
        const absolutePath = path.join(directory, entry.name)
        const relativePath = toRelativePath(root, absolutePath)
        if (isIgnoredPath(relativePath)) continue
        if (entry.isDirectory()) await walk(absolutePath)
        else if (entry.isFile() && isMarkdownFile(absolutePath)) files.push(absolutePath)
      }
    }
    await walk(root)
    files.sort()
    const selected = maxNotes > 0 ? files.slice(0, maxNotes) : files
    const notes = []
    for (const file of selected) {
      try { notes.push(await parseMarkdownNote(root, file)) } catch { /* keep vault usable */ }
    }
    return notes.sort((a, b) => a.path.localeCompare(b.path))
  }

  async graph({ vaultRoot, windowId = null, semanticThreshold = DEFAULT_GRAPH_OPTIONS.semanticThreshold, lexicalThreshold = DEFAULT_GRAPH_OPTIONS.lexicalThreshold, maxNotes = DEFAULT_GRAPH_OPTIONS.maxNotes, maxLinksPerNote = DEFAULT_GRAPH_OPTIONS.maxLinksPerNote, maxEdges = DEFAULT_GRAPH_OPTIONS.maxEdges } = {}) {
    const root = assertVaultRoot(vaultRoot)
    const notes = await this.listNotes(root, { maxNotes })
    const byPath = new Map(notes.map((note) => [note.path, note]))
    const knownPaths = new Set(byPath.keys())
    const rawEdges = new Map()

    addLimitedCliqueEdges({ notes, getter: (note) => note.tags, type: 'tag', reasonPrefix: '#', baseWeight: 0.58, edges: rawEdges, maxBucketSize: DEFAULT_GRAPH_OPTIONS.maxInvertedBucketSize, maxLinksPerNote })
    addLimitedCliqueEdges({ notes, getter: (note) => note.folder ? [note.folder] : [], type: 'folder', reasonPrefix: '', baseWeight: 0.34, edges: rawEdges, maxBucketSize: DEFAULT_GRAPH_OPTIONS.maxInvertedBucketSize, maxLinksPerNote })
    addLimitedCliqueEdges({ notes, getter: (note) => note.keyTerms.slice(0, 8), type: 'lexical', reasonPrefix: 'term:', baseWeight: 0.28, edges: rawEdges, maxBucketSize: 60, maxLinksPerNote: Math.max(2, Math.floor(maxLinksPerNote / 2)) })

    for (let leftIndex = 0; leftIndex < notes.length; leftIndex += 1) {
      const left = notes[leftIndex]
      const candidates = new Set()
      for (const edge of rawEdges.values()) {
        if (edge.source === left.path) candidates.add(edge.target)
        if (edge.target === left.path) candidates.add(edge.source)
      }
      for (const targetPath of candidates) {
        const right = byPath.get(targetPath)
        if (!right) continue
        const lexicalScore = jaccard(left.keyTerms, right.keyTerms)
        if (lexicalScore >= lexicalThreshold) addScoredEdge(rawEdges, { source: left.path, target: right.path, type: 'lexical', reason: 'shared meaning keywords', weight: Math.min(0.88, lexicalScore + 0.2) })
      }
    }

    for (const note of notes) {
      for (const link of note.links) {
        const target = resolveRelativeReference(link.label, knownPaths)
        if (target && target !== note.path) addScoredEdge(rawEdges, { source: note.path, target, type: 'explicit-link', reason: link.type, weight: 1 })
      }
    }

    const inspection = await getSearchService().inspectIndex(windowId).catch(() => null)
    for (const semanticLink of inspection?.semanticLinks || []) {
      if (!byPath.has(semanticLink.source) || !byPath.has(semanticLink.target)) continue
      if (Number(semanticLink.score || 0) < semanticThreshold) continue
      addScoredEdge(rawEdges, { source: semanticLink.source, target: semanticLink.target, type: 'semantic', reason: 'embedding similarity', weight: Math.max(0.1, Math.min(1, Number(semanticLink.score || 0))) })
    }

    const byNode = new Map()
    for (const edge of rawEdges.values()) {
      if (!byNode.has(edge.source)) byNode.set(edge.source, [])
      if (!byNode.has(edge.target)) byNode.set(edge.target, [])
      byNode.get(edge.source).push(edge)
      byNode.get(edge.target).push(edge)
    }
    const limitedEdges = new Map()
    for (const list of byNode.values()) {
      for (const edge of list.sort((a, b) => b.weight - a.weight).slice(0, maxLinksPerNote)) limitedEdges.set(edge.id, edge)
    }
    const edges = [...limitedEdges.values()].sort((a, b) => b.weight - a.weight).slice(0, maxEdges)
    const clusters = this.buildClusters(notes)

    return {
      generatedAt: new Date().toISOString(),
      vaultRoot: root,
      nodes: notes.map((note) => ({ id: note.path, path: note.path, title: note.title, folder: note.folder, tags: note.tags, summary: note.summary, headings: note.headings.slice(0, 8), keyTerms: note.keyTerms.slice(0, 10), updatedAt: note.updatedAt, cluster: note.tags[0] || note.folder || 'untagged', weakTitle: isWeakTitle(note) })),
      edges,
      clusters,
      stats: { totalNotes: notes.length, totalCandidateEdges: rawEdges.size, renderedEdges: edges.length, maxLinksPerNote, maxEdges },
      indexPath: inspection?.indexPath || '',
      searchStatus: inspection?.status || null
    }
  }

  buildClusters(notes = []) {
    const clusters = new Map()
    for (const note of notes) {
      const clusterId = note.tags[0] || note.folder || 'untagged'
      const current = clusters.get(clusterId) || { id: clusterId, label: clusterId, noteCount: 0, paths: [], keyTerms: new Map() }
      current.noteCount += 1
      current.paths.push(note.path)
      for (const term of note.keyTerms.slice(0, 8)) current.keyTerms.set(term, (current.keyTerms.get(term) || 0) + 1)
      clusters.set(clusterId, current)
    }
    return [...clusters.values()].map((cluster) => ({ id: cluster.id, label: cluster.label, noteCount: cluster.noteCount, paths: cluster.paths, keyTerms: [...cluster.keyTerms.entries()].sort((a, b) => b[1] - a[1]).slice(0, 8).map(([term]) => term) })).sort((a, b) => b.noteCount - a.noteCount || a.label.localeCompare(b.label))
  }

  async wiki({ vaultRoot, windowId = null, providerConfig = {} } = {}) {
    const graph = await this.graph({ vaultRoot, windowId })
    const byPath = new Map(graph.nodes.map((node) => [node.path, node]))
    const groups = new Map()
    const addToGroup = (topic, node) => {
      if (!topic || !node) return
      const current = groups.get(topic) || []
      current.push(node)
      groups.set(topic, current)
    }
    for (const node of graph.nodes) {
      for (const tag of node.tags || []) addToGroup(`#${tag}`, node)
      if (node.folder) addToGroup(node.folder, node)
      if (!(node.tags || []).length && !node.folder) addToGroup('Untagged knowledge', node)
    }
    for (const cluster of graph.clusters) for (const pathname of cluster.paths || []) addToGroup(cluster.label, byPath.get(pathname))
    const records = []
    const seen = new Set()
    for (const [topic, nodes] of groups.entries()) {
      const uniqueNodes = [...new Map(nodes.map((node) => [node.path, node])).values()]
      if (uniqueNodes.length < 2 && graph.nodes.length > 2) continue
      const id = `atomic-wiki-${slugify(topic)}`
      if (seen.has(id)) continue
      seen.add(id)
      const citations = uniqueNodes.sort((a, b) => String(b.updatedAt || '').localeCompare(String(a.updatedAt || ''))).slice(0, 8).map(createCitation)
      const keyTerms = topTerms(citations.map((citation) => `${citation.title} ${citation.snippet} ${(citation.tags || []).join(' ')}`).join('\n'), 12)
      let summary = `This topic connects ${citations.length} local note${citations.length === 1 ? '' : 's'} around ${keyTerms.slice(0, 5).join(', ') || topic}.`
      const localContext = citations.map((citation) => `[${citation.index}] ${citation.title} (${citation.path})\n${citation.snippet}`).join('\n\n')
      try {
        const aiSummary = await callChatProvider([{ role: 'system', content: 'You create concise private wiki summaries. Use only the provided cited local notes. Keep citation markers like [1].' }, { role: 'user', content: `Topic: ${topic}\n\nCited local notes:\n${localContext}\n\nWrite a 5 sentence synthesis with citations.` }], providerConfig)
        if (aiSummary) summary = aiSummary.trim()
      } catch { /* deterministic fallback */ }
      records.push({ id, topic, title: topic.replace(/^#/, ''), summary, keyTerms, citations, status: 'proposed', confidence: Math.min(1, 0.35 + citations.length * 0.08), suggestedMarkdown: this.renderWikiMarkdown({ topic, summary, citations, keyTerms }) })
    }
    return { generatedAt: new Date().toISOString(), records: records.sort((a, b) => b.confidence - a.confidence || a.title.localeCompare(b.title)).slice(0, 40) }
  }

  renderWikiMarkdown({ topic, summary, citations, keyTerms }) {
    return `---\ntitle: "${String(topic || 'Wiki').replace(/"/g, '\\"')}"\ntype: "wiki"\ntags: ["wiki", "atomic"]\ncreatedAt: "${new Date().toISOString()}"\nupdatedAt: "${new Date().toISOString()}"\n---\n\n# ${topic}\n\n${summary}\n\n## Key terms\n\n${(keyTerms || []).map((term) => `- ${term}`).join('\n') || '- No key terms yet.'}\n\n## Citations\n\n${(citations || []).map((citation) => `- [${citation.index}] [[${citation.path}]] — ${citation.title}`).join('\n') || '- No citations.'}\n`
  }

  async createWikiPage({ vaultRoot, record = {}, windowId = null } = {}) {
    const root = assertVaultRoot(vaultRoot)
    const wikiDir = path.join(root, 'Wiki')
    await fs.ensureDir(wikiDir)
    const title = record.title || record.topic || 'Atomic Wiki Page'
    const filename = await nextAvailableName(wikiDir, `${safeFilename(title)}.md`)
    const fullPath = path.join(wikiDir, filename)
    const markdown = record.suggestedMarkdown || this.renderWikiMarkdown({ topic: title, summary: record.summary || '', citations: record.citations || [], keyTerms: record.keyTerms || [] })
    await fs.writeFile(fullPath, markdown, 'utf8')
    await getSearchService().indexFile(fullPath, windowId).catch(() => {})
    return { path: toRelativePath(root, fullPath), fullPath, title: path.basename(filename, '.md') }
  }

  async summarize({ vaultRoot, relativePath, providerConfig = {} } = {}) {
    const { root, fullPath } = assertRelativeNotePath(vaultRoot, relativePath)
    const note = await parseMarkdownNote(root, fullPath)
    let summary = note.summary
    try {
      const aiSummary = await callChatProvider([{ role: 'system', content: 'Summarize this local markdown note. Keep it factual, structured, and concise.' }, { role: 'user', content: note.markdown.slice(0, 18000) }], providerConfig)
      if (aiSummary) summary = aiSummary.trim()
    } catch { /* fallback summary exists */ }
    return { relativePath: note.path, title: note.title, summary, citations: [createCitation(note, 0)], keyTerms: note.keyTerms, headings: note.headings, actionItems: note.actionItems }
  }

  async structure({ vaultRoot, relativePath, providerConfig = {} } = {}) {
    const { root, fullPath } = assertRelativeNotePath(vaultRoot, relativePath)
    const note = await parseMarkdownNote(root, fullPath)
    const suggestedTags = [...new Set([...note.tags, ...note.keyTerms.slice(0, 6)])].slice(0, 10)
    let restructuring = ''
    try {
      restructuring = await callChatProvider([{ role: 'system', content: 'Suggest a better markdown structure for this note. Return headings and bullet sections only. Do not invent facts.' }, { role: 'user', content: note.markdown.slice(0, 18000) }], providerConfig)
    } catch { restructuring = '' }
    return { relativePath: note.path, title: note.title, outline: note.headings, suggestedTags, keyTerms: note.keyTerms, actionItems: note.actionItems, restructuring: restructuring || this.renderLocalStructure(note) }
  }

  renderLocalStructure(note) {
    const sections = note.headings.length ? note.headings.map((heading) => `${'  '.repeat(Math.max(0, heading.level - 1))}- ${heading.title}`).join('\n') : '- Summary\n- Details\n- Next actions\n- References'
    const tags = note.keyTerms.slice(0, 6).map((term) => `#${term}`).join(' ')
    return `Suggested outline:\n${sections}\n\nSuggested tags: ${tags || 'none'}\n\nSummary:\n${note.summary}`
  }

  async autoNameNote({ vaultRoot, relativePath, apply = true, providerConfig = {}, windowId = null } = {}) {
    const { root, normalized, fullPath } = assertRelativeNotePath(vaultRoot, relativePath)
    const note = await parseMarkdownNote(root, fullPath)
    let suggestedTitle = inferNoteTitle(note)
    try {
      const aiTitle = await callChatProvider([{ role: 'system', content: 'Return only a short filename-safe note title, no extension, no quotes.' }, { role: 'user', content: note.markdown.slice(0, 8000) }], providerConfig)
      if (aiTitle) suggestedTitle = safeFilename(aiTitle.replace(/\.md$/i, ''))
    } catch { /* local title remains */ }
    const targetDir = path.dirname(fullPath)
    const targetName = await nextAvailableName(targetDir, `${suggestedTitle}.md`, fullPath)
    const targetPath = path.join(targetDir, targetName)
    const targetRelativePath = toRelativePath(root, targetPath)
    const changed = targetPath !== fullPath
    if (apply && changed) {
      await fs.move(fullPath, targetPath, { overwrite: false })
      await getSearchService().deleteFile(fullPath, windowId).catch(() => {})
      await getSearchService().indexFile(targetPath, windowId).catch(() => {})
    }
    return { oldPath: normalized, newPath: targetRelativePath, title: path.basename(targetName, '.md'), changed: apply ? changed : false, suggested: !apply, weakTitle: isWeakTitle(note) }
  }

  async pullModel({ id, provider = 'node-llama-cpp', vaultRoot = '', onProgress = null } = {}) {
    const model = RECOMMENDED_MODELS.find((item) => item.id === id || item.browserModel === id || item.pull === id) || { id, provider }
    if (!model.id) throw new Error('Model id is required.')
    const modelDir = vaultRoot ? await getVaultModelDir(vaultRoot) : ''
    if (modelDir) {
      await fs.writeJson(path.join(modelDir, `${slugify(model.id)}.json`), {
        ...model,
        installed: false,
        managedBy: 'node-llama-cpp',
        updatedAt: new Date().toISOString()
      }, { spaces: 2 })
    }
    onProgress?.({
      id: model.id,
      modelId: model.id,
      name: model.name || model.id,
      phase: 'ready',
      percent: 100,
      message: 'Model metadata is ready for node-llama-cpp.'
    })
    return {
      id: model.id,
      provider: model.provider || provider,
      downloaded: false,
      modelDir,
      message: 'Model metadata saved. Download the GGUF from Settings > AI with node-llama-cpp.'
    }
  }

  async listLocalModels({ vaultRoot = '' } = {}) {
    const modelDir = vaultRoot ? await getVaultModelDir(vaultRoot) : ''
    const vaultModels = modelDir
      ? (await fs.readdir(modelDir).catch(() => []))
        .filter((file) => file.endsWith('.json'))
        .map((file) => path.join(modelDir, file))
      : []
    const vaultModelMetadata = []
    for (const file of vaultModels) {
      const item = await fs.readJson(file).catch(() => null)
      if (item) vaultModelMetadata.push(item)
    }
    return {
      provider: 'node-llama-cpp',
      available: true,
      modelDir,
      vaultModels: vaultModelMetadata,
      models: [],
      raw: '',
      message: 'node-llama-cpp models are inspected in the Electron main process.'
    }
  }
}
