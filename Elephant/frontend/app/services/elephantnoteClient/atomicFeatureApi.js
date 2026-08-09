import { toPlainObject } from 'elephant-shared/plainObject'

const getBridge = () => globalThis.window?.elephantnote
const getTestIpc = () => globalThis.window?.tauri?.ipcRenderer

export const requireAtomicFeatureApi = () => {
  const atomicFeatures = getBridge()?.atomicFeatures
  if (atomicFeatures) return atomicFeatures

  const ipcRenderer = getTestIpc()
  if (!ipcRenderer?.invoke) {
    throw new Error('Atomic feature IPC is not available in this renderer context.')
  }

  return {
    describeApi: () => ipcRenderer.invoke('en:atomic:api:describe'),
    callApi: (payload = {}) => ipcRenderer.invoke('en:atomic:api:call', toPlainObject(payload)),
    providers: () => ipcRenderer.invoke('en:atomic:providers')
  }
}
