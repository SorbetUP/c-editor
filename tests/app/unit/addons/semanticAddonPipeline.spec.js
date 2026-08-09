import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')
const json = (file) => JSON.parse(read(file))

describe('physical semantic addon pipeline', () => {
  it('publishes generic inference from AI without moving Knowledge into core', () => {
    const ai = read('addons/official/ai/main.js')
    const inference = read('addons/official/ai/inference.js')
    expect(ai).toContain("api.resources.provide('ai.inference'")
    expect(inference).toContain('async embed(')
    expect(inference).toContain('async complete(')
    expect(inference).toContain("method: 'http.request'")
    expect(inference).toContain("'/embeddings'")
    expect(inference).toContain("'/chat/completions'")
  })

  it('synchronizes pending Knowledge vectors from Search through ai.inference', () => {
    const search = read('addons/official/ai-search/main.js')
    const sync = read('addons/official/ai-search/semanticEmbeddingSync.js')
    expect(search).toContain("const AI_INFERENCE_RESOURCE = 'ai.inference'")
    expect(search).toContain('synchronizeKnowledgeEmbeddings')
    expect(search).toContain('Knowledge provider rebuild failed; using local fallback')
    expect(sync).toContain('knowledge.pendingEmbeddings')
    expect(sync).toContain('inference.embed')
    expect(sync).toContain('knowledge.saveEmbeddings')
  })

  it('uses AI only to qualify communities whose membership is owned by Knowledge', () => {
    const wiki = read('addons/official/wiki/main.v2.js')
    const semantic = read('addons/official/wiki/semanticWikiProposals.js')
    expect(wiki).toContain("const AI_INFERENCE_RESOURCE = 'ai.inference'")
    expect(wiki).toContain('inference')
    expect(semantic).toContain('Do not invent notes or move notes between communities')
    expect(semantic).toContain('knowledge.semanticCommunities')
    expect(semantic).toContain('knowledge.semanticDiscover')
    expect(semantic).toContain('labeling = \'ai-inference\'')
  })

  it('keeps manifests, catalogue and protected packs on the same versions', () => {
    const catalog = json('addons/catalog.json')
    const catalogVersions = new Map(catalog.addons.map((addon) => [addon.id, addon.version]))
    for (const packPath of ['packs/base.enaddonpack', 'packs/develop-parity.enaddonpack']) {
      const pack = json(packPath)
      for (const addon of pack.addons) expect(addon.version).toBe(catalogVersions.get(addon.id))
    }
    for (const slug of ['ai', 'ai-search', 'knowledge', 'wiki']) {
      const manifest = json(`addons/official/${slug}/manifest.json`)
      expect(catalogVersions.get(manifest.id)).toBe(manifest.version)
    }
  })
})
