import { describe, expect, it } from 'vitest'

import {
  applyCatalogFilters,
  applyRoleChoice,
  buildStateBadge,
  dedupeModelsById,
  formatBytes,
  formatCompactCount,
  formatRelativeDate,
  getModelCapabilities,
  getModelFormat,
  getModelQuantization,
  getModelSource,
  getModelUpdatedDate,
  isAssignedToRole,
  isLocalModel,
  isRemoteModel,
  normalizeSelection,
  resolveModelId
} from '../../../Elephant/frontend/app/components/views/modelsViewHelpers.js'

describe('models view Electron/Tauri parity helpers', () => {
  it('formats stable values for display', () => {
    expect(formatBytes(1024)).toBe('1 KB')
    expect(formatCompactCount(1200)).toBe('1.2K')
    expect(formatRelativeDate('bad-date')).toBe('')
    expect(getModelUpdatedDate({ updatedAt: 'bad-date' })).toBe('')
  })

  it('detects local and remote models consistently', () => {
    expect(isLocalModel({ path: '/tmp/model.gguf' })).toBe(true)
    expect(isRemoteModel({ repoId: 'org/model' })).toBe(true)
    expect(getModelSource({ path: '/tmp/model.gguf' })).toBe('Local')
    expect(getModelSource({ repoId: 'org/model' })).toBe('Hugging Face')
  })

  it('detects model formats capabilities and quantization', () => {
    expect(getModelFormat({ fileName: 'model.Q4_K_M.gguf' })).toBe('GGUF')
    expect(getModelQuantization({ fileName: 'model.Q4_K_M.gguf' })).toBe('Q4_K_M')
    expect(getModelCapabilities({ repoId: 'sentence-transformers/all-MiniLM', pipelineTag: 'feature-extraction' })).toContain('Embedding')
    expect(getModelCapabilities({ task: 'ocr' })).toContain('OCR')
  })

  it('dedupes local models over remote copies', () => {
    const models = dedupeModelsById([
      { repoId: 'org/a', downloads: 10 },
      { repoId: 'org/a', path: '/models/a.gguf', downloads: 1 }
    ])
    expect(models).toHaveLength(1)
    expect(models[0].path).toBe('/models/a.gguf')
  })

  it('assigns and clears roles deterministically', () => {
    const model = { id: 'chat-model' }
    const selected = applyRoleChoice(normalizeSelection({}), 'chat', model, 'chat')
    expect(isAssignedToRole(model, 'chat', selected)).toBe(true)
    expect(resolveModelId(model)).toBe('chat-model')
    expect(buildStateBadge(model, selected).tone).toBe('active')
  })

  it('filters catalog by source and format', () => {
    const models = applyCatalogFilters({
      models: [
        { id: 'local', path: '/models/local.gguf', fileName: 'local.gguf' },
        { id: 'remote', repoId: 'org/remote', fileName: 'remote.gguf' }
      ],
      source: 'remote',
      format: 'gguf'
    })
    expect(models).toHaveLength(1)
    expect(models[0].id).toBe('remote')
  })
})
