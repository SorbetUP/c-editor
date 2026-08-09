import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('versioned physical package resources', () => {
  it('publishes Sites through generic asset permissions only', () => {
    const source = read('addons/official/sites/main.js')

    expect(source).toContain("const PROVIDER_RESOURCE = 'sites.provider'")
    expect(source).toContain('apiVersion: 1')
    expect(source).toContain("this.invoke('tauri_addons_assets_allow_directory'")
    expect(source).toContain('convertFileSrc(indexPath)')
    expect(source).toContain("runtime: 'tauri-asset-protocol'")
    expect(source).not.toContain('tauri_site_preview_')
    expect(source).not.toContain('localhost')
  })

  it('publishes only methods supported by the Open Models service', () => {
    const source = read('addons/official/open-models/main.js')
    const native = read('addons/official/open-models/native/src/main.rs')

    expect(source).toContain("const MODELS_RESOURCE = 'models.provider'")
    expect(source).toContain('apiVersion: 1')
    for (const method of [
      'models.status',
      'models.search',
      'models.info',
      'models.list',
      'models.download',
      'models.activate',
      'models.deactivate',
      'models.active',
      'models.delete',
      'models.chat'
    ]) {
      expect(source).toContain(`'${method}'`)
      expect(native).toContain(`"${method}"`)
    }
    expect(source).not.toContain('models.download-status')
    expect(source).not.toContain('models.cancel-download')
    expect(source).not.toContain('models.refresh-index')
  })

  it('keeps model route configuration on the AI package resource', () => {
    const openModels = read('addons/official/open-models/main.js')
    const codex = read('addons/official/codex-connection/main.js')
    const ai = read('addons/official/ai/main.js')

    expect(ai).toContain("api.resources.provide('ai.config'")
    expect(openModels).toContain("const AI_CONFIG_RESOURCE = 'ai.config'")
    expect(codex).toContain("const AI_CONFIG_RESOURCE = 'ai.config'")
    expect(openModels).not.toContain("elephantnote.api.call('ai.config")
    expect(codex).not.toContain("elephantnote.api.call('ai.config")
  })
})
