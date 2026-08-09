import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

const OPTIONAL_MARKERS = [
  'importGoogleKeep',
  'notes.autotag',
  'calendar:',
  'wiki:',
  'sync:',
  'models:',
  'ai:',
  'sitePreview:',
  'agents:',
  'plugins:',
  'tasks:',
  'rag:',
  'mcp:',
  'programs:',
  'ocr:',
  'tauri_sync_',
  'tauri_ai_config_',
  'tauri_models_',
  'tauri_ocr_',
  'tauri_site_preview_',
  'tauri_calendar_',
  'tauri_import_google_keep'
]

describe('core-only Tauri renderer bridge', () => {
  it('contains only stable core capabilities', () => {
    const source = read('Elephant/frontend/src/renderer/src/platform/tauriElephantNoteBridge.js')

    for (const marker of OPTIONAL_MARKERS) expect(source).not.toContain(marker)
    expect(source).toContain("invoke(target, 'tauri_vaults_get')")
    expect(source).toContain("invoke(target, 'tauri_search_query'")
    expect(source).toContain("invoke(target, 'tauri_features_get')")
    expect(source).toContain("invoke(target, 'tauri_atomic_features_list')")
    expect(source).toContain("bridge: 'elephantnote-tauri-core'")
    expect(source).toContain('throw new Error(`Unsupported Elephant core API action:')
    expect(source).not.toContain('return { ok: true, queued: true }')
    expect(source).not.toContain("runtime: 'tauri-rust-addon-bridge'")
  })

  it('keeps optional actions out of shared contracts and local fallbacks', () => {
    const contracts = read('Elephant/shared/apiContracts.js')
    const compatibility = read('Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js')
    const clients = read('Elephant/frontend/app/services/elephantnoteClient/domainClients.js')

    for (const action of [
      'wiki.list',
      'sync.run',
      'ai.config.get',
      'models.list',
      'ocr.extract',
      'sites.previewFolder',
      'calendar.list',
      'import.googleKeep',
      'rag.chat'
    ]) {
      expect(contracts).not.toContain(action)
      expect(compatibility).not.toContain(action)
      expect(clients).not.toContain(action)
    }
  })

  it('leaves optional implementations in their physical packages', () => {
    expect(read('addons/official/wiki/main.v2.js')).toContain("const PROVIDER_RESOURCE = 'wiki.provider'")
    expect(read('addons/official/ai-search/main.js')).toContain("const PROVIDER_RESOURCE = 'search.provider'")
    expect(read('addons/official/sync/main.service.js')).toContain("const SERVICE_RESOURCE = 'sync.native-service'")
    expect(read('addons/official/ai/main.js')).toContain("api.resources.provide('ai.config'")
    expect(read('addons/official/google-keep-import/main.js')).toContain("const PROVIDER_RESOURCE = 'import.google-keep'")
  })
})
