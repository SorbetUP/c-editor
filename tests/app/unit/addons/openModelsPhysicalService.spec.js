import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')
const exists = (relativePath) => fs.existsSync(path.join(root, relativePath))

describe('Open Models physical service', () => {
  it('owns native model lifecycle and chat inside the addon package', () => {
    const manifest = JSON.parse(read('addons/official/open-models/manifest.json'))
    const entry = read('addons/official/open-models/main.js')
    const service = read('addons/official/open-models/native/src/main.rs')

    expect(manifest.version).toBe('2.0.0')
    expect(manifest.permissions.native).toBe(true)
    expect(manifest.native.runner).toBe('service')
    expect(manifest.native.protocol).toBe('elephant-addon-service-v1')
    expect(entry).toContain("this.api.native.service.call")
    expect(entry).toContain("this.service('models.chat'")
    expect(entry).not.toContain("this.call('models.list'")
    expect(entry).not.toContain("this.invoke('tauri_rag_chat'")
    expect(service).toContain('"models.download"')
    expect(service).toContain('"models.chat"')
    expect(service).toContain('ELEPHANT_ADDON_DATA_DIR')
  })

  it('removes the legacy model implementation from Elephant core', () => {
    const core = read('Elephant/backend/tauri/src/lib_min.rs')

    expect(exists('Elephant/backend/tauri/src/model_domain.rs')).toBe(false)
    expect(exists('Elephant/backend/tauri/src/model_library.rs')).toBe(false)
    expect(exists('Elephant/backend/tauri/src/local_llama_runtime.rs')).toBe(false)
    expect(core).not.toContain('pub mod model_domain;')
    expect(core).not.toContain('pub mod model_library;')
    expect(core).not.toContain('pub mod local_llama_runtime;')
    expect(core).not.toContain('tauri_models_download')
    expect(core).not.toContain('tauri_models_activate')
  })
})
