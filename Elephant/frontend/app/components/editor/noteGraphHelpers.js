const normalizeNodeId = (node = {}) => String(node.relativePath || node.path || node.id || '').trim()

const pickRelatedNodeKey = (node = {}) => normalizeNodeId(node)

const safeArray = (value) => Array.isArray(value) ? value : []

export const buildNoteGraphPreview = ({
  graph = null,
  notePath = '',
  limit = 5,
  includeStructure = false
} = {}) => {
  const nodes = Array.isArray(graph?.nodes) ? graph.nodes : []
  const edges = Array.isArray(graph?.edges) ? graph.edges : []
  const currentNode = nodes.find((node) => pickRelatedNodeKey(node) === notePath)
  if (!currentNode) return null

  const byId = new Map(nodes.map((node) => [pickRelatedNodeKey(node), node]))
  const incidentEdges = edges.filter((edge) => {
    if (edge?.source !== notePath && edge?.target !== notePath) return false
    if (includeStructure) return true
    return edge?.type === 'semantic' || edge?.type === 'explicit-link'
  })
  const related = new Map()

  for (const edge of incidentEdges) {
    const otherId = edge.source === notePath ? edge.target : edge.source
    const otherNode = byId.get(otherId)
    if (!otherNode) continue
    const previous = related.get(otherId)
    const score = Number(edge.weight || 0)
    const nextEntry = {
      id: otherId,
      title: otherNode.title || otherNode.name || otherId,
      summary: otherNode.summary || '',
      kind: otherNode.kind || 'note',
      tags: Array.isArray(otherNode.tags) ? otherNode.tags : [],
      weight: Math.max(previous?.weight || 0, score),
      types: new Set([...(previous?.types || []), edge.type || 'related'])
    }
    related.set(otherId, nextEntry)
  }

  const cluster = Array.isArray(graph?.clusters)
    ? graph.clusters.find((item) => Array.isArray(item?.paths) && item.paths.includes(notePath))
    : null
  const sources = safeArray(currentNode.sources).map((source) => ({
    path: String(source.path || source.url || source.id || '').trim(),
    title: String(source.title || source.path || source.url || 'Source'),
    excerpt: String(source.excerpt || source.summary || '').trim(),
    type: String(source.type || 'note')
  })).filter((source) => source.path || source.title)

  const linkTypeCounts = new Map()
  for (const edge of incidentEdges) {
    const type = String(edge.type || 'related')
    linkTypeCounts.set(type, (linkTypeCounts.get(type) || 0) + 1)
  }

  return {
    node: currentNode,
    relatedNodes: [...related.values()]
      .sort((a, b) => b.weight - a.weight || String(a.title).localeCompare(String(b.title)))
      .slice(0, Math.max(1, limit))
      .map((entry) => ({
        ...entry,
        types: [...entry.types]
      })),
      stats: {
      totalNodes: nodes.length,
      totalLinks: edges.length,
      semanticLinks: edges.filter((edge) => edge.type === 'semantic').length,
      incidentLinks: incidentEdges.length,
      clusters: Array.isArray(graph?.clusters) ? graph.clusters.length : 0
    },
    cluster: cluster ? {
      id: String(cluster.id || ''),
      label: String(cluster.label || cluster.id || 'Cluster'),
      nodeCount: Number(cluster.nodeCount || cluster.paths?.length || 0),
      paths: safeArray(cluster.paths).map((path) => String(path || '').trim()).filter(Boolean),
      tags: safeArray(cluster.tags)
    } : null,
    sources,
    linkTypeCounts: [...linkTypeCounts.entries()]
      .sort((a, b) => b[1] - a[1] || String(a[0]).localeCompare(String(b[0])))
      .map(([type, count]) => ({ type, count }))
  }
}
