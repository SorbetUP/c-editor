import { ipcMain } from 'electron'
import { SitePreviewService } from './SitePreviewService'

const sitePreviewService = new SitePreviewService()

const normalizeFolderPayload = (params = {}) => {
  if (typeof params !== 'object' || params === null) {
    throw new Error('Invalid website preview payload.')
  }
  if (typeof params.vaultRoot !== 'string' || !params.vaultRoot.trim()) {
    throw new Error('A vault root is required.')
  }
  if (typeof params.folderPath !== 'string' || !params.folderPath.trim()) {
    throw new Error('A folder path is required.')
  }
  return {
    vaultRoot: params.vaultRoot,
    folderPath: params.folderPath
  }
}

export const registerSitePreviewIpc = () => {
  ipcMain.handle('en:site-preview:preview-folder', async(_event, params) => {
    return sitePreviewService.previewFolder(normalizeFolderPayload(params))
  })

  ipcMain.handle('en:site-preview:build-folder', async(_event, params) => {
    return sitePreviewService.buildFolder(normalizeFolderPayload(params))
  })

  ipcMain.handle('en:site-preview:stop', async(_event, siteId) => {
    if (typeof siteId !== 'string' || !siteId) throw new Error('A site id is required.')
    return sitePreviewService.stopPreview(siteId)
  })

  ipcMain.handle('en:site-preview:status', async(_event, siteId) => {
    if (typeof siteId !== 'string' || !siteId) throw new Error('A site id is required.')
    return sitePreviewService.getStatus(siteId)
  })

  ipcMain.handle('en:site-preview:open-external', async(_event, url) => {
    return sitePreviewService.openExternal(url)
  })
}

export const getSitePreviewService = () => sitePreviewService
