import { buildSemanticGraphSurface, selectSemanticGraphSource } from './semanticGraphViewHelpers'

const safeArray = (value) => Array.isArray(value) ? value : []

export const buildWikiGraphPanel = ({
  inspectionGraph = null,
  fallbackGraph = null,
  includeStructure = false
} = {}) => {
  const graph = selectSemanticGraphSource({
    inspectionGraph,
    fallbackGraph
  })
  const surface = buildSemanticGraphSurface({
    graph,
    fallback: fallbackGraph,
    includeStructure
  })
  const nodes = safeArray(surface.nodes)
  const edges = safeArray(surface.edges)
  const clusters = safeArray(surface.clusters)
  return {
    graph,
    surface,
    summary: {
      nodes: nodes.length,
      semanticEdges: edges.filter((edge) => edge.type === 'semantic').length,
      structureEdges: edges.filter((edge) => edge.type !== 'semantic').length,
      clusters: clusters.length,
      sources: nodes.reduce((total, node) => total + Number(node.sourceCount || 0), 0)
    },
    clusters: clusters.slice(0, 8)
  }
}

export const buildWikiRecordCard = (record = {}) => {
  const citations = safeArray(record.citations)
  return {
    id: String(record.id || ''),
    title: String(record.title || record.topic || 'Wiki topic'),
    topic: String(record.topic || record.title || 'Wiki topic'),
    summary: String(record.summary || ''),
    status: String(record.status || 'proposed'),
    citationCount: citations.length,
    citations: citations.map((citation) => ({
      path: String(citation.path || ''),
      title: String(citation.title || citation.path || 'Source'),
      excerpt: String(citation.excerpt || '').trim(),
      updatedAt: String(citation.updatedAt || '')
    })),
    notePath: String(record.notePath || ''),
    tags: safeArray(record.tags)
  }
}

export const buildWikiSourceCard = ({
  selectedRecord = null,
  selectedCitation = null,
  sourceInsight = null
} = {}) => {
  const source = sourceInsight?.source || null
  const citation = selectedCitation || null
  const relatedNodes = safeArray(sourceInsight?.relatedNodes)
  return {
    source: source ? {
      path: String(source.path || citation?.path || ''),
      title: String(source.title || citation?.title || source.path || 'Source'),
      summary: String(source.summary || citation?.excerpt || '').trim(),
      kind: String(source.kind || 'note'),
      sourceCount: Number(source.sourceCount || 0),
      chunkCount: Number(source.chunkCount || 0),
      updatedAt: String(source.updatedAt || ''),
      tags: safeArray(source.tags)
    } : null,
    citation: citation ? {
      path: String(citation.path || ''),
      title: String(citation.title || citation.path || 'Source'),
      excerpt: String(citation.excerpt || '').trim(),
      updatedAt: String(citation.updatedAt || '')
    } : null,
    selectedRecord: selectedRecord ? buildWikiRecordCard(selectedRecord) : null,
    relatedNodes: relatedNodes.map((node) => ({
      id: String(node.id || ''),
      title: String(node.title || node.id || 'Untitled'),
      summary: String(node.summary || '').trim(),
      kind: String(node.kind || 'note'),
      linkType: String(node.linkType || 'related'),
      weight: Number(node.weight || 0),
      tags: safeArray(node.tags)
    })),
    cluster: sourceInsight?.cluster ? {
      id: String(sourceInsight.cluster.id || ''),
      label: String(sourceInsight.cluster.label || sourceInsight.cluster.id || 'Cluster'),
      nodeCount: Number(sourceInsight.cluster.nodeCount || 0),
      paths: safeArray(sourceInsight.cluster.paths)
    } : null
  }
}
