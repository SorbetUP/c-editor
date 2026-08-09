import { BrowserWindow, ipcMain } from 'electron'

import { AtomicFeatureService } from './AtomicFeatureService'

const service = new AtomicFeatureService()
let registered = false

const getSenderWindowId = (event) => BrowserWindow.fromWebContents(event.sender)?.id ?? null

const withWindow = (event, payload = {}) => ({
  ...payload,
  windowId: getSenderWindowId(event)
})

const handleOnce = (channel, handler) => {
  ipcMain.removeHandler(channel)
  ipcMain.handle(channel, handler)
}

export const registerAtomicFeatureIpc = () => {
  if (registered) return
  registered = true
  handleOnce('en:atomic:api:describe', async() => service.describeApi())
  handleOnce('en:atomic:api:call', async(event, payload = {}) => service.callApi(withWindow(event, payload)))
  handleOnce('en:atomic:providers', async() => service.providers())
  handleOnce('en:atomic:overview', async(event, payload = {}) => service.overview(withWindow(event, payload)))
  handleOnce('en:atomic:graph', async(event, payload = {}) => service.graph(withWindow(event, payload)))
  handleOnce('en:atomic:wiki', async(event, payload = {}) => service.wiki(withWindow(event, payload)))
  handleOnce('en:atomic:wiki:create-page', async(event, payload = {}) => service.createWikiPage(withWindow(event, payload)))
  handleOnce('en:atomic:summarize', async(event, payload = {}) => service.summarize(withWindow(event, payload)))
  handleOnce('en:atomic:structure', async(event, payload = {}) => service.structure(withWindow(event, payload)))
  handleOnce('en:atomic:notes:auto-name', async(event, payload = {}) => service.autoNameNote(withWindow(event, payload)))
  handleOnce('en:atomic:models:list-local', async(_event, payload = {}) => service.listLocalModels(payload))
  handleOnce('en:atomic:models:pull', async(event, payload = {}) => service.pullModel({
    ...payload,
    onProgress: (progress) => event.sender.send('en:atomic:models:pull:progress', progress)
  }))
}

registerAtomicFeatureIpc()

export const getAtomicFeatureService = () => service
