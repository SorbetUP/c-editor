import { describe, expect, it, vi } from 'vitest'
import {
  installPiProviderBridge,
  listPiCodexModels,
  listPiModels,
  piModelsUrl
} from '@/platform/piProviderInterface'

describe('PI provider interface', () => {
  it('builds standard and codex-client model URLs from a configured base URL', () => {
    expect(piModelsUrl({ baseUrl: 'http://127.0.0.1:8317/v1/' })).toBe('http://127.0.0.1:8317/v1/models')
    expect(piModelsUrl({ baseUrl: 'http://127.0.0.1:8317/v1/', codexClient: true, clientVersion: 'test client' })).toBe(
      'http://127.0.0.1:8317/v1/models?client_version=test%20client'
    )
  })

  it('normalizes OpenAI-compatible PI model responses', async() => {
    const fetchImpl = vi.fn(async() => ({
      ok: true,
      status: 200,
      json: async() => ({
        data: [
          { id: 'local-llama', object: 'model', owned_by: 'pi' },
          { id: '' }
        ]
      })
    }))

    const result = await listPiModels({ fetchImpl })

    expect(result.ok).toBe(true)
    expect(result.count).toBe(1)
    expect(result.models[0]).toMatchObject({ id: 'local-llama', provider: 'pi', source: 'openai-compatible' })
    expect(fetchImpl).toHaveBeenCalledWith('http://127.0.0.1:8317/v1/models', expect.any(Object))
  })

  it('normalizes codex-client PI model responses', async() => {
    const fetchImpl = vi.fn(async() => ({
      ok: true,
      status: 200,
      json: async() => ({
        models: [
          { slug: 'codex-local', display_name: 'Codex Local', context_window: 8192, max_context_window: 16384 },
          { slug: '' }
        ]
      })
    }))

    const result = await listPiCodexModels({ fetchImpl, clientVersion: 'elephantnote-test' })

    expect(result.ok).toBe(true)
    expect(result.codexClient).toBe(true)
    expect(result.count).toBe(1)
    expect(result.models[0]).toMatchObject({
      id: 'codex-local',
      name: 'Codex Local',
      provider: 'pi',
      source: 'codex-client',
      contextLength: 8192,
      maxContextLength: 16384
    })
    expect(fetchImpl).toHaveBeenCalledWith(
      'http://127.0.0.1:8317/v1/models?client_version=elephantnote-test',
      expect.any(Object)
    )
  })

  it('returns a structured error instead of throwing on invalid JSON responses', async() => {
    const fetchImpl = vi.fn(async() => ({
      ok: true,
      status: 200,
      json: async() => {
        throw new SyntaxError('Unexpected token < in JSON')
      }
    }))

    await expect(listPiModels({ fetchImpl })).resolves.toMatchObject({
      ok: false,
      provider: 'pi',
      status: 200,
      error: 'invalid_json',
      models: [],
      count: 0,
      codexClient: false
    })
  })

  it('installs PI actions without replacing existing bridge API calls', async() => {
    const previousCall = vi.fn(async(action, payload) => ({ ok: true, data: { action, payload } }))
    const target = {
      fetch: vi.fn(async() => ({
        ok: true,
        status: 200,
        json: async() => ({ data: [{ id: 'pi-model' }] })
      })),
      elephantnote: {
        api: {
          describe: async() => ({ actions: ['vaults.get'] }),
          call: previousCall
        }
      }
    }

    expect(installPiProviderBridge(target)).toBe(true)
    await expect(target.elephantnote.api.describe()).resolves.toMatchObject({
      actions: expect.arrayContaining(['vaults.get', 'pi.models.list', 'pi.codex.models.list', 'codex.models.list'])
    })
    await expect(target.elephantnote.api.call('pi.models.list')).resolves.toMatchObject({
      ok: true,
      data: { count: 1, models: [{ id: 'pi-model', name: 'pi-model', provider: 'pi', source: 'openai-compatible', object: 'model', ownedBy: null, raw: { id: 'pi-model' } }] }
    })
    await expect(target.elephantnote.api.call('vaults.get', { id: 'v1' })).resolves.toEqual({
      ok: true,
      data: { action: 'vaults.get', payload: { id: 'v1' } }
    })
  })
})
