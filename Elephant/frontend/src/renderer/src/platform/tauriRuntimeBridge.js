import { installRuntimeBridge } from './runtimeBridge'

export const installTauriRuntimeBridge = (target = globalThis) => {
  if (!target?.__TAURI__) {
    throw new Error('ElephantNote renderer requires the Tauri runtime bridge.')
  }

  const result = installRuntimeBridge(target)
  const mode = result?.mode || target.__MARKTEXT_RUNTIME__ || ''
  if (mode !== 'tauri') {
    throw new Error(`ElephantNote renderer expected Tauri runtime, got "${mode || 'unknown'}".`)
  }
  return result
}
