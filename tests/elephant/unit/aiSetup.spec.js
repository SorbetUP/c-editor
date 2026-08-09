import { describe, expect, it } from 'vitest'
import { ATOMIC_MODEL_CATALOG } from 'common/elephantnote/atomicWorkspace'
import {
  createSelectionPatchForModel,
  getModelRuntimeName,
  getRecommendedSetupModels,
  isRunnableSetupModel,
  isSetupModelInstalled
} from 'common/elephantnote/aiSetup'

describe('AI setup workflow helpers', () => {
  it('chooses runnable Tauri Rust defaults for embeddings and chat', () => {
    const recommended = getRecommendedSetupModels(ATOMIC_MODEL_CATALOG, 'tauri-rust')

    expect(recommended.embedding).toMatchObject({
      id: 'smollm2-node-llama-cpp',
      provider: 'tauri-rust',
      task: 'embedding'
    })
    expect(recommended.chat).toMatchObject({
      id: 'smollm2-node-llama-cpp-chat',
      provider: 'tauri-rust',
      task: 'chat-completion'
    })
    expect(recommended.ocr).toMatchObject({
      id: 'local-tesseract-ocr',
      provider: 'local-ocr',
      task: 'ocr'
    })
    expect(isRunnableSetupModel(recommended.embedding)).toBe(true)
    expect(isRunnableSetupModel(recommended.chat)).toBe(true)
    expect(isRunnableSetupModel(recommended.ocr)).toBe(true)
  })

  it('stores model slots with stable catalog ids, not runtime ids', () => {
    const model = ATOMIC_MODEL_CATALOG.find((item) => item.id === 'smollm2-node-llama-cpp')

    expect(getModelRuntimeName(model)).toBe('hf:bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M')
    expect(createSelectionPatchForModel(model)).toEqual({
      embedding: 'smollm2-node-llama-cpp'
    })
  })

  it('recognizes installed local models by runtime name and stable id', () => {
    const embedding = ATOMIC_MODEL_CATALOG.find((item) => item.id === 'smollm2-node-llama-cpp')
    const chat = ATOMIC_MODEL_CATALOG.find((item) => item.id === 'smollm2-node-llama-cpp-chat')

    expect(isSetupModelInstalled(embedding, [
      { id: 'hf_bartowski_SmolLM2-135M-Instruct.Q4_K_M.gguf', model: 'hf:bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M' }
    ])).toBe(true)
    expect(isSetupModelInstalled(chat, [
      { id: 'smollm2-node-llama-cpp-chat', provider: 'tauri-rust' }
    ])).toBe(true)
  })
})
