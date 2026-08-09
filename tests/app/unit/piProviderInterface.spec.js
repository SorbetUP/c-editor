import { describe, expect, it, vi } from 'vitest'
import {
  PI_DEFAULT_BASE_URL,
  listPiCodexModels,
  listPiModels,
  normalizePiCodexModels,
  normalizePiModels,
  piModelsUrl
} from '../../../Elephant/frontend/src/renderer/src/platform/piProviderInterface.js'

const jsonResponse = (body, status = 200) => ({
  ok: status >= 200 && status < 300,
  status,
  statusText: status === 200 ? 'OK' : 'Error',
  json: vi.fn().mockResolvedValue(body)
})

describe('PI provider interface', () => {
  it('builds PI model endpoints like CLIProxyAPI', () => {
    expect(piModelsUrl()).toBe(`${PI_DEFAULT_BASE_URL}/models`)
    expect(piModelsUrl({ codexClient: true, clientVersion: 'elephantnote-test' })).toBe(`${PI_DEFAULT_BASE_URL}/models?client_version=elephantnote-test`)
  })

  it('normalizes OpenAI-compatible /v1/models responses', () => {
    const models = normalizePiModels({ object: 'list', data: [{ id: 'gpt-5.5', object: 'model', owned_by: 'openai' }] })
    expect(models).toEqual([expect.objectContaining({ id: 'gpt-5.5', provider: 'pi', source: 'openai-compatible', ownedBy: 'openai' })])
  })

  it('normalizes PI client-version model responses', () => {
    const models = normalizePiCodexModels({ models: [{ slug: 'gpt-5.5', display_name: 'GPT-5.5', context_window: 128000 }] })
    expect(models).toEqual([expect.objectContaining({ id: 'gpt-5.5', name: 'GPT-5.5', provider: 'pi', source: 'codex-client', contextLength: 128000 })])
  })

  it('lists normal PI models through the local proxy endpoint', async() => {
    const fetchImpl = vi.fn().mockResolvedValue(jsonResponse({ object: 'list', data: [{ id: 'gpt-5.5', object: 'model' }] }))
    const result = await listPiModels({ fetchImpl })
    expect(fetchImpl).toHaveBeenCalledWith(`${PI_DEFAULT_BASE_URL}/models`, expect.objectContaining({ method: 'GET' }))
    expect(result.ok).toBe(true)
    expect(result.models.map((model) => model.id)).toEqual(['gpt-5.5'])
  })

  it('lists PI client-version models through the local proxy endpoint', async() => {
    const fetchImpl = vi.fn().mockResolvedValue(jsonResponse({ models: [{ slug: 'gpt-5.5', display_name: 'GPT-5.5' }] }))
    const result = await listPiCodexModels({ fetchImpl, clientVersion: 'elephantnote-test' })
    expect(fetchImpl).toHaveBeenCalledWith(`${PI_DEFAULT_BASE_URL}/models?client_version=elephantnote-test`, expect.objectContaining({ method: 'GET' }))
    expect(result.ok).toBe(true)
    expect(result.codexClient).toBe(true)
    expect(result.models.map((model) => model.id)).toEqual(['gpt-5.5'])
  })
})
