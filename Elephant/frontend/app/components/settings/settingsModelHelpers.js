import { normalizeAiConfig } from 'common/elephantnote/aiProviders'
import { toPlainObject } from 'elephant-shared/plainObject'

export const clonePlainObject = (value) => {
  return toPlainObject(value)
}

export const createNodeLlamaCppTestConfig = ({
  aiConfig = {},
  modelSelection = {},
  fallbackChatModelId = ''
} = {}) => {
  const model =
    modelSelection.chat ||
    fallbackChatModelId ||
    aiConfig.model ||
    ''

  return normalizeAiConfig({
    ...clonePlainObject(aiConfig),
    preset: 'tauriRustLocal',
    transport: 'tauri-rust',
    endpoint: 'tauri-rust://local',
    model: String(model || '').trim()
  })
}
