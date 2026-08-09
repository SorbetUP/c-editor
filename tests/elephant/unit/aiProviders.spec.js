import { describe, expect, it } from 'vitest'
import {
  ELEPHANTNOTE_AI_PRESETS,
  createAiRequestBody,
  extractAiResponseText,
  normalizeAiConfig,
  normalizeAiEndpoint,
  resolveAiEndpoint
} from 'common/elephantnote/aiProviders'

describe('ElephantNote AI providers', () => {
  it('accepts IP and port endpoints without a scheme', () => {
    expect(normalizeAiEndpoint('192.168.1.25:11434/api/chat')).toBe('http://192.168.1.25:11434/api/chat')
    expect(normalizeAiEndpoint('localhost:1234/v1/chat/completions')).toBe('http://localhost:1234/v1/chat/completions')
  })

  it('normalizes Ollama base URLs to the chat API route', () => {
    expect(resolveAiEndpoint({ transport: 'ollama', endpoint: 'http://127.0.0.1:11434' })).toBe('http://127.0.0.1:11434/api/chat')
    expect(resolveAiEndpoint({ transport: 'ollama', endpoint: '127.0.0.1:11434/api/chat' })).toBe('http://127.0.0.1:11434/api/chat')
  })

  it('starts without an implicit local model provider', () => {
    expect(normalizeAiConfig({})).toMatchObject({
      preset: 'custom',
      provider: undefined,
      transport: 'openai-compatible',
      endpoint: '',
      model: '',
      localAi: {
        enabled: false,
        showModelLibraryInSidebar: false,
        allowHuggingFaceDownloads: false,
        allowLocalRuntimeAutostart: false
      }
    })
    expect(resolveAiEndpoint({ transport: 'tauri-rust', endpoint: 'tauri-rust://local' })).toBe('tauri-rust://local')
  })

  it('allows the native local preset only when local AI is explicitly enabled', () => {
    expect(normalizeAiConfig({ preset: 'nodeLlamaCpp' })).toMatchObject({
      preset: 'custom',
      provider: 'disabled',
      transport: 'openai-compatible',
      endpoint: '',
      model: ''
    })
    expect(normalizeAiConfig({
      preset: 'nodeLlamaCpp',
      localAi: { enabled: true }
    })).toMatchObject({
      preset: 'tauriRustLocal',
      transport: 'tauri-rust',
      endpoint: ELEPHANTNOTE_AI_PRESETS.tauriRustLocal.endpoint,
      localAi: { enabled: true }
    })
  })

  it('normalizes remote presets without a global enabled flag', () => {
    expect(normalizeAiConfig({ preset: 'mlx' })).toMatchObject({
      preset: 'mlx',
      transport: 'openai-compatible'
    })
    expect(normalizeAiConfig({ preset: 'openrouter' })).toMatchObject({
      preset: 'openrouter',
      transport: 'openai-compatible'
    })
    expect(normalizeAiConfig({ preset: 'codex' })).toMatchObject({
      preset: 'codex',
      transport: 'openai-compatible'
    })
    expect(normalizeAiConfig({ enabled: false })).not.toHaveProperty('enabled')
  })

  it('creates provider request bodies and extracts common response shapes', () => {
    const messages = [{ role: 'user', content: 'Hello' }]
    expect(createAiRequestBody({ transport: 'tauri-rust', model: 'local.gguf', messages })).toEqual({
      model: 'local.gguf',
      messages,
      stream: false
    })
    expect(createAiRequestBody({ transport: 'ollama', model: 'llama3.2', messages })).toEqual({
      model: 'llama3.2',
      messages,
      stream: false
    })
    expect(extractAiResponseText({ message: { content: 'Ollama response' } })).toBe('Ollama response')
    expect(extractAiResponseText({ choices: [{ message: { content: 'OpenAI response' } }] })).toBe('OpenAI response')
  })
})
