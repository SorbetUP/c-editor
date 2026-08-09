import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const absolute = (file) => path.join(root, file)
const read = (file) => fs.readFileSync(absolute(file), 'utf8')

describe('Search physical package ownership', () => {
  it('indexes notes through permission-scoped addon commands', () => {
    const manifest = JSON.parse(read('addons/official/ai-search/manifest.json'))
    const source = read('addons/official/ai-search/main.js')

    expect(manifest.version).toBe('1.3.0')
    expect(manifest.permissions.notes.read).toEqual(['*'])
    expect(source).toContain('tauri_addons_notes_list')
    expect(source).toContain('tauri_addons_notes_read')
    expect(source).toContain('api.resources.provide(PROVIDER_RESOURCE')
    expect(source).toContain("const LOCAL_ENGINE = Object.freeze({ engine: 'package-owned-bm25' })")
    expect(source).toContain("this.knowledgeProvider() ? 'knowledge-provider' : LOCAL_ENGINE.engine")
    expect(source).toContain('query: (text, options) => this.query(text, options)')
    expect(source).toContain('rebuild: () => this.rebuild()')
    expect(source).toContain('clear: () => this.clear()')
    expect(source).toContain('status: () => this.status()')
  })

  it('composes with package-owned Knowledge and AI inference without moving ownership to core', () => {
    const source = read('addons/official/ai-search/main.js')
    const semantic = read('addons/official/ai-search/semanticEmbeddingSync.js')
    expect(source).toContain("const KNOWLEDGE_RESOURCE = 'knowledge.provider'")
    expect(source).toContain("const AI_INFERENCE_RESOURCE = 'ai.inference'")
    expect(source).toContain('synchronizeKnowledgeEmbeddings')
    expect(source).toContain("engine: 'knowledge-provider'")
    expect(source).toContain('using local fallback')
    expect(semantic).toContain('pendingEmbeddings')
    expect(semantic).toContain('saveEmbeddings')
  })

  it('does not call the legacy global Search actions', () => {
    const source = read('addons/official/ai-search/main.js')
    for (const action of ['search.status', 'search.rebuild', 'search.clear', 'search.enable', 'search.disable']) {
      expect(source).not.toContain(action)
    }
    expect(source).not.toContain('elephantnote.api')
  })

  it('keeps the parallel inspection index and optional fallbacks physically absent from the core shell', () => {
    const core = read('Elephant/backend/tauri/src/lib_min.rs')
    const coreCommands = read('Elephant/backend/tauri/src/core_commands.rs')
    const compatibility = read('Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js')

    expect(fs.existsSync(absolute('Elephant/backend/tauri/src/tauri_extra_commands.rs'))).toBe(false)
    expect(core).not.toContain('tauri_search_inspect')
    expect(core).not.toContain('tauri_search_rebuild')
    expect(coreCommands).not.toContain('pub fn tauri_search_inspect')
    expect(coreCommands).not.toContain('pub fn tauri_search_rebuild')
    expect(coreCommands).not.toContain('fn build_search_index')
    expect(coreCommands).not.toContain('fn scan_markdown_notes')
    expect(coreCommands).not.toContain('fn extract_wikilinks')
    expect(coreCommands).not.toContain('SEARCH_INDEX_FILE')
    expect(compatibility).toContain("'search.query'")
    expect(compatibility).toContain("'search.status'")
    expect(compatibility).not.toContain("'search.inspect'")
    expect(compatibility).not.toContain("'search.rebuild'")
    expect(compatibility).not.toContain("'search.clear'")
    expect(compatibility).not.toContain("'search.enable'")
    expect(compatibility).not.toContain("'search.disable'")
  })

  it('keeps generic note access bounded and permission checked', () => {
    const rust = read('Elephant/backend/tauri/src/addon_note_access.rs')
    expect(rust).toContain('MAX_NOTE_BYTES')
    expect(rust).toContain('read_enabled_addon')
    expect(rust).toContain('Addon is not permitted to read')
    expect(rust).toContain('Addons cannot access notes in hidden directories')
  })
})
