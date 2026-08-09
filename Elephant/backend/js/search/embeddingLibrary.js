import log from 'electron-log'

const DEFAULT_EMBEDDING_DIMENSIONS = 64

export const createDeterministicEmbedding = (text = '', dimensions = DEFAULT_EMBEDDING_DIMENSIONS) => {
  const vector = Array.from({ length: dimensions }, () => 0)
  const source = String(text || '')
  for (let index = 0; index < source.length; index += 1) {
    const code = source.charCodeAt(index)
    vector[index % dimensions] += (code % 17) + 1
  }
  const magnitude = Math.sqrt(vector.reduce((sum, value) => sum + value * value, 0)) || 1
  return vector.map((value) => Number((value / magnitude).toFixed(6)))
}

export const createNodeLlamaCppEmbeddingLibrary = ({
  getSelectedModel,
  resolveLocalModel,
  resolveConfiguredLocalModel,
  runtime,
  logger = log
} = {}) => {
  if (typeof getSelectedModel !== 'function') throw new Error('getSelectedModel is required.')
  if (typeof resolveLocalModel !== 'function') throw new Error('resolveLocalModel is required.')
  if (typeof resolveConfiguredLocalModel !== 'function') {
    throw new Error('resolveConfiguredLocalModel is required.')
  }
  if (!runtime?.nodeLlamaCppRuntime?.embedText) {
    throw new Error('runtime.nodeLlamaCppRuntime.embedText is required.')
  }

  const resolvedModels = new Map()
  const warnedMissingModel = new Set()
  const warnedInferenceFallback = new Set()

  const resolveEmbeddingModel = async () => {
    const selectedModel = getSelectedModel('embedding') || 'smollm2-node-llama-cpp'
    if (resolvedModels.has(selectedModel)) return resolvedModels.get(selectedModel)

    const configuredModel = resolveConfiguredLocalModel({
      purpose: 'embedding',
      model: selectedModel
    })

    try {
      const localModel = await resolveLocalModel(configuredModel)
      const normalizedModel = {
        ...configuredModel,
        ...localModel,
        path: localModel.path || localModel.modelPath || configuredModel.path || '',
        modelPath: localModel.modelPath || localModel.path || configuredModel.modelPath || ''
      }
      logger.info('[embedding] model resolved', {
        selectedModel,
        modelPath: normalizedModel.modelPath || normalizedModel.path || ''
      })
      resolvedModels.set(selectedModel, normalizedModel)
      return normalizedModel
    } catch (error) {
      if (!warnedMissingModel.has(selectedModel)) {
        logger.warn('[embedding] embedding model unavailable, using deterministic fallback', {
          selectedModel,
          error: error instanceof Error ? error.message : String(error || '')
        })
        warnedMissingModel.add(selectedModel)
      }
      resolvedModels.set(selectedModel, null)
      return null
    }
  }

  return {
    source: 'node-llama-cpp',
    resolveModel: resolveEmbeddingModel,
    embedText: async (text) => {
      const selectedModel = getSelectedModel('embedding') || 'smollm2-node-llama-cpp'
      const model = await resolveEmbeddingModel()
      if (!model) return createDeterministicEmbedding(text)
      try {
        return await runtime.nodeLlamaCppRuntime.embedText({ model, text })
      } catch (error) {
        if (!warnedInferenceFallback.has(selectedModel)) {
          logger.warn('[embedding] inference fallback to deterministic index', {
            selectedModel: model.id || model.name || model.model || '',
            error: error instanceof Error ? error.message : String(error || '')
          })
          warnedInferenceFallback.add(selectedModel)
        }
        return createDeterministicEmbedding(text)
      }
    }
  }
}
