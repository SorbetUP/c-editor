const normalizeEndpoint = (endpoint = '') => String(endpoint || '').trim().replace(/\/+$/, '')

const readJsonResponse = async (response) => {
  try {
    const text = await response.text()
    return text ? JSON.parse(text) : {}
  } catch {
    return {}
  }
}

export class SyncthingManager {
  constructor({ endpoint = 'http://127.0.0.1:8384', apiKey = '', fetchImpl = globalThis.fetch } = {}) {
    this.endpoint = normalizeEndpoint(endpoint) || 'http://127.0.0.1:8384'
    this.apiKey = apiKey
    this.fetchImpl = fetchImpl
  }

  request(path, options = {}) {
    return this.fetchImpl(`${this.endpoint}${path}`, {
      ...options,
      headers: {
        ...(options.headers || {}),
        ...(this.apiKey ? { 'X-API-Key': this.apiKey } : {})
      }
    })
  }

  async ping() {
    try {
      const response = await this.request('/rest/system/status', { method: 'GET' })
      if (!response.ok) {
        return {
          connected: false,
          endpoint: this.endpoint,
          lastError: `Syncthing API returned HTTP ${response.status}.`
        }
      }
      const data = await readJsonResponse(response)
      return {
        connected: true,
        endpoint: this.endpoint,
        ...data
      }
    } catch (error) {
      return {
        connected: false,
        endpoint: this.endpoint,
        lastError: error?.message || 'Unable to reach Syncthing API.'
      }
    }
  }

  async getConfig() {
    const response = await this.request('/rest/config', { method: 'GET' })
    if (!response.ok) {
      throw new Error(`Syncthing API returned HTTP ${response.status}.`)
    }
    return readJsonResponse(response)
  }

  async putConfig(config) {
    const response = await this.request('/rest/config', {
      method: 'PUT',
      body: JSON.stringify(config),
      headers: {
        'Content-Type': 'application/json'
      }
    })
    if (!response.ok) {
      throw new Error(`Syncthing API returned HTTP ${response.status}.`)
    }
    return readJsonResponse(response)
  }

  async ensureFolder({ folderId, label = '', path = '', type = 'sendreceive' } = {}) {
    const config = await this.getConfig()
    const folders = Array.isArray(config.folders) ? config.folders.slice() : []
    const nextFolder = {
      id: folderId,
      label: label || folderId,
      path,
      type
    }
    const index = folders.findIndex((folder) => folder.id === folderId)
    if (index >= 0) folders[index] = { ...(folders[index] || {}), ...nextFolder }
    else folders.push(nextFolder)
    await this.putConfig({ ...config, folders })
    return {
      id: folderId,
      path,
      label: label || folderId,
      type
    }
  }

  async ensurePeer({ deviceId, address, folderId } = {}) {
    const config = await this.getConfig()
    const devices = Array.isArray(config.devices) ? config.devices.slice() : []
    const folders = Array.isArray(config.folders) ? config.folders.slice() : []
    const deviceIndex = devices.findIndex((device) => device.deviceID === deviceId)
    const nextDevice = {
      deviceID: deviceId,
      name: deviceId,
      addresses: address ? [address] : []
    }
    if (deviceIndex >= 0) devices[deviceIndex] = { ...(devices[deviceIndex] || {}), ...nextDevice }
    else devices.push(nextDevice)

    const folderIndex = folders.findIndex((folder) => folder.id === folderId)
    if (folderIndex >= 0) {
      const folder = folders[folderIndex]
      const folderDevices = Array.isArray(folder.devices) ? folder.devices.slice() : []
      if (!folderDevices.some((item) => item.deviceID === deviceId)) {
        folderDevices.push({ deviceID: deviceId })
      }
      folders[folderIndex] = { ...folder, devices: folderDevices }
    }

    await this.putConfig({ ...config, devices, folders })
    return {
      deviceId,
      name: deviceId,
      address
    }
  }

  async folderStatus() {
    return {
      connected: false,
      configured: Boolean(this.endpoint),
      endpoint: this.endpoint,
      folderState: ''
    }
  }
}
