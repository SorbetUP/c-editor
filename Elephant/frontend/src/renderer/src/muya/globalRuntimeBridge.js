import { createMuyaFullEditorRuntime } from './fullEditorRuntime.js'
import { isMuyaRuntimeActive, isMuyaRuntimeEnabled, readMuyaRuntimeMode } from './runtimeFlags.js'

const dispatchModeChanged = (target) => {
  if (typeof target.dispatchEvent !== 'function' || typeof target.Event !== 'function') return
  target.dispatchEvent(new target.Event('elephantnote:muya-runtime-mode-changed'))
}

export const createGlobalMuyaRuntimeBridge = (target = globalThis) => ({
  mode: () => readMuyaRuntimeMode(target),
  enabled: () => isMuyaRuntimeEnabled(readMuyaRuntimeMode(target)),
  active: () => isMuyaRuntimeActive(readMuyaRuntimeMode(target)),
  setMode: (mode) => {
    target.__ELEPHANT_MUYA_RUNTIME_MODE__ = mode
    const resolved = readMuyaRuntimeMode(target)
    dispatchModeChanged(target)
    return resolved
  },
  createEditor: createMuyaFullEditorRuntime
})

export const installGlobalMuyaRuntimeBridge = (target = globalThis) => {
  if (!target.__ELEPHANT_MUYA_RUNTIME__) {
    target.__ELEPHANT_MUYA_RUNTIME__ = createGlobalMuyaRuntimeBridge(target)
  }
  return target.__ELEPHANT_MUYA_RUNTIME__
}

export default installGlobalMuyaRuntimeBridge
