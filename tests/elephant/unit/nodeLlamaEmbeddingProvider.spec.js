import { describe, expect, it, vi } from 'vitest'

const { createNodeLlamaCppEmbeddingProvider } = await import(
  'main_renderer/elephantnote/search/embeddingProvider'
)

describe('createNodeLlamaCppEmbeddingProvider', () => {
  it('falls back cleanly when the selected embedding model is missing', async() => {
    const logger = {
      info: vi.fn(),
      warn: vi.fn(),
      error: vi.fn()
    }
    const provider = createNodeLlamaCppEmbeddingProvider({
      getSelectedModel: () => 'missing-embed-model',
      resolveLocalModel: async() => {
        throw new Error('No model file found at "/tmp/missing-embed-model.gguf"')
      },
      resolveConfiguredLocalModel: (config) => ({
        id: config.model,
        name: config.model,
        provider: 'node-llama-cpp',
        model: config.model,
        uri: config.model,
        pull: config.model
      }),
      runtime: {
        nodeLlamaCppRuntime: {
          embedText: vi.fn()
        }
      },
      logger
    })

    const first = await provider.embedText('local note text')
    const second = await provider.embedText('another note text')
    const repeated = await provider.embedText('local note text')

    expect(Array.isArray(first)).toBe(true)
    expect(first.length).toBeGreaterThan(0)
    expect(second).toHaveLength(first.length)
    expect(repeated).toEqual(first)

    expect(logger.warn).toHaveBeenCalledTimes(1)
    expect(logger.warn.mock.calls[0][0]).toContain('embedding model unavailable')
    expect(logger.info).not.toHaveBeenCalled()
  })

  it('uses the local llama runtime when the model resolves', async() => {
    const embedText = vi.fn().mockResolvedValue([1, 0, 0])
    const logger = {
      info: vi.fn(),
      warn: vi.fn(),
      error: vi.fn()
    }
    const provider = createNodeLlamaCppEmbeddingProvider({
      getSelectedModel: () => 'smollm2-node-llama-cpp',
      resolveLocalModel: async(model) => ({
        ...model,
        path: '/models/smollm2-node-llama-cpp.gguf',
        modelPath: '/models/smollm2-node-llama-cpp.gguf'
      }),
      resolveConfiguredLocalModel: (config) => ({
        id: config.model,
        name: config.model,
        provider: 'node-llama-cpp',
        model: config.model,
        uri: config.model,
        pull: config.model
      }),
      runtime: {
        nodeLlamaCppRuntime: {
          embedText
        }
      },
      logger
    })

    await expect(provider.embedText('hello world')).resolves.toEqual([1, 0, 0])
    expect(embedText).toHaveBeenCalledWith({
      model: expect.objectContaining({
        modelPath: '/models/smollm2-node-llama-cpp.gguf'
      }),
      text: 'hello world'
    })
  })
})
