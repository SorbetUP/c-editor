import os from 'os'
import {
  createDefaultSyncPlan,
  createSyncConfig,
  createSyncHistoryRecord,
  createSyncIdentity,
  createSyncQueueItem,
  createSyncStatus,
  SYNC_BACKENDS,
  SYNC_OPERATION_IDS,
  SYNC_STATUSES
} from 'common/elephantnote/sync'

const isNonEmptyObject = (value) => value && typeof value === 'object' && !Array.isArray(value) && Object.keys(value).length > 0

const defaultExecutor = async () => ({ stdout: '', stderr: '' })

const toTrimmedText = (value = '') => String(value || '').trim()

export class GitSyncEngine {
  constructor({ cwd = '', executor = defaultExecutor, syncthing = null, hostname = os.hostname() } = {}) {
    this.cwd = cwd
    this.executor = executor
    this.syncthing = syncthing
    this.identity = createSyncIdentity({ cwd, hostname })
    this.config = createSyncConfig({ cwd, hostname })
    this.queue = []
    this.history = []
    this.repository = {
      branch: '',
      ahead: 0,
      behind: 0,
      dirty: false
    }
    this.syncthingStatus = {}
    this.running = false
    this.lastError = ''
  }

  setCwd(cwd = '') {
    this.cwd = cwd
    this.identity = createSyncIdentity({ cwd, hostname: os.hostname() })
    this.config = createSyncConfig({ ...this.config, cwd, hostname: os.hostname() })
  }

  status() {
    return createSyncStatus({
      cwd: this.cwd,
      running: this.running,
      queue: this.queue,
      history: this.history,
      lastError: this.lastError,
      config: {
        ...this.identity,
        backend: this.config.backend,
        remote: this.config.remote,
        peers: this.config.peers,
        branch: this.config.branch,
        syncthingEndpoint: this.config.syncthingEndpoint
      },
      repository: this.repository,
      syncthing: this.syncthingStatus
    })
  }

  enqueue({ operation, payload = {} } = {}) {
    const item = createSyncQueueItem({ operation, payload }, new Date(), { strict: false })
    item.status = SYNC_STATUSES.QUEUED
    this.queue.push(item)
    return item
  }

  enqueuePlan(payloadByOperation = {}) {
    for (const { operation, payload } of createDefaultSyncPlan(payloadByOperation)) {
      if (isNonEmptyObject(payload)) {
        this.enqueue({ operation, payload })
      }
    }
    return this.queue
  }

  async run() {
    if (!toTrimmedText(this.cwd)) {
      const error = new Error('Git sync requires an active vault path.')
      this.lastError = error.message
      throw error
    }
    this.running = true
    try {
      for (const item of this.queue) {
        if (item.status !== SYNC_STATUSES.QUEUED) continue
        item.status = SYNC_STATUSES.RUNNING
        item.updatedAt = new Date().toISOString()
        try {
          await this.executeOperation(item)
          item.status = SYNC_STATUSES.DONE
          item.error = ''
          this.history.unshift(createSyncHistoryRecord(item))
        } catch (error) {
          item.status = SYNC_STATUSES.ERROR
          item.error = error?.message || String(error)
          this.lastError = item.error
          this.history.unshift(createSyncHistoryRecord(item))
          throw error
        } finally {
          item.updatedAt = new Date().toISOString()
        }
      }
      this.lastError = ''
      return this.status()
    } finally {
      this.running = false
    }
  }

  async executeOperation(item) {
    if (!SYNC_OPERATION_IDS.includes(item.operation)) {
      throw new Error(`Unknown sync operation: ${item.operation}`)
    }

    if (item.operation === 'init') {
      await this.executeInit(item.payload || {})
      return
    }

    if (item.operation === 'snapshot') {
      await this.executeSnapshot(item.payload || {})
      return
    }

    if (item.operation === 'pull') {
      await this.executor('git', ['pull'], { cwd: this.cwd })
      return
    }

    if (item.operation === 'push') {
      await this.executor('git', ['push'], { cwd: this.cwd })
      return
    }

    throw new Error(`Unknown sync operation: ${item.operation}`)
  }

  async executeInit(payload = {}) {
    this.config = createSyncConfig({
      ...this.config,
      cwd: this.cwd,
      hostname: os.hostname(),
      backend: payload.backend || this.config.backend,
      branch: payload.branch || this.config.branch,
      syncthingEndpoint: payload.syncthingEndpoint || this.config.syncthingEndpoint,
      syncthingApiKey: payload.syncthingApiKey || this.config.syncthingApiKey,
      peers: payload.peerDeviceId && payload.peerAddress
        ? [{ deviceId: payload.peerDeviceId, address: payload.peerAddress }]
        : this.config.peers
    })

    await this.executor('git', ['init'], { cwd: this.cwd })

    if (this.config.backend === SYNC_BACKENDS.SYNCTHING_GIT && this.syncthing) {
      if (typeof this.syncthing.configure === 'function') {
        await this.syncthing.configure({
          endpoint: this.config.syncthingEndpoint,
          apiKey: this.config.syncthingApiKey,
          binaryPath: ''
        })
      }
      if (typeof this.syncthing.ensureFolder === 'function') {
        await this.syncthing.ensureFolder({
          folderId: this.identity.folderId,
          label: this.identity.folderLabel,
          path: this.cwd,
          type: 'sendreceive'
        })
      }
      if (payload.peerDeviceId && payload.peerAddress && typeof this.syncthing.ensurePeer === 'function') {
        await this.syncthing.ensurePeer({
          deviceId: payload.peerDeviceId,
          address: payload.peerAddress,
          folderId: this.identity.folderId
        })
      }
      if (typeof this.syncthing.folderStatus === 'function') {
        this.syncthingStatus = await this.syncthing.folderStatus({ folderId: this.identity.folderId })
      }
    }
  }

  async executeSnapshot(payload = {}) {
    const statusResult = await this.executor('git', ['status', '--porcelain'], { cwd: this.cwd })
    await this.executor('git', ['branch', '--show-current'], { cwd: this.cwd })
    const dirty = Boolean(String(statusResult?.stdout || '').trim())
    this.repository = {
      ...this.repository,
      dirty
    }

    if (!dirty) return

    await this.executor('git', ['add', '-A'], { cwd: this.cwd })
    await this.executor('git', ['commit', '-m', payload.message || 'Sync snapshot'], { cwd: this.cwd })
  }
}
