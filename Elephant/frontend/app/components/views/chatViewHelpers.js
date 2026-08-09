const DEFAULT_QUICK_PROMPTS = [
  {
    label: 'Summarize context',
    hint: 'Use the active note context',
    icon: 'graph',
    prompt: 'Summarize the active note context and cite the most relevant sources.'
  },
  {
    label: 'Find follow-ups',
    hint: 'Surface related notes',
    icon: 'link',
    prompt: 'Find the best follow-up notes or open questions connected to this context.'
  },
  {
    label: 'Draft summary',
    hint: 'Prepare a cited topic',
    icon: 'doc',
    prompt: 'Draft a concise summary for the active context with citations.'
  },
  {
    label: 'Explain sources',
    hint: 'Clarify why they matter',
    icon: 'source',
    prompt: 'Explain why the cited sources matter and what each one contributes.'
  }
]

const safeArray = (value) => Array.isArray(value) ? value : []

export const buildChatContextPanel = ({ graph = null, limit = 4 } = {}) => {
  const nodes = safeArray(graph?.nodes)
  const edges = safeArray(graph?.edges)
  const clusters = safeArray(graph?.clusters)
  const normalizedLimit = Math.max(1, Number(limit) || 4)
  return {
    summary: {
      nodes: nodes.length,
      edges: edges.length,
      clusters: clusters.length
    },
    clusters: clusters.slice(0, normalizedLimit).map((cluster) => ({
      id: String(cluster?.id || cluster?.label || ''),
      label: String(cluster?.label || cluster?.id || 'Cluster'),
      nodeCount: Number(cluster?.nodeCount || safeArray(cluster?.nodes).length || 0),
      paths: safeArray(cluster?.paths)
    })),
    quickPrompts: DEFAULT_QUICK_PROMPTS
  }
}

const summarizeWikiContext = (wikiContext = null) => {
  if (!wikiContext) return null
  const source = wikiContext.source || {}
  const graphSummary = wikiContext.graphSummary || {}
  return {
    name: 'wiki.context',
    label: source.title || source.path || 'Wiki context',
    status: 'done',
    summary: `${graphSummary.nodes || 0} nodes · ${graphSummary.semanticLinks || 0} semantic links · ${graphSummary.clusters || 0} clusters`,
    sources: [source].filter((entry) => entry && (entry.path || entry.title))
  }
}

const summarizeCitations = (citations = []) => {
  if (!Array.isArray(citations) || citations.length === 0) return null
  return {
    name: 'rag.search',
    label: `Retrieved ${citations.length} cited note${citations.length === 1 ? '' : 's'}`,
    status: 'done',
    summary: citations
      .slice(0, 3)
      .map((citation) => citation.title || citation.path)
      .join(', '),
    sources: citations
      .slice(0, 8)
      .map((citation) => ({
        path: citation.path,
        title: citation.title || citation.path,
        score: citation.score
      }))
  }
}

export const shapeToolCallsForAssistant = (result = {}) => {
  const calls = []
  const citations = result?.citations || []
  const wikiContext = result?.wikiContext || null

  const searchCall = summarizeCitations(citations)
  if (searchCall) calls.push(searchCall)

  const wikiCall = summarizeWikiContext(wikiContext)
  if (wikiCall) calls.push(wikiCall)

  return calls
}

export const formatChatTimestamp = (timestamp) => {
  if (!timestamp) return ''
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return ''
  const now = new Date()
  const sameDay =
    date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate()
  const time = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  if (sameDay) return time
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${month}/${day} · ${time}`
}
