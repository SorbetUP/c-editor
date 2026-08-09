import { createWikiMarkdown as createSharedWikiMarkdown, normalizeWikiRecord } from 'common/elephantnote/wiki'

const slugify = (value = '') =>
  String(value || '')
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '') || 'topic'

const getNodePath = (node = {}) => String(node.relativePath || node.path || node.id || '')

const isNoteLikeNode = (node = {}) => {
  const kind = String(node.kind || node.type || '').toLowerCase()
  const nodePath = getNodePath(node)
  if (!nodePath) return false
  if (kind === 'folder' || kind === 'directory') return false
  if (kind === 'note') return true
  return /\.md$/i.test(nodePath)
}

const createCitation = (node = {}) => ({
  path: getNodePath(node),
  title: String(node.title || node.relativePath || node.path || node.id || 'Untitled'),
  excerpt: String(node.summary || node.plainText || '').trim(),
  updatedAt: String(node.updatedAt || ''),
  kind: String(node.kind || 'note'),
  tags: Array.isArray(node.tags) ? node.tags : [],
  sourceCount: Number(node.sourceCount || node.sources?.length || 0),
  chunkCount: Number(node.chunkCount || node.chunks?.length || 0)
})

const toNodeMap = (graph = {}) => {
  const nodes = Array.isArray(graph?.nodes) ? graph.nodes : []
  return new Map(nodes.map((node) => [getNodePath(node), node]))
}

const createGraphWikiSummary = ({ topic, nodeCount, semanticLinkCount, sourceCount }) => {
  const nodePhrase = `${nodeCount} note${nodeCount === 1 ? '' : 's'}`
  const linkPhrase = semanticLinkCount
    ? ` and ${semanticLinkCount} semantic link${semanticLinkCount === 1 ? '' : 's'}`
    : ''
  const sourcePhrase = sourceCount
    ? ` from ${sourceCount} cited source${sourceCount === 1 ? '' : 's'}`
    : ''
  return `This wiki proposal connects ${nodePhrase} around ${topic}${linkPhrase}${sourcePhrase}.`
}

export const buildWikiProposalsFromGraph = ({
  graph = {},
  now = new Date()
} = {}) => {
  const nodes = Array.isArray(graph?.nodes) ? graph.nodes : []
  const edges = Array.isArray(graph?.edges) ? graph.edges : []
  const clusters = Array.isArray(graph?.clusters) ? graph.clusters : []
  const noteNodes = nodes.filter(isNoteLikeNode)
  const byPath = new Map(noteNodes.map((node) => [getNodePath(node), node]))
  const semanticEdges = edges.filter((edge) => edge?.type === 'semantic')
  const proposals = []

  for (const cluster of clusters) {
    const clusterPaths = Array.isArray(cluster?.paths) ? cluster.paths.map((relativePath) => String(relativePath || '')) : []
    const clusterPathSet = new Set(clusterPaths)
    const clusterNodes = clusterPaths.map((relativePath) => byPath.get(relativePath)).filter(Boolean)
    if (clusterNodes.length < 2) continue

    const citations = clusterNodes
      .slice(0, 8)
      .map(createCitation)
      .filter((citation) => citation.path)
    if (citations.length < 2) continue

    const topic = String(cluster.label || cluster.id || citations[0]?.title || 'Wiki topic').trim()
    const sourceCount = clusterNodes.reduce((total, node) => total + Number(node.sourceCount || node.sources?.length || 0), 0)
    const semanticLinkCount = semanticEdges.filter((edge) =>
      clusterPathSet.has(String(edge.source || '')) || clusterPathSet.has(String(edge.target || ''))
    ).length

    proposals.push(normalizeWikiRecord({
      id: `wiki-${slugify(topic)}`,
      topic,
      title: topic,
      summary: createGraphWikiSummary({
        topic,
        nodeCount: clusterNodes.length,
        semanticLinkCount,
        sourceCount
      }),
      citations,
      status: 'proposed',
      createdAt: now.toISOString(),
      updatedAt: now.toISOString()
    }))
  }

  return proposals.sort((a, b) => {
    if (b.citations.length !== a.citations.length) return b.citations.length - a.citations.length
    return a.topic.localeCompare(b.topic)
  })
}

export const buildWikiSourceInsight = ({
  graph = {},
  path: sourcePath = '',
  record = null
} = {}) => {
  const lookupPath = String(sourcePath || '').trim()
  if (!lookupPath) {
    return {
      source: null,
      relatedNodes: [],
      cluster: null
    }
  }

  const nodes = Array.isArray(graph?.nodes) ? graph.nodes : []
  const edges = Array.isArray(graph?.edges) ? graph.edges : []
  const clusters = Array.isArray(graph?.clusters) ? graph.clusters : []
  const byPath = toNodeMap(graph)
  const node = byPath.get(lookupPath) || nodes.find((item) => getNodePath(item) === lookupPath) || null
  const selectedSources = Array.isArray(record?.citations) ? record.citations : []
  const relatedNodes = []
  const byId = new Map(nodes.map((item) => [getNodePath(item), item]))

  for (const edge of edges) {
    if (edge.source !== lookupPath && edge.target !== lookupPath) continue
    const otherPath = edge.source === lookupPath ? edge.target : edge.source
    const otherNode = byId.get(String(otherPath || ''))
    if (!otherNode) continue
    relatedNodes.push({
      id: getNodePath(otherNode),
      title: String(otherNode.title || otherNode.relativePath || otherNode.path || otherNode.id || 'Untitled'),
      summary: String(otherNode.summary || ''),
      kind: String(otherNode.kind || 'note'),
      linkType: String(edge.type || 'related'),
      weight: Number(edge.weight || 0),
      tags: Array.isArray(otherNode.tags) ? otherNode.tags : []
    })
  }

  const cluster = clusters.find((item) =>
    Array.isArray(item?.paths) && item.paths.map((clusterPath) => String(clusterPath || '')).includes(lookupPath)
  ) || null

  const source = node ? {
    path: lookupPath,
    title: String(node.title || node.relativePath || node.path || node.id || 'Untitled'),
    summary: String(node.summary || node.plainText || '').trim(),
    kind: String(node.kind || 'note'),
    tags: Array.isArray(node.tags) ? node.tags : [],
    sourceCount: Number(node.sourceCount || node.sources?.length || 0),
    chunkCount: Number(node.chunkCount || node.chunks?.length || 0),
    updatedAt: String(node.updatedAt || ''),
    sources: Array.isArray(node.sources) ? node.sources : []
  } : null

  const citations = selectedSources
    .filter((citation) => citation?.path)
    .map((citation) => ({
      path: String(citation.path),
      title: String(citation.title || citation.path),
      excerpt: String(citation.excerpt || '').trim(),
      updatedAt: String(citation.updatedAt || '')
    }))

  return {
    source,
    citations,
    relatedNodes: relatedNodes
      .sort((a, b) => b.weight - a.weight || a.title.localeCompare(b.title))
      .slice(0, 12),
    cluster: cluster ? {
      id: String(cluster.id || ''),
      label: String(cluster.label || cluster.id || 'Cluster'),
      nodeCount: Number(cluster.nodeCount || 0),
      paths: Array.isArray(cluster.paths) ? cluster.paths : []
    } : null
  }
}

export const buildWikiChatContext = ({
  graph = {},
  path: sourcePath = '',
  record = null,
  limit = 12
} = {}) => {
  const insight = buildWikiSourceInsight({ graph, path: sourcePath, record })
  const graphNodes = Array.isArray(graph?.nodes) ? graph.nodes : []
  const graphEdges = Array.isArray(graph?.edges) ? graph.edges : []
  return {
    ...insight,
    graphSummary: {
      nodes: graphNodes.length,
      semanticLinks: graphEdges.filter((edge) => edge.type === 'semantic').length,
      clusters: Array.isArray(graph?.clusters) ? graph.clusters.length : 0
    },
    relatedNodes: insight.relatedNodes.slice(0, Math.max(1, Number(limit) || 12)),
    citations: insight.citations.slice(0, Math.max(1, Number(limit) || 12))
  }
}

export const createGraphBackedWikiMarkdown = (proposal = {}, now = new Date()) => {
  const record = normalizeWikiRecord({
    ...proposal,
    updatedAt: proposal.updatedAt || now.toISOString()
  })
  const citationLines = record.citations.length
    ? record.citations.map((citation, index) => `- [${index + 1}] [[${citation.path}]]${citation.excerpt ? ` — ${citation.excerpt}` : ''}`).join('\n')
    : '- No citations yet.'
  const sourcePaths = record.citations.map((citation) => citation.path).filter(Boolean)

  return `${createSharedWikiMarkdown(record, now).trimEnd()}

## Related graph

- Topic: ${record.topic || 'Wiki'}
- Citations: ${record.citations.length}
- Source notes: ${sourcePaths.length}

## Sources

${citationLines}
`
}
