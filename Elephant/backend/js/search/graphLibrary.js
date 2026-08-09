const byCountDesc = (a, b) => b[1] - a[1] || String(a[0]).localeCompare(String(b[0]))

const GRAPH_STOP_WORDS = new Set([
  'and',
  'app',
  'avec',
  'dans',
  'des',
  'for',
  'from',
  'les',
  'note',
  'notes',
  'the',
  'une',
  'wiki',
  'with'
])

const normalizeGraphText = (value = '') => String(value || '')
  .normalize('NFKD')
  .replace(/[\u0300-\u036f]/g, '')
  .toLowerCase()

const tokenizeGraphText = (value = '') => normalizeGraphText(value)
  .match(/[a-z0-9][a-z0-9_-]{2,}/g)?.filter((token) => !GRAPH_STOP_WORDS.has(token)) || []

const stableGraphId = (value = '') => {
  let hash = 2166136261
  const normalized = normalizeGraphText(value)
  for (let index = 0; index < normalized.length; index += 1) {
    hash ^= normalized.charCodeAt(index)
    hash = Math.imul(hash, 16777619)
  }
  return (hash >>> 0).toString(36)
}

const getDocumentPath = (document = {}) => String(document.relativePath || document.path || document.id || '')

const collectDocumentNodes = (documents = []) => {
  return documents.map((document) => ({
    id: getDocumentPath(document),
    title: document.title,
    kind: 'note',
    relativePath: getDocumentPath(document),
    summary: document.summary || document.plainText?.slice(0, 240) || '',
    sources: document.sources || [],
    tags: document.tags || [],
    chunkCount: Array.isArray(document.chunks) ? document.chunks.length : Number(document.chunkCount || 0),
    sourceCount: Array.isArray(document.sources) ? document.sources.length : Number(document.sourceCount || 0)
  })).filter((node) => node.relativePath)
}

const collectFolderNodes = (documents = []) => {
  const folders = new Map()
  for (const document of documents) {
    const documentPath = getDocumentPath(document)
    const folderPath = documentPath.includes('/')
      ? documentPath.split('/').slice(0, -1).join('/')
      : ''
    if (!folderPath) continue
    if (!folders.has(folderPath)) {
      folders.set(folderPath, {
        id: folderPath,
        title: folderPath.split('/').pop() || folderPath,
        kind: 'folder',
        relativePath: folderPath
      })
    }
  }
  return [...folders.values()]
}

const collectTagEdges = (documents = []) => {
  const byTag = new Map()
  for (const document of documents) {
    for (const tag of document.tags || []) {
      if (!tag) continue
      if (!byTag.has(tag)) byTag.set(tag, [])
      byTag.get(tag).push(document)
    }
  }
  const edges = []
  for (const [tag, taggedDocuments] of byTag.entries()) {
    if (taggedDocuments.length < 2) continue
    for (let index = 1; index < taggedDocuments.length; index += 1) {
      const source = getDocumentPath(taggedDocuments[0])
      const target = getDocumentPath(taggedDocuments[index])
      if (!source || !target) continue
      edges.push({
        id: `tag:${source}->${target}#${tag}`,
        source,
        target,
        type: 'tag',
        reason: `#${tag}`,
        weight: 0.58
      })
    }
  }
  return edges
}

const collectFolderEdges = (documents = []) => {
  const edges = []
  for (const document of documents) {
    const documentPath = getDocumentPath(document)
    const folderPath = documentPath.includes('/')
      ? documentPath.split('/').slice(0, -1).join('/')
      : ''
    if (!folderPath) continue
    edges.push({
      id: `folder:${folderPath}->${documentPath}`,
      source: folderPath,
      target: documentPath,
      type: 'folder',
      reason: 'folder',
      weight: 0.34
    })
  }
  return edges
}

const createClusterLabel = (nodes = [], fallback = 'Cluster') => {
  const counts = new Map()
  for (const node of nodes) {
    for (const tag of node.tags || []) {
      if (!tag) continue
      counts.set(tag, (counts.get(tag) || 0) + 3)
    }
    for (const token of tokenizeGraphText(`${node.title || ''} ${node.relativePath || ''}`)) {
      counts.set(token, (counts.get(token) || 0) + 1)
    }
  }
  const terms = [...counts.entries()].sort(byCountDesc).slice(0, 3).map(([term]) => term)
  if (!terms.length) return fallback
  return terms.map((term) => term.replace(/[-_]+/g, ' ')).join(' · ')
}

const createSemanticClusters = ({ documentNodes = [], edges = [], minWeight = 0.5 } = {}) => {
  const noteIds = new Set(documentNodes.map((node) => node.relativePath))
  const adjacency = new Map(documentNodes.map((node) => [node.relativePath, []]))
  const semanticEdges = edges.filter((edge) =>
    noteIds.has(edge.source) &&
    noteIds.has(edge.target) &&
    ['semantic', 'tag'].includes(edge.type) &&
    Number(edge.weight || 0) >= minWeight
  )

  for (const edge of semanticEdges) {
    adjacency.get(edge.source)?.push(edge)
    adjacency.get(edge.target)?.push(edge)
  }

  const byPath = new Map(documentNodes.map((node) => [node.relativePath, node]))
  const visited = new Set()
  const clusters = []

  for (const node of documentNodes) {
    if (visited.has(node.relativePath)) continue
    const stack = [node.relativePath]
    const paths = []
    const componentEdges = []
    visited.add(node.relativePath)

    while (stack.length) {
      const current = stack.pop()
      paths.push(current)
      for (const edge of adjacency.get(current) || []) {
        componentEdges.push(edge)
        const next = edge.source === current ? edge.target : edge.source
        if (visited.has(next)) continue
        visited.add(next)
        stack.push(next)
      }
    }

    if (paths.length < 2) continue
    const componentNodes = paths.map((path) => byPath.get(path)).filter(Boolean)
    const uniqueEdges = [...new Map(componentEdges.map((edge) => [edge.id, edge])).values()]
    const cohesion = uniqueEdges.length
      ? uniqueEdges.reduce((sum, edge) => sum + Number(edge.weight || 0), 0) / uniqueEdges.length
      : 0
    const tags = [...new Map(componentNodes.flatMap((item) => item.tags || []).map((tag) => [tag, tag])).values()]
    const label = createClusterLabel(componentNodes, 'Semantic cluster')
    clusters.push({
      id: `semantic:${stableGraphId(paths.sort().join('|'))}`,
      kind: 'semantic',
      type: 'semantic',
      label,
      nodeCount: componentNodes.length,
      paths: paths.sort((a, b) => a.localeCompare(b)),
      tags: tags.slice(0, 8),
      cohesion: Number(cohesion.toFixed(4)),
      edgeCount: uniqueEdges.length
    })
  }

  return clusters
}

const createFolderClusters = (documentNodes = []) => {
  const byFolder = new Map()
  for (const node of documentNodes) {
    const folderPath = String(node.relativePath || '').includes('/')
      ? String(node.relativePath).split('/').slice(0, -1).join('/')
      : 'root'
    if (!byFolder.has(folderPath)) {
      byFolder.set(folderPath, {
        id: folderPath,
        kind: 'folder',
        type: 'folder',
        label: folderPath === 'root' ? 'Root' : folderPath.split('/').pop() || folderPath,
        nodeCount: 0,
        paths: [],
        tags: new Map(),
        cohesion: 0.34
      })
    }
    const cluster = byFolder.get(folderPath)
    cluster.nodeCount += 1
    cluster.paths.push(node.relativePath)
    for (const tag of node.tags || []) {
      if (!tag) continue
      cluster.tags.set(tag, (cluster.tags.get(tag) || 0) + 1)
    }
  }
  return [...byFolder.values()].map((cluster) => ({
    ...cluster,
    tags: [...cluster.tags.entries()].sort(byCountDesc).slice(0, 8).map(([tag]) => tag)
  }))
}

export const createSemanticGraph = ({
  documents = [],
  semanticLinks = [],
  includeFolderEdges = true,
  includeTagEdges = true
} = {}) => {
  const documentNodes = collectDocumentNodes(documents)
  const folderNodes = collectFolderNodes(documents)
  const nodes = [...folderNodes, ...documentNodes]

  const edges = []
  if (includeFolderEdges) edges.push(...collectFolderEdges(documents))
  if (includeTagEdges) edges.push(...collectTagEdges(documents))
  for (const link of semanticLinks || []) {
    if (!link?.source || !link?.target) continue
    edges.push({
      id: link.id || `semantic:${link.source}->${link.target}`,
      source: link.source,
      target: link.target,
      type: 'semantic',
      reason: link.reason || 'embedding-similarity',
      weight: Number(link.score || link.weight || 0)
    })
  }

  const semanticClusters = createSemanticClusters({ documentNodes, edges })
  const folderClusters = createFolderClusters(documentNodes)
  const clusters = [...semanticClusters, ...folderClusters]

  return {
    nodes,
    edges: edges.sort((a, b) => b.weight - a.weight || a.id.localeCompare(b.id)),
    clusters: clusters.sort((a, b) => {
      const semanticDelta = (b.kind === 'semantic' ? 1 : 0) - (a.kind === 'semantic' ? 1 : 0)
      if (semanticDelta) return semanticDelta
      if (b.nodeCount !== a.nodeCount) return b.nodeCount - a.nodeCount
      return a.label.localeCompare(b.label)
    })
  }
}
