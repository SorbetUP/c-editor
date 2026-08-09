import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')
const exists = (relativePath) => fs.existsSync(path.join(root, relativePath))

describe('Codex physical service', () => {
  it('owns account, model and chat operations inside the addon package', () => {
    const manifest = JSON.parse(read('addons/official/codex-connection/manifest.json'))
    const entry = read('addons/official/codex-connection/main.js')
    const service = read('addons/official/codex-connection/native/src/main.rs')

    expect(manifest.version).toBe('2.0.0')
    expect(manifest.permissions.native).toBe(true)
    expect(manifest.native.runner).toBe('service')
    expect(entry).toContain("this.api.native.service.call")
    expect(entry).toContain("this.service('codex.chat'")
    expect(entry).not.toContain("invoke('tauri_rag_chat'")
    expect(service).toContain('"codex.status"')
    expect(service).toContain('"codex.models"')
    expect(service).toContain('"codex.chat"')
    expect(service).toContain('app-server')
  })

  it('removes Codex runtime wiring and binaries from Elephant core', () => {
    const core = read('Elephant/backend/tauri/src/lib_min.rs')
    const packageJson = JSON.parse(read('package.json'))

    expect(exists('Elephant/backend/tauri/src/chat_runtime.rs')).toBe(false)
    expect(exists('Elephant/backend/tauri/src/chat_runtime/codex_app_server.rs')).toBe(false)
    expect(exists('build/scripts/ensure-tauri-codex-runtime.mjs')).toBe(false)
    expect(core).not.toContain('pub mod chat_runtime;')
    expect(core).not.toContain('tauri_rag_chat')
    expect(core).not.toContain('ELEPHANTNOTE_CODEX_PATH')
    expect(packageJson.scripts).not.toHaveProperty('tauri:codex:install')
    expect(packageJson.scripts['tauri:build']).not.toContain('codex')
  })
})
