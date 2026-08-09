import MuyaRuntimeEditor from './MuyaRuntimeEditor.vue'
import { createMuyaFullEditorRuntime } from './fullEditorRuntime.js'
import { isMuyaRuntimeActive, isMuyaRuntimeEnabled, readMuyaRuntimeMode } from './runtimeFlags.js'
import { useMuyaRuntimeEditor } from './useMuyaRuntimeEditor.js'

export const createMuyaRuntimeApi = (target = globalThis) => ({
  mode: () => readMuyaRuntimeMode(target),
  enabled: () => isMuyaRuntimeEnabled(readMuyaRuntimeMode(target)),
  active: () => isMuyaRuntimeActive(readMuyaRuntimeMode(target)),
  createEditor: createMuyaFullEditorRuntime,
  useEditor: useMuyaRuntimeEditor,
  component: MuyaRuntimeEditor,
  setMode: (mode) => {
    target.__ELEPHANT_MUYA_RUNTIME_MODE__ = mode
    return readMuyaRuntimeMode(target)
  }
})

export const installMuyaRuntimeVuePlugin = (app, target = globalThis) => {
  const api = createMuyaRuntimeApi(target)
  target.__ELEPHANT_MUYA_RUNTIME__ = api
  app.config.globalProperties.$muyaRuntime = api
  app.provide('muyaRuntime', api)
  app.component('MuyaRuntimeEditor', MuyaRuntimeEditor)
  return api
}

export default {
  install(app) {
    installMuyaRuntimeVuePlugin(app)
  }
}
