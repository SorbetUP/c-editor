import fs from 'fs-extra'
import path from 'path'
import {
  SYNC_BACKENDS,
  SYNC_STATUSES,
  createSyncHistoryRecord,
  createSyncQueueItem,
  createUnknownSyncOperationError,
  mergeSyncPeers
} from 'common/elephantnote/sync'

const nowIso = () => new Date().toISOString()
const CONFIG_DIR = '.elephantnote'
const CONFIG_FILE = 'sync-config.json'
const HISTORY_FILE = 'sync-log.json'
const MANIFEST_FILE = 'sync-manifest.json'
const RCLONE_FILTER_FILE = 'rclone-filter.txt'
const RUN_OPERATIONS = ['snapshot', 'pull', 'push', 'sync']
const PLAN_OPERATIONS = ['init', ...RUN_OPERATIONS]
const MISSING_REMOTE_MESSAGE = 'Sync remote is not configured. Choose a local LAN, Docker, or shared-folder target before syncing.'
const IGNORED_NAMES = new Set(['.git', '.elephantnote', 'node_modules', '.DS_Store'])

const hasOwnPayload = (payloadByOperation = {}, operation) => (
  Object.prototype.hasOwnProperty.call(payloadByOperation || {}, operation)
)

const samePayload = (left = {}, right = {}) => JSON.stringify(left || {}) === JSON.stringify(right || {})

const normalizePlanOperation = (operation = '') => {
  const normalized = String(operation || '').trim()
  return PLAN_OPERATIONS.includes(normalized) ? normalized : ''
}

const createEmbeddedLocalPlan = (payloadByOperation = {}) => {
  const explicitOperations = Array.isArray(payloadByOperation?.operations)
    ? payloadByOperation.operations.map(normalizePlanOperation).filter(Boolean)
    : []
  const operations = explicitOperations.length
    ? explicitOperations
    : PLAN_OPERATIONS.filter((operation) => hasOwnPayload(payloadByOperation, operation))

  return operations.map((operation) => ({
    operation,
    payload: payloadByOperation?.[operation] || {}
  }))
}

const readHistoryRecords = async(filePath) => {
  if (!(await fs.pathExists(filePath))) return []
  const data = await fs.readJson(filePath).catch(() => [])
  const records = Array.isArray(data) ? data : data?.records || data?.history || []
  return Array.isArray(records) ? records.filter((item) => item?.id && item?.operation) : []
}

const createPeerFromPayload = (payload = {}) => {
  const deviceId = String(payload.peerDeviceId || payload.deviceId || payload.id || '').trim()
  if (!deviceId) return null
  return {
    deviceId,
    name: payload.peerName || payload.name || payload.deviceName || deviceId,
    address: payload.peerAddress || payload.address || payload.endpoint || '',
    endpoint: payload.endpoint || payload.peerAddress || payload.address || '',
    vaultIds: payload.vaultIds || payload.vaults || [],
    online: payload.online !== false,
    trusted: payload.trusted !== false,
    encrypted: payload.encrypted !== false,
    pairedAt: payload.pairedAt || nowIso()
  }
}

const isIgnoredRelativePath = (relativePath = '') => String(relativePath).split(/[\\/]/).some((part) => IGNORED_NAMES.has(part))

const listFiles = async(root, current = root, out = []) => {
  if (!(await fs.pathExists(current))) return out
  for (const entry of await fs.readdir(current, { withFileTypes: true })) {
    if (IGNORED_NAMES.has(entry.name)) continue
    const absolutePath = path.join(current, entry.name)
    const relativePath = path.relative(root, absolutePath).split(path.sep).join('/')
    if (isIgnoredRelativePath(relativePath)) continue
    if (entry.isDirectory()) {
      await listFiles(root, absolutePath, out)
    } else if (entry.isFile()) {
      out.push(relativePath)
    }
  }
  return out.sort()
}

const sameFileContent = async(left, right) => {
  const [leftBuffer, rightBuffer] = await Promise.all([fs.readFile(left), fs.readFile(right)])
  return leftBuffer.length === rightBuffer.length && leftBuffer.equals(rightBuffer)
}

const conflictTarget = (target, tag, timestamp = Date.now()) => {
  const parsed = path.parse(target)
  return path.join(parsed.dir, `${parsed.name}.${tag}-conflict-${timestamp}${parsed.ext}`)
}

const copyTreeSafely = async({ sourceRoot, targetRoot, conflictTag }) => {
  const copied = []
  const conflicts = []
  await fs.ensureDir(targetRoot)
  for (const relativePath of await listFiles(sourceRoot)) {
    const source = path.join(sourceRoot, relativePath)
    const target = path.join(targetRoot, relativePath)
    await fs.ensureDir(path.dirname(target))
    if (!(await fs.pathExists(target))) {
      await fs.copy(source, target, { overwrite: false, errorOnExist: false })
      copied.push(relativePath)
      continue
    }
    if (await sameFileContent(source, target)) continue
    const safeConflictTarget = conflictTarget(target, conflictTag)
    await fs.copy(source, safeConflictTarget, { overwrite: false, errorOnExist: false })
    conflicts.push({ path: relativePath, preserved: target, incoming: safeConflictTarget })
  }
  return { copied, conflicts }
}

const fileHash = async(target) => {
  const buffer = await fs.readFile(target)
  let hash = 2166136261
  for (const byte of buffer) {
    hash ^= byte
    hash = Math.imul(hash, 16777619)
  }
  return (hash >>> 0).toString(16)
}

const createManifest = async(root) => {
  const files = []
  for (const relativePath of await listFiles(root)) {
    const target = path.join(root, relativePath)
    const stat = await fs.stat(target)
    files.push({ path: relativePath, size: stat.size, hash: await fileHash(target), updatedAt: stat.mtime.toISOString() })
  }
  return { version: 1, updatedAt: nowIso(), files }
}

export class RcloneVaultEngine {
  constructor({ cwd = '', rclone = null } = {}) {
    this.cwd = cwd
    this.rclone = rclone
    this.queue = []
    this.history = []
    this.running = false
    this.lastError = ''
    this.lastRunAt = ''
    this.remotePath = ''
    this.firstRunDone = false
    this.peers = []
    this.lastConfigLoadedFor = ''
    this.lastConflicts = []
  }

  get compatibilityRcloneMode() {
    return Boolean(this.rclone)
  }

  setCwd(cwd) {
    const nextCwd = cwd || ''
    if (nextCwd !== this.cwd) {
      this.cwd = nextCwd
      this.remotePath = ''
      this.firstRunDone = false
      this.peers = []
      this.lastConfigLoadedFor = ''
      this.queue = []
      this.history = []
      this.lastError = ''
      this.lastConflicts = []
    }
  }

  configPath() {
    return path.join(this.cwd, CONFIG_DIR, CONFIG_FILE)
  }

  syncDir() {
    return path.join(this.cwd, CONFIG_DIR, 'sync')
  }

  manifestPath() {
    return path.join(this.syncDir(), MANIFEST_FILE)
  }

  filterPath() {
    return path.join(this.syncDir(), RCLONE_FILTER_FILE)
  }

  historyPath() {
    return path.join(this.syncDir(), HISTORY_FILE)
  }

  compatibilityHistoryPath() {
    return path.join(this.cwd, CONFIG_DIR, HISTORY_FILE)
  }

  async ensureSupportFiles() {
    if (!this.cwd) return {}
    await fs.ensureDir(this.syncDir())
    if (this.compatibilityRcloneMode) {
      await fs.writeFile(this.filterPath(), [
        '- .git/**',
        '- .elephantnote/**',
        '- node_modules/**',
        '- .DS_Store'
      ].join('\n'))
    }
    return { manifestFile: this.manifestPath(), filterFile: this.filterPath() }
  }

  applyConfig(config = {}) {
    if (config?.remotePath || config?.remote) this.remotePath = String(config.remotePath || config.remote)
    this.firstRunDone = Boolean(config?.firstRunDone)
    this.peers = mergeSyncPeers([], config?.peers || [])
  }

  async loadConfig({ force = false } = {}) {
    if (!this.cwd) return
    if (!force && this.lastConfigLoadedFor === this.cwd) return
    const target = this.configPath()
    if (await fs.pathExists(target)) this.applyConfig(await fs.readJson(target).catch(() => ({})))
    const history = await readHistoryRecords(this.historyPath())
    this.history = history.length ? history : await readHistoryRecords(this.compatibilityHistoryPath())
    this.lastConfigLoadedFor = this.cwd
  }

  async persistConfig() {
    if (!this.cwd) return
    const target = this.configPath()
    await fs.ensureDir(path.dirname(target))
    await fs.writeJson(target, {
      version: 3,
      backend: this.compatibilityRcloneMode ? SYNC_BACKENDS.RCLONE : SYNC_BACKENDS.ELEPHANT_LOCAL,
      remotePath: this.remotePath,
      firstRunDone: this.firstRunDone,
      peers: this.peers,
      security: {
        encryptionRequired: true,
        externalRelayRequired: false
      },
      updatedAt: nowIso()
    }, { spaces: 2 })
    this.lastConfigLoadedFor = this.cwd
  }

  async persistHistory() {
    if (!this.cwd) return
    await fs.ensureDir(path.dirname(this.historyPath()))
    await fs.writeJson(this.historyPath(), { version: 1, updatedAt: nowIso(), history: this.history.slice(-100) }, { spaces: 2 })
  }

  plan(payloadByOperation = {}) {
    return createEmbeddedLocalPlan(payloadByOperation)
  }

  enqueue(operation) {
    const item = createSyncQueueItem(operation, new Date(), { strict: false })
    const duplicate = this.queue.find((entry) =>
      entry.status === SYNC_STATUSES.QUEUED &&
      entry.operation === item.operation &&
      samePayload(entry.payload, item.payload)
    )
    if (duplicate) return duplicate
    this.queue.push(item)
    return item
  }

  enqueuePlan(payloadByOperation = {}) {
    const operations = this.plan(payloadByOperation)
    if (!operations.length) {
      if (this.queue.some((item) => item.status === SYNC_STATUSES.QUEUED)) return []
      return this.remotePath ? [this.enqueue({ operation: 'sync', payload: {} })] : []
    }
    return operations.map((operation) => this.enqueue(operation))
  }

  configured() {
    return Boolean(this.remotePath)
  }

  status() {
    const backend = this.compatibilityRcloneMode ? SYNC_BACKENDS.RCLONE : SYNC_BACKENDS.ELEPHANT_LOCAL
    const rcloneStatus = this.rclone?.status?.() || { configured: false, lastError: '' }
    return {
      runtime: backend,
      backend,
      cwd: this.cwd,
      running: this.running,
      configured: this.configured(),
      remotePath: this.remotePath,
      firstRunDone: this.firstRunDone,
      peers: this.peers,
      queued: this.queue.filter((item) => item.status === SYNC_STATUSES.QUEUED).length,
      operations: this.queue.slice(-20),
      history: this.history.slice(-50),
      lastRunAt: this.lastRunAt,
      lastError: this.lastError,
      conflicts: this.lastConflicts.slice(-20),
      capabilities: {
        embeddedBackend: !this.compatibilityRcloneMode,
        requiresExternalBinary: this.compatibilityRcloneMode,
        requiresCloudAccount: false,
        encryptionRequired: true,
        desktopRclone: this.compatibilityRcloneMode,
        mobileRcloneBinary: false,
        mobileSyncRequiresBackend: this.compatibilityRcloneMode
      },
      rclone: {
        ...rcloneStatus,
        installed: this.compatibilityRcloneMode,
        running: this.running,
        lastError: rcloneStatus.lastError || ''
      }
    }
  }

  async run(payload = {}) {
    if (this.running) return this.status()
    await this.loadConfig({ force: true })
    this.mergeRemoteFromPayload(payload)
    this.mergePeersFromPayload(payload)
    const payloadKeys = Object.keys(payload || {})
    if (payloadKeys.length) {
      const enqueued = this.enqueuePlan(payload)
      if (!enqueued.length) await this.persistConfig()
    } else {
      this.enqueuePlan({})
    }
    this.running = true
    try {
      const queued = this.queue.filter((entry) => entry.status === SYNC_STATUSES.QUEUED)
      if (!queued.length) {
        if (!this.remotePath) this.lastError = MISSING_REMOTE_MESSAGE
        this.lastRunAt = nowIso()
        return this.status()
      }
      for (const item of queued) await this.runQueueItem(item)
      this.lastRunAt = nowIso()
      return this.status()
    } catch (error) {
      this.lastError = error?.message || 'Elephant sync failed.'
      throw error
    } finally {
      this.running = false
    }
  }

  mergeRemoteFromPayload(payload = {}) {
    const candidates = [
      payload?.remotePath,
      payload?.remote,
      payload?.init?.remotePath,
      payload?.init?.remote,
      payload?.snapshot?.remotePath,
      payload?.snapshot?.remote,
      payload?.sync?.remotePath,
      payload?.sync?.remote,
      payload?.pull?.remotePath,
      payload?.pull?.remote,
      payload?.push?.remotePath,
      payload?.push?.remote
    ]
    const nextRemotePath = candidates.find((value) => typeof value === 'string' && value.trim())
    if (nextRemotePath) this.remotePath = nextRemotePath.trim()
  }

  mergePeersFromPayload(payload = {}) {
    const candidates = [
      payload?.peer,
      createPeerFromPayload(payload),
      payload?.init?.peer,
      createPeerFromPayload(payload?.init || {}),
      payload?.snapshot?.peer,
      createPeerFromPayload(payload?.snapshot || {}),
      payload?.sync?.peer,
      createPeerFromPayload(payload?.sync || {}),
      payload?.pull?.peer,
      createPeerFromPayload(payload?.pull || {}),
      payload?.push?.peer,
      createPeerFromPayload(payload?.push || {})
    ]
    const peerList = [
      ...candidates.filter(Boolean),
      ...(Array.isArray(payload?.peers) ? payload.peers : []),
      ...(Array.isArray(payload?.init?.peers) ? payload.init.peers : [])
    ]
    if (peerList.length) this.peers = mergeSyncPeers(this.peers, peerList)
  }

  async recordQueueItem(item, status, error = '') {
    item.status = status
    item.error = error || ''
    item.updatedAt = nowIso()
    const record = createSyncHistoryRecord(item)
    this.history.push(record)
    await this.persistHistory()
    return item
  }

  async runQueueItem(item) {
    item.status = SYNC_STATUSES.RUNNING
    item.updatedAt = nowIso()
    try {
      if (item.operation === 'init') {
        this.mergeRemoteFromPayload({ init: item.payload })
        this.mergePeersFromPayload({ init: item.payload })
        await this.ensureSupportFiles()
        await this.persistConfig()
      } else if (RUN_OPERATIONS.includes(item.operation)) {
        if (item.payload.remotePath || item.payload.remote) this.remotePath = String(item.payload.remotePath || item.payload.remote).trim()
        if (this.compatibilityRcloneMode) await this.rcloneBisync(item.payload)
        else if (item.operation === 'snapshot') await this.snapshot()
        else {
          if (!this.remotePath) {
            this.lastError = MISSING_REMOTE_MESSAGE
            return this.recordQueueItem(item, SYNC_STATUSES.ERROR, MISSING_REMOTE_MESSAGE)
          }
          await this.sync(item.operation, item.payload)
        }
      } else {
        throw createUnknownSyncOperationError(item.operation)
      }
      await this.recordQueueItem(item, SYNC_STATUSES.DONE)
      return item
    } catch (error) {
      await this.recordQueueItem(item, SYNC_STATUSES.ERROR, error?.message || 'Sync operation failed.')
      throw error
    }
  }

  async snapshot() {
    if (!this.cwd) throw new Error('Elephant local sync requires an active vault path.')
    await this.ensureSupportFiles()
    await fs.writeJson(this.manifestPath(), await createManifest(this.cwd), { spaces: 2 })
    this.lastError = ''
    return this.status()
  }

  async rcloneBisync(payload = {}) {
    if (!this.cwd) throw new Error('Elephant rclone sync requires an active vault path.')
    if (payload.remotePath || payload.remote) this.remotePath = String(payload.remotePath || payload.remote).trim()
    if (!this.remotePath) {
      this.lastError = MISSING_REMOTE_MESSAGE
      return this.status()
    }
    await this.ensureSupportFiles()
    const args = ['bisync', this.cwd, this.remotePath, '--filters-file', this.filterPath()]
    if (!this.firstRunDone) args.push('--resync')
    await this.rclone.run(args, { cwd: this.cwd })
    this.firstRunDone = true
    this.lastError = ''
    await this.persistConfig()
    return this.status()
  }

  async sync(operation = 'sync', payload = {}) {
    if (typeof operation === 'object' && operation !== null) {
      payload = operation
      operation = 'sync'
    }
    if (this.compatibilityRcloneMode) return this.rcloneBisync(payload)
    const { remotePath = this.remotePath, remote = '' } = payload || {}
    this.remotePath = String(remotePath || remote || this.remotePath || '').trim()
    if (!this.cwd) throw new Error('Elephant local sync requires an active vault path.')
    if (!this.remotePath) {
      this.lastError = MISSING_REMOTE_MESSAGE
      return this.status()
    }
    await this.ensureSupportFiles()
    await this.persistConfig()
    await fs.ensureDir(this.remotePath)

    const conflicts = []
    if (operation === 'pull' || operation === 'sync') {
      const pullResult = await copyTreeSafely({ sourceRoot: this.remotePath, targetRoot: this.cwd, conflictTag: 'remote' })
      conflicts.push(...pullResult.conflicts)
    }
    if (operation === 'push' || operation === 'sync') {
      const pushResult = await copyTreeSafely({ sourceRoot: this.cwd, targetRoot: this.remotePath, conflictTag: 'local' })
      conflicts.push(...pushResult.conflicts)
    }

    this.lastConflicts = conflicts
    this.firstRunDone = true
    await this.snapshot()
    await this.persistConfig()
    this.lastError = conflicts.length ? 'Some files changed on both devices. Elephant kept both versions for review.' : ''
    return this.status()
  }
}
