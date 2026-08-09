import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('Knowledge provider consumers', () => {
  it('delegates Search and Chat retrieval while retaining local fallbacks', () => {
    const search = read('addons/official/ai-search/main.js')
    const chat = read('addons/official/ai-chat/main.v2.js')
    const chatBase = read('addons/official/ai-chat/main.js')
    expect(search).toContain("const KNOWLEDGE_RESOURCE = 'knowledge.provider'")
    expect(search).toContain('Knowledge provider search failed; using local fallback')
    expect(search).toContain('this.index.documents')
    expect(chatBase).toContain("const KNOWLEDGE_RESOURCE = 'knowledge.provider'")
    expect(chat).toContain('retrieveContext(')
  })

  it('uses the package graph projection and merges provider Wiki drafts', () => {
    const graph = read('addons/official/graph/main.js')
    const wiki = read('addons/official/wiki/main.js')
    expect(graph).toContain("const KNOWLEDGE_RESOURCE = 'knowledge.provider'")
    expect(graph).toContain("engine: 'knowledge-provider'")
    expect(graph).toContain('using local fallback')
    expect(wiki).toContain('knowledge.listWikis')
    expect(wiki).toContain('knowledge.acceptWiki')
    expect(wiki).toContain('knowledge.rejectWiki')
    expect(wiki).toContain('using local records')
  })
})
