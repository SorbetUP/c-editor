import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('AI physical package navigation', () => {
  it('has one physical navigation owner and package-owned child slots', () => {
    const parent = read('addons/official/ai/main.js')
    const chat = read('addons/official/ai-chat/main.js')
    const search = read('addons/official/ai-search/main.js')
    const ocr = read('addons/official/ai-ocr/main.js')

    expect(parent).toContain('const PAGE_DEFINITIONS')
    expect(parent).toContain("slot: 'ai.chat'")
    expect(parent).toContain("slot: 'ai.search'")
    expect(parent).toContain("slot: 'ai.ocr'")
    expect(parent).toContain("setAttribute('data-elephant-addon-settings-slot', active.slot)")
    expect(chat).toContain("slot: 'ai.chat'")
    expect(search).toContain("slot: 'ai.search'")
    expect(ocr).toContain("slot: 'ai.ocr'")
  })

  it('exposes local and subscription models only through physical provider contributions', () => {
    const routeSettings = read('Elephant/frontend/app/components/settings/AiProviderSettingsPanel.vue')
    const openModels = read('addons/official/open-models/main.js')
    const codex = read('addons/official/codex-connection/main.js')

    expect(routeSettings).toContain("getContributions('ai.providers')")
    expect(routeSettings).not.toContain('<option v-if="form.localAi.enabled" value="app-local">')
    expect(routeSettings).not.toContain('Unavailable addon provider')
    expect(openModels).toContain('providerId: PROVIDER_ID')
    expect(openModels).toContain("capabilities: ['chat']")
    expect(openModels).toContain('api.native.service.start()')
    expect(codex).toContain("capabilities: ['chat']")
    expect(codex).toContain('api.native.service.start()')
  })

  it('keeps every migrated AI implementation outside the builtin catalogue and core backend', () => {
    const builtinIndex = read('Elephant/frontend/src/renderer/src/addons/builtin/index.js')
    const core = read('Elephant/backend/tauri/src/lib_min.rs')
    const physicalIds = [
      'elephant.ai',
      'elephant.ai-chat',
      'elephant.ai-search',
      'elephant.ai-ocr',
      'elephant.wiki',
      'elephant.graph',
      'elephant.open-models',
      'elephant.codex-connection'
    ]

    for (const id of physicalIds) expect(builtinIndex).not.toContain(`id: '${id}'`)
    expect(core).not.toContain('pub mod ocr;')
    expect(core).not.toContain('tauri_ocr_')
    expect(core).not.toContain('pub mod chat_runtime;')
    expect(core).not.toContain('tauri_rag_chat')
    expect(core).not.toContain('pub mod model_library;')
    expect(read('addons/official/ai-ocr/manifest.json')).toContain('"native": true')
    expect(read('addons/official/ai-ocr/main.js')).toContain('this.api.native.call')
  })
})

describe('workspace vertical border alignment', () => {
  it('uses the same rail width in flex layout and title strip divider math', () => {
    const layoutFixes = read('Elephant/frontend/app/styles/runtime-layout-fixes.css')
    expect(layoutFixes).toContain('width: 56px !important')
    expect(layoutFixes).toContain('flex: 0 0 56px !important')
    expect(layoutFixes).toContain('left: calc(56px + var(--en-sidebar-width)) !important')
    expect(layoutFixes).toContain('width: 1px !important')
  })
})
