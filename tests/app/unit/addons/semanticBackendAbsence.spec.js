import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import { ELEPHANTNOTE_API_ACTIONS } from '../../../../Elephant/shared/apiContracts.js'
import { COMPATIBILITY_CALLS } from '../../../../Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('legacy semantic backend physical absence', () => {
  it('does not ship the old embedding database module in Elephant core', () => {
    expect(fs.existsSync(path.join(root, 'Elephant/backend/tauri/src/embeddings.rs'))).toBe(false)
    const lib = read('Elephant/backend/tauri/src/lib_min.rs')
    expect(lib).not.toContain('pub mod embeddings;')
    expect(lib).not.toContain('embeddings::tauri_embeddings_')
  })

  it('does not expose core semantic inspection or rebuild commands', () => {
    const lib = read('Elephant/backend/tauri/src/lib_min.rs')
    const commands = read('Elephant/backend/tauri/src/core_commands.rs')
    expect(fs.existsSync(path.join(root, 'Elephant/backend/tauri/src/tauri_extra_commands.rs'))).toBe(false)
    expect(lib).not.toContain('tauri_extra_commands::tauri_search_rebuild')
    expect(lib).not.toContain('tauri_extra_commands::tauri_search_inspect')
    expect(commands).not.toContain('tauri_search_rebuild')
    expect(commands).not.toContain('tauri_search_inspect')
    expect(commands).not.toContain('portable-markdown-index')
  })

  it('keeps only native filename and text search as a core capability', () => {
    const lib = read('Elephant/backend/tauri/src/lib_min.rs')
    expect(lib).toContain('vault::commands::tauri_search_query')
    expect(lib).toContain('vault::commands::tauri_search_status')
    expect(ELEPHANTNOTE_API_ACTIONS.SEARCH_QUERY).toBe('search.query')
    expect(ELEPHANTNOTE_API_ACTIONS.SEARCH_STATUS).toBe('search.status')
    expect(ELEPHANTNOTE_API_ACTIONS.SEARCH_INSPECT).toBeUndefined()
    expect(ELEPHANTNOTE_API_ACTIONS.SEARCH_REBUILD).toBeUndefined()
    expect(COMPATIBILITY_CALLS['search.query']).toBeTypeOf('function')
    expect(COMPATIBILITY_CALLS['search.status']).toBeTypeOf('function')
    expect(COMPATIBILITY_CALLS['search.inspect']).toBeUndefined()
  })

  it('keeps semantic indexing in Knowledge and orchestration in the Search package', () => {
    const source = read('addons/official/ai-search/main.js')
    const semantic = read('addons/official/ai-search/semanticEmbeddingSync.js')
    const knowledge = read('addons/official/knowledge/main.js')
    expect(source).toContain("const KNOWLEDGE_RESOURCE = 'knowledge.provider'")
    expect(source).toContain("const AI_INFERENCE_RESOURCE = 'ai.inference'")
    expect(source).toContain("const LOCAL_ENGINE = Object.freeze({ engine: 'package-owned-bm25' })")
    expect(source).toContain("this.knowledgeProvider() ? 'knowledge-provider' : LOCAL_ENGINE.engine")
    expect(source).toContain('api.storage.set(INDEX_KEY, this.index)')
    expect(source).toContain('api.resources.provide(PROVIDER_RESOURCE')
    expect(semantic).toContain('pendingEmbeddings')
    expect(semantic).toContain('saveEmbeddings')
    expect(knowledge).toContain('semanticCommunities')
    expect(knowledge).toContain('semanticDiscover')
  })
})
