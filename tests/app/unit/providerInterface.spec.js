import { describe, expect, it, vi } from 'vitest'
import {
  OPENROUTER_MODELS_URL,
  buildOpenRouterHeaders,
  installCodexProviderBridge,
  listOpenRouterModels,
  normalizeOpenRouterModels,
  testOpenRouterChatAccess
} from '../../../Elephant/frontend/src/renderer/src/platform/providerInterface.js'

const jsonResponse = (body, status = 200) => ({
  ok: status >= 200 && status < 300,
  status,
  statusText: status === 200 ? 'OK' : 'Error',
  json: vi.fn().mockResolvedValue(body)
})

describe('Codex Provider Interface', () => {
  it('normalizes OpenRouter model records', () => {
    const models = normalizeOpenRouterModels({
      data: [{ id: 'openai/gpt-5.2', name: 'GPT 5.2', context_length: 128000, supported_parameters: ['tools'] }]
    })
    expect(models).toEqual([
      expect.objectContaining({ id: 'openai/gpt-5.2', name: 'GPT 5.2', provider: 'openrouter', contextLength: 128000 })
    ])
  })

  it('lists OpenRouter models without an API key when the public endpoint allows it', async() => {
    const fetchImpl = vi.fn().mockResolvedValue(jsonResponse({ data: [{ id: 'anthropic/claude-sonnet-4.5' }] }))
    const result = await listOpenRouterModels({ fetchImpl })

    expect(fetchImpl).toHaveBeenCalledWith(OPENROUTER_MODELS_URL, expect.objectContaining({ method: 'GET' }))
    expect(fetchImpl.mock.calls[0][1].headers.Authorization).toBeUndefined()
    expect(result.ok).toBe(true)
    expect(result.authenticated).toBe(false)
    expect(result.models.map((model) => model.id)).toEqual(['anthropic/claude-sonnet-4.5'])
  })

  it('keeps Authorization only when an API key is provided', () => {
    expect(buildOpenRouterHeaders({}).Authorization).toBeUndefined()
    expect(buildOpenRouterHeaders({ apiKey: 'sk-or-test' }).Authorization).toBe('Bearer sk-or-test')
  })

  it('returns a clean authentication_required result for protected OpenRouter endpoints', async() => {
    const fetchImpl = vi.fn().mockResolvedValue(jsonResponse({ error: { message: 'No auth credentials found' } }, 401))
    const result = await listOpenRouterModels({ fetchImpl })

    expect(result.ok).toBe(false)
    expect(result.status).toBe(401)
    expect(result.error).toBe('authentication_required')
    expect(result.message).toContain('No auth credentials found')
  })

  it('does not call chat/completions without an API key', async() => {
    const fetchImpl = vi.fn()
    const result = await testOpenRouterChatAccess({ fetchImpl })

    expect(fetchImpl).not.toHaveBeenCalled()
    expect(result.ok).toBe(false)
    expect(result.error).toBe('authentication_required')
  })

  it('installs Codex API actions on the Elephant bridge', async() => {
    const fetchImpl = vi.fn().mockResolvedValue(jsonResponse({ data: [{ id: 'openai/gpt-5.2' }] }))
    const target = {
      fetch: fetchImpl,
      elephantnote: {
        ai: {},
        api: {
          describe: vi.fn().mockResolvedValue({ runtime: 'tauri', actions: ['api.describe'] }),
          call: vi.fn().mockResolvedValue({ ok: true, data: 'fallback' })
        }
      }
    }

    expect(installCodexProviderBridge(target)).toBe(true)
    const description = await target.elephantnote.api.describe()
    expect(description.actions).toContain('codex.models.list')
    const result = await target.elephantnote.api.call('codex.models.list', {})
    expect(result.ok).toBe(true)
    expect(result.data.models[0].id).toBe('openai/gpt-5.2')
  })
})
