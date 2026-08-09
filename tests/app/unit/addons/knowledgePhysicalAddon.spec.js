import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('physical Knowledge addon', () => {
  it('owns one reusable Rust engine for desktop sidecars and mobile embedding', () => {
    const manifest = JSON.parse(read('addons/official/knowledge/manifest.json'))
    const entry = read('addons/official/knowledge/main.js')
    const sidecar = read('addons/official/knowledge/native/src/main.rs')
    const service = read('addons/official/knowledge/native/knowledge-core/src/service.rs')
    const exports = read('addons/official/knowledge/native/knowledge-core/src/lib.rs')

    expect(manifest.version).toBe('1.2.0')
    expect(manifest.native.runner).toBe('service')
    expect(manifest.native.mobile.android).toEqual({
      supported: true,
      runner: 'embedded-rust',
      host: 'elephant-knowledge-v1'
    })
    expect(manifest.native.mobile.ios).toEqual(manifest.native.mobile.android)
    expect(entry).toContain('api.resources.provide(RESOURCE_ID')
    expect(entry).toContain('apiVersion: 1')
    expect(entry).toContain('owner: ADDON_ID')
    expect(sidecar).toContain('KnowledgeService::open')
    expect(sidecar).toContain('service.call(method, params)')
    expect(exports).toContain('pub use service::KnowledgeService')
    expect(service).toContain('rebuild_vault')
    expect(service).toContain('graph_projection')
    expect(service).toContain('knowledge.wiki.render')
    expect(service).toContain('knowledge.embedding.pending')
  })

  it('routes mobile service calls through the generic addon service host', () => {
    const cargo = read('Elephant/backend/tauri/Cargo.toml')
    const shell = read('Elephant/backend/tauri/src/lib_min.rs')
    const services = read('Elephant/backend/tauri/src/addon_services.rs')
    const embedded = read('Elephant/backend/tauri/src/embedded_addon_services.rs')

    expect(cargo).toContain("cfg(any(target_os = \"android\", target_os = \"ios\"))")
    expect(cargo).toContain('elephantnote-knowledge-core')
    expect(shell).toContain('#[cfg(mobile)]\nmod embedded_addon_services;')
    expect(services).toContain('embedded_mobile_descriptor')
    expect(services).toContain('Only an official addon may use a statically embedded service host')
    expect(services).toContain('exchange_embedded')
    expect(embedded).toContain('KnowledgeService::open(vault_dir)?.call(method, params)')
  })

  it('does not restore the historical knowledge backend to Tauri core', () => {
    const tauri = fs.readdirSync(path.join(root, 'Elephant/backend/tauri/src'))
    expect(tauri).not.toContain('knowledge.rs')
    expect(tauri).not.toContain('knowledge_wikis.rs')
    expect(tauri).not.toContain('knowledge_graph.rs')
  })
})
