import { modelRuntime } from '../runtime/elephantRuntime'

let registered = false

const DROP_KEYS = new Set(['runtime', 'loaded', 'controller', 'signal'])

const handleOnce = (ipcMain, channel, handler) => {
  ipcMain.removeHandler(channel)
  ipcMain.handle(channel, handler)
}

const serializeIpcValue = (value, seen = new WeakSet()) => {
  if (value == null) return value

  const valueType = typeof value
  if (valueType === 'string' || valueType === 'number' || valueType === 'boolean') {
    return value
  }
  if (valueType === 'bigint') {
    return value.toString()
  }
  if (valueType === 'function' || valueType === 'symbol' || valueType === 'undefined') {
    return undefined
  }
  if (value instanceof Date) {
    return value.toISOString()
  }
  if (Array.isArray(value)) {
    return value.map((item) => {
      const serialized = serializeIpcValue(item, seen)
      return serialized === undefined ? null : serialized
    })
  }
  if (value instanceof Map) {
    return serializeIpcValue(Object.fromEntries(value.entries()), seen)
  }
  if (value instanceof Set) {
    return serializeIpcValue(Array.from(value.values()), seen)
  }
  if (value instanceof Error) {
    return {
      name: value.name,
      message: value.message,
      stack: value.stack || ''
    }
  }
  if (valueType === 'object') {
    if (seen.has(value)) return undefined
    seen.add(value)

    const output = {}
    for (const [key, entry] of Object.entries(value)) {
      if (DROP_KEYS.has(key)) continue
      const serialized = serializeIpcValue(entry, seen)
      if (serialized !== undefined) {
        output[key] = serialized
      }
    }
    return output
  }

  return undefined
}

export const registerModelLibraryIpc = ({ ipcMain } = {}) => {
  if (registered || !ipcMain) return
  registered = true

  handleOnce(ipcMain, 'elephantnote:models:list', async() => serializeIpcValue(await modelRuntime.listLocalModels()))
  handleOnce(ipcMain, 'elephantnote:models:search-hf', async(_event, payload = {}) =>
    serializeIpcValue(await modelRuntime.searchHuggingFaceModels(payload)))
  handleOnce(ipcMain, 'elephantnote:models:info', async(_event, payload = {}) =>
    serializeIpcValue(await modelRuntime.getModelInfo(payload?.modelRef || payload)))
  handleOnce(ipcMain, 'elephantnote:models:download', async(event, payload = {}) =>
    serializeIpcValue(await modelRuntime.downloadModel(payload, {
      ...payload,
      onProgress: (progress) => event.sender.send('elephantnote:models:download:progress', progress)
    })))
  handleOnce(ipcMain, 'elephantnote:models:download:cancel', async(_event, payload = {}) =>
    serializeIpcValue(await modelRuntime.cancelDownload(payload?.downloadId || payload?.id || payload)))
  handleOnce(ipcMain, 'elephantnote:models:activate', async(_event, payload = {}) =>
    serializeIpcValue(await modelRuntime.activateModel(payload?.model || payload, payload)))
  handleOnce(ipcMain, 'elephantnote:models:deactivate', async(_event, payload = {}) =>
    serializeIpcValue(await modelRuntime.deactivateModel(payload?.modelRef || payload?.downloadId || payload)))
  handleOnce(ipcMain, 'elephantnote:models:delete', async(_event, payload = {}) =>
    serializeIpcValue(await modelRuntime.deleteModel(payload?.modelRef || payload?.model || payload)))
  handleOnce(ipcMain, 'elephantnote:models:active', async() => serializeIpcValue(await modelRuntime.getActiveModel()))
  handleOnce(ipcMain, 'elephantnote:models:download-status', async(_event, payload = {}) =>
    serializeIpcValue(await modelRuntime.getDownloadStatus(payload?.downloadId || payload?.id || payload)))
  handleOnce(ipcMain, 'elephantnote:models:refresh-index', async() => serializeIpcValue(await modelRuntime.refreshModelIndex()))
}

export {
  serializeIpcValue
}
