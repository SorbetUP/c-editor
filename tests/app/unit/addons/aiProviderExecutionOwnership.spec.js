import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const absolute = (relativePath) => path.join(root, relativePath)
const read = (relativePath) => fs.readFileSync(absolute(relativePath), 'utf8')

describe('AI provider execution ownership', () => {
  it('keeps the chat addon as an orchestrator over installed provider contributions', () => {
    const chat = read('addons/official/ai-chat/main.v2.js')
    const base = read('addons/official/ai-chat/main.js')

    expect(base).toContain("getContributions?.('ai.providers')")
    expect(chat).toContain('async runProvider(')
    expect(base).toContain('await option.provider.chat({')
    expect(chat).not.toContain("this.call('rag.chat'")
  })

  it('exposes service-backed chat functions from Open Models and Codex packages', () => {
    const openModels = read('addons/official/open-models/main.js')
    const codex = read('addons/official/codex-connection/main.js')

    expect(openModels).toContain("chat({ messages = [], model = '', route = {}, config = {} } = {})")
    expect(openModels).toContain("return this.service('models.chat'")
    expect(openModels).toContain('chat: (request) => this.chat(request)')
    expect(codex).toContain("async chat({ messages = [], model = '', route = {} } = {})")
    expect(codex).toContain("this.service('codex.chat'")
    expect(codex).toContain('chat: (request) => this.chat(request)')
  })

  it('offers only actual installed or configured providers in chat settings', () => {
    const chat = read('addons/official/ai-chat/main.js')

    expect(chat).toContain("node(documentRef, 'option', '', 'Désactivé')")
    expect(chat).toContain('for (const option of options)')
    expect(chat).toContain('providerOptions(config)')
    expect(chat).not.toContain('Unavailable addon provider')
    expect(chat).not.toContain('SmolLM2')
  })

  it('keeps provider configuration and runtime tests physically absent from the core Tauri shell', () => {
    const core = read('Elephant/backend/tauri/src/lib_min.rs')
    const coreCommands = read('Elephant/backend/tauri/src/core_commands.rs')
    const compatibility = read('Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js')

    expect(fs.existsSync(absolute('Elephant/backend/tauri/src/tauri_extra_commands.rs'))).toBe(false)
    expect(core).not.toContain('tauri_ai_config_get')
    expect(core).not.toContain('tauri_ai_config_set')
    expect(core).not.toContain('tauri_ai_config_test')
    expect(coreCommands).not.toContain('pub fn tauri_ai_config_get')
    expect(coreCommands).not.toContain('pub fn tauri_ai_config_set')
    expect(coreCommands).not.toContain('pub fn tauri_ai_config_test')
    expect(coreCommands).not.toContain('pub fn tauri_models_get_selection')
    expect(coreCommands).not.toContain('pub fn tauri_models_set_selection')
    expect(coreCommands).not.toContain('test_codex_cli')
    expect(coreCommands).not.toContain('test_tcp_endpoint')
    expect(coreCommands).not.toContain('PROVIDER_CONFIG_CATEGORY')
    expect(compatibility).not.toContain("'ai.config.get'")
    expect(compatibility).not.toContain("'ai.config.set'")
    expect(compatibility).not.toContain("'ai.config.test'")
    expect(compatibility).not.toContain("'models.selection.get'")
    expect(compatibility).not.toContain("'models.selection.set'")
    expect(compatibility).not.toContain("'models.download'")
  })
})
