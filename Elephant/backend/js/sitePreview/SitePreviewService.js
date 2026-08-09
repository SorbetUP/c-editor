import { shell } from 'electron'
import fs from 'fs-extra'
import path from 'path'
import { getConfig } from '../config/elephantConfigStore'
import { ElephantSiteManager } from './ElephantSiteManager'
import { StaticSiteServer } from './StaticSiteServer'
import { SITE_PREVIEW_STATUS } from './siteTypes'

const getActiveVaultRoot = () => {
  const config = getConfig()
  const vault = config.vaults.find((item) => item.id === config.activeVaultId)
  if (!vault?.path) throw new Error('No active ElephantNote vault.')
  return path.resolve(vault.path)
}

const assertPathInside = (root, target) => {
  const relative = path.relative(root, target)
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error('Website preview folder must stay inside the active vault.')
  }
}

const resolveTrustedPreviewRequest = ({ folderPath } = {}) => {
  if (typeof folderPath !== 'string' || !folderPath.trim()) {
    throw new Error('A folder path is required.')
  }

  const vaultRoot = getActiveVaultRoot()
  const resolvedFolderPath = path.resolve(
    path.isAbsolute(folderPath) ? folderPath : path.join(vaultRoot, folderPath)
  )
  assertPathInside(vaultRoot, resolvedFolderPath)
  return {
    vaultRoot,
    folderPath: resolvedFolderPath
  }
}

export class SitePreviewService {
  constructor({ siteManager = new ElephantSiteManager() } = {}) {
    this.siteManager = siteManager
    this.previews = new Map()
  }

  async previewFolder(request) {
    const info = await this.siteManager.preparePreview({
      ...resolveTrustedPreviewRequest(request),
      mode: 'preview'
    })
    if (info.status === SITE_PREVIEW_STATUS.ERROR) {
      this.previews.set(info.id, info)
      return info
    }

    const existing = this.previews.get(info.id)
    await existing?.server?.stop()

    if (!(await fs.pathExists(path.join(info.outputDir, 'index.html')))) {
      const errorInfo = {
        ...info,
        status: SITE_PREVIEW_STATUS.ERROR,
        error: 'Website preview failed because no home page was generated.'
      }
      this.previews.set(info.id, errorInfo)
      return errorInfo
    }

    const server = new StaticSiteServer()
    const { port, url } = await server.start(info.outputDir)
    const readyInfo = {
      ...info,
      port,
      url,
      status: SITE_PREVIEW_STATUS.READY
    }
    this.previews.set(info.id, { ...readyInfo, server })
    return readyInfo
  }

  async buildFolder(request) {
    return this.siteManager.buildStaticSite({
      ...resolveTrustedPreviewRequest(request),
      mode: 'static-export'
    })
  }

  async stopPreview(siteId) {
    const preview = this.previews.get(siteId)
    if (!preview) return null
    await preview.server?.stop()
    const stopped = {
      ...preview,
      server: undefined,
      status: SITE_PREVIEW_STATUS.STOPPED
    }
    this.previews.set(siteId, stopped)
    return stopped
  }

  async getStatus(siteId) {
    const preview = this.previews.get(siteId)
    if (!preview) return null
    const { server, ...info } = preview
    return info
  }

  async openExternal(url) {
    if (typeof url !== 'string' || !url.startsWith('http://127.0.0.1:')) {
      throw new Error('Only local preview URLs can be opened.')
    }
    return shell.openExternal(url)
  }
}
