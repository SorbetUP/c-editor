const normalizeSearchPath = (value = '') => String(value || '').replace(/\\/g, '/').split('/').filter(Boolean).join('/')

const titleFromPath = (path = '') => (normalizeSearchPath(path).split('/').pop() || 'Concept').replace(/\.md$/i, '')

export const installTauriSearchConceptFallback = (target = globalThis) => {
  if (!target.__TAURI__ || !target.elephantnote?.search || typeof target.elephantnote.search.concepts === 'function') {
    return false
  }

  target.elephantnote.search.concepts = async(params = {}) => {
    const query = String(params.query || params.q || '').trim()
    const limit = Math.max(1, Math.min(20, Number(params.limit) || 5))
    const evidenceLimit = Math.max(1, Math.min(20, Number(params.evidenceLimit) || 4))
    console.info('[search] concepts:fallback:start', { query, limit, evidenceLimit })
    if (!query) {
      return { runtime: 'tauri-js-fallback', query, candidates: [], ambiguous: false, reason: 'empty-query' }
    }

    const results = await target.elephantnote.search.query({ query, mode: 'exact', limit: Math.max(limit * evidenceLimit, limit) })
    const items = Array.isArray(results) ? results : []
    const groups = new Map()
    for (const result of items) {
      const path = normalizeSearchPath(result.relativePath || result.path || '')
      if (!path) continue
      const parts = path.split('/').filter(Boolean)
      const id = parts.length > 1 ? parts[0] : titleFromPath(path)
      const current = groups.get(id) || {
        id,
        title: id,
        score: 0,
        evidenceChunks: []
      }
      current.score += Number(result.score || 1)
      if (current.evidenceChunks.length < evidenceLimit) {
        current.evidenceChunks.push({
          path,
          title: result.title || titleFromPath(path),
          excerpt: result.excerpt || result.preview || '',
          score: Number(result.score || 1)
        })
      }
      groups.set(id, current)
    }

    const candidates = [...groups.values()]
      .sort((a, b) => b.score - a.score)
      .slice(0, limit)
    const route = {
      runtime: 'tauri-js-fallback',
      query,
      candidates,
      ambiguous: candidates.length > 1,
      reason: 'rust-search-concepts-command-unavailable'
    }
    console.info('[search] concepts:fallback:done', {
      query,
      candidates: candidates.length,
      evidence: candidates.reduce((sum, candidate) => sum + candidate.evidenceChunks.length, 0)
    })
    return route
  }

  console.info('[search] concepts:fallback:installed')
  return true
}
