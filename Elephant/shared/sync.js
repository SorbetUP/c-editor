export const SYNC_OPERATIONS = Object.freeze({
  INIT: 'init',
  SNAPSHOT: 'snapshot',
  PULL: 'pull',
  PUSH: 'push',
  SYNC: 'sync',
  DISCOVER: 'discover',
  PAIR: 'pair',
  EXCHANGE_HEADS: 'exchange-heads',
  RECONCILE: 'reconcile',
  TRANSFER: 'transfer',
  VERIFY: 'verify',
  ACK: 'ack'
})

export const SYNC_OPERATION_SEQUENCE = Object.freeze([
  SYNC_OPERATIONS.INIT,
  SYNC_OPERATIONS.SNAPSHOT
])

export const SYNC_GIT_REMOTE_OPERATION_SEQUENCE = Object.freeze([
  ...SYNC_OPERATION_SEQUENCE,
  SYNC_OPERATIONS.PULL,
  SYNC_OPERATIONS.PUSH
])

export const SYNC_USER_FRIENDLY_OPERATION_SEQUENCE = Object.freeze([
  SYNC_OPERATIONS.DISCOVER,
  SYNC_OPERATIONS.PAIR,
  SYNC_OPERATIONS.EXCHANGE_HEADS,
  SYNC_OPERATIONS.RECONCILE,
  SYNC_OPERATIONS.TRANSFER,
  SYNC_OPERATIONS.VERIFY,
  SYNC_OPERATIONS.ACK
])

export const SYNC_OPERATION_IDS = Object.freeze(Object.values(SYNC_OPERATIONS))

export const SYNC_STATUSES = Object.freeze({
  QUEUED: 'queued',
  RUNNING: 'running',
  DONE: 'done',
  ERROR: 'error',
  NEEDS_USER: 'needs-user-action'
})

export const SYNC_ERROR_CODES = Object.freeze({
  NO_VAULT: 'ELEPHANTNOTE_SYNC_NO_VAULT',
  UNKNOWN_OPERATION: 'ELEPHANTNOTE_UNKNOWN_SYNC_OPERATION',
  PAIRING_EXPIRED: 'ELEPHANTNOTE_SYNC_PAIRING_EXPIRED',
  CONFLICT_REQUIRES_REVIEW: 'ELEPHANTNOTE_SYNC_CONFLICT_REQUIRES_REVIEW'
})

export const SYNC_DEFAULT_REMOTE = 'origin'
export const SYNC_METADATA_DIR = '.elephantnote/sync'
export const SYNC_COMPATIBILITY_METADATA_DIR = '.elephantnote'
export const SYNC_HISTORY_FILE = 'sync-log.json'
export const SYNC_CONFIG_FILE = 'sync-config.json'
export const SYNC_BACKENDS = Object.freeze({
  ELEPHANT_LOCAL: 'elephant-local',
  GIT: 'git',
  RCLONE: 'rclone',
  SYNCTHING_GIT: 'syncthing-git'
})

export const SYNC_BACKEND_IDS = Object.freeze(Object.values(SYNC_BACKENDS))

export const SYNC_PLATFORMS = Object.freeze({
  ANDROID: 'android',
  WINDOWS: 'windows',
  MACOS: 'macos',
  LINUX: 'linux',
  DOCKER: 'docker',
  WEB: 'web'
})

export const SYNC_SECURITY_MODES = Object.freeze({
  LOCAL_FIRST_E2EE: 'local-first-e2ee',
  TRUSTED_LAN: 'trusted-lan'
})

export const SYNC_TRANSPORTS = Object.freeze({
  LOCAL_LAN: 'local-lan',
  DOCKER_BRIDGE: 'docker-bridge',
  LOCAL_FOLDER: 'local-folder'
})

const DEFAULT_PAIRING_TTL_MS = 10 * 60 * 1000

const compactHash = (value = '') => {
  let hash = 5381
  for (const char of String(value)) hash = ((hash << 5) + hash) ^ char.charCodeAt(0)
  return Math.abs(hash >>> 0).toString(36)
}

const normalizeIsoDate = (value = '', fallback = new Date()) => {
  const text = String(value || '').trim()
  if (text && !Number.isNaN(Date.parse(text))) return new Date(text).toISOString()
  return fallback.toISOString()
}

const normalizeStringList = (value = []) => {
  const list = Array.isArray(value) ? value : []
  return [...new Set(list.map((item) => {
    if (typeof item === 'string') return item.trim()
    return String(item?.id || item?.name || item?.path || '').trim()
  }).filter(Boolean))]
}

const normalizePeerId = (peer = {}) => String(peer.deviceId || peer.id || '').trim()
const normalizePlatform = (platform = '') => {
  const normalized = String(platform || '').trim().toLowerCase()
  return Object.values(SYNC_PLATFORMS).includes(normalized) ? normalized : SYNC_PLATFORMS.WEB
}

export const normalizeSyncOperation = (operation = '') => {
  const normalized = String(operation || '').trim()
  return SYNC_OPERATION_IDS.includes(normalized) ? normalized : ''
}

export const createUnknownSyncOperationError = (operation = '') => {
  const error = new Error(`Unknown sync operation: ${operation}.`)
  error.code = SYNC_ERROR_CODES.UNKNOWN_OPERATION
  return error
}

export const createMissingVaultSyncError = () => {
  const error = new Error('Elephant Sync requires an active vault path.')
  error.code = SYNC_ERROR_CODES.NO_VAULT
  return error
}

export const createSyncQueueItem = ({ operation, payload = {} } = {}, now = new Date(), { strict = true } = {}) => {
  const normalizedOperation = normalizeSyncOperation(operation)
  if (!normalizedOperation && strict) throw createUnknownSyncOperationError(operation)
  const timestamp = now.toISOString()
  return {
    id: `sync-${now.getTime()}-${Math.random().toString(36).slice(2, 8)}`,
    operation: normalizedOperation || String(operation || '').trim(),
    payload: payload && typeof payload === 'object' && !Array.isArray(payload) ? payload : {},
    status: SYNC_STATUSES.QUEUED,
    createdAt: timestamp,
    updatedAt: timestamp,
    error: ''
  }
}

const hasPayloadObject = (payloadByOperation = {}, operation) => (
  Object.prototype.hasOwnProperty.call(payloadByOperation || {}, operation)
)

const createPlanItem = (payloadByOperation = {}, operation) => ({
  operation,
  payload: payloadByOperation?.[operation] || {}
})

const normalizeExplicitOperations = (operations = []) => (Array.isArray(operations) ? operations : [])
  .map((operation) => normalizeSyncOperation(operation))
  .filter(Boolean)

export const createDefaultSyncPlan = (payloadByOperation = {}) => {
  const explicitOperations = normalizeExplicitOperations(payloadByOperation?.operations)
  if (explicitOperations.length) {
    return explicitOperations.map((operation) => createPlanItem(payloadByOperation, operation))
  }

  if (hasPayloadObject(payloadByOperation, SYNC_OPERATIONS.SYNC)) {
    return SYNC_GIT_REMOTE_OPERATION_SEQUENCE.map((operation) => ({
      operation,
      payload: payloadByOperation?.[operation] || payloadByOperation?.[SYNC_OPERATIONS.SYNC] || {}
    }))
  }

  const hasExplicitGitOperation = [SYNC_OPERATIONS.INIT, SYNC_OPERATIONS.SNAPSHOT, SYNC_OPERATIONS.PULL, SYNC_OPERATIONS.PUSH]
    .some((operation) => hasPayloadObject(payloadByOperation, operation))

  if (!hasExplicitGitOperation) {
    return SYNC_OPERATION_SEQUENCE.map((operation) => createPlanItem(payloadByOperation, operation))
  }

  const plan = []
  if (
    hasPayloadObject(payloadByOperation, SYNC_OPERATIONS.INIT) ||
    hasPayloadObject(payloadByOperation, SYNC_OPERATIONS.SNAPSHOT) ||
    hasPayloadObject(payloadByOperation, SYNC_OPERATIONS.PULL) ||
    hasPayloadObject(payloadByOperation, SYNC_OPERATIONS.PUSH)
  ) {
    plan.push(createPlanItem(payloadByOperation, SYNC_OPERATIONS.INIT))
  }
  for (const operation of [SYNC_OPERATIONS.SNAPSHOT, SYNC_OPERATIONS.PULL, SYNC_OPERATIONS.PUSH]) {
    if (hasPayloadObject(payloadByOperation, operation)) plan.push(createPlanItem(payloadByOperation, operation))
  }
  return plan
}

export const createSyncIdentity = ({ cwd = '', hostname = '', now = new Date() } = {}) => {
  const seed = `${cwd}|${hostname}`
  return {
    deviceId: `en-${compactHash(seed || now.toISOString())}`,
    folderId: `vault-${compactHash(cwd || 'vault')}`,
    folderLabel: cwd ? String(cwd).split(/[\\/]/).filter(Boolean).at(-1) || 'Vault' : 'Vault'
  }
}

export const normalizeSyncPeer = (peer = {}, now = new Date()) => {
  const deviceId = normalizePeerId(peer)
  const address = String(peer.address || peer.peerAddress || peer.endpoint || '').trim()
  const name = String(peer.name || peer.deviceName || peer.label || deviceId || 'Elephant device').trim()
  const fallbackIdSeed = `${name}|${address}|${peer.publicKey || ''}`
  const id = deviceId || (fallbackIdSeed.trim() ? `peer-${compactHash(fallbackIdSeed)}` : '')
  if (!id) return null
  const pairedAt = peer.pairedAt ? normalizeIsoDate(peer.pairedAt, now) : ''
  return {
    id,
    deviceId: deviceId || id,
    name: name || id,
    address,
    endpoint: String(peer.endpoint || address || '').trim(),
    vaultIds: normalizeStringList(peer.vaultIds || peer.vaults),
    online: Boolean(peer.online),
    pairedAt,
    lastSeenAt: normalizeIsoDate(peer.lastSeenAt || peer.announcedAt, now),
    trusted: Boolean(peer.trusted),
    encrypted: peer.encrypted !== false && Boolean(peer.trusted || peer.encrypted),
    fingerprint: String(peer.fingerprint || '').trim()
  }
}

export const mergeSyncPeers = (currentPeers = [], incomingPeers = [], now = new Date()) => {
  const peers = new Map()
  const upsertPeer = (peer, incoming = false) => {
    const normalized = normalizeSyncPeer(peer, now)
    if (!normalized) return
    const previous = peers.get(normalized.id)
    if (!previous) {
      peers.set(normalized.id, normalized)
      return
    }
    peers.set(normalized.id, {
      ...previous,
      ...normalized,
      vaultIds: [...new Set([...(previous.vaultIds || []), ...(normalized.vaultIds || [])])],
      pairedAt: normalized.pairedAt || previous.pairedAt,
      online: incoming ? normalized.online : previous.online || normalized.online,
      trusted: previous.trusted || normalized.trusted,
      encrypted: previous.encrypted || normalized.encrypted,
      fingerprint: normalized.fingerprint || previous.fingerprint,
      lastSeenAt: incoming ? normalized.lastSeenAt : previous.lastSeenAt || normalized.lastSeenAt
    })
  }

  for (const peer of Array.isArray(currentPeers) ? currentPeers : []) upsertPeer(peer, false)
  for (const peer of Array.isArray(incomingPeers) ? incomingPeers : []) upsertPeer(peer, true)
  return [...peers.values()].sort((a, b) => a.name.localeCompare(b.name) || a.id.localeCompare(b.id))
}

export const createSyncConfig = ({
  cwd = '',
  hostname = '',
  remote = '',
  remotePath = '',
  remoteName = SYNC_DEFAULT_REMOTE,
  branch = '',
  mode = 'send-receive',
  backend = SYNC_BACKENDS.ELEPHANT_LOCAL,
  peers = [],
  now = new Date()
} = {}) => ({
  version: 3,
  ...createSyncIdentity({ cwd, hostname, now }),
  backend: SYNC_BACKEND_IDS.includes(backend) ? backend : SYNC_BACKENDS.ELEPHANT_LOCAL,
  mode,
  remoteName: remoteName || SYNC_DEFAULT_REMOTE,
  remote,
  remotePath,
  branch,
  peers: mergeSyncPeers([], peers, now),
  security: {
    mode: SYNC_SECURITY_MODES.LOCAL_FIRST_E2EE,
    encryptionRequired: true,
    externalRelayRequired: false
  },
  updatedAt: now.toISOString()
})

export const createPlatformSyncCapabilities = (platform = SYNC_PLATFORMS.WEB) => {
  const normalized = normalizePlatform(platform)
  const isDocker = normalized === SYNC_PLATFORMS.DOCKER
  return {
    platform: normalized,
    backend: SYNC_BACKENDS.ELEPHANT_LOCAL,
    embeddedBackend: true,
    requiresExternalBinary: false,
    requiresCloudAccount: false,
    supportsLocalLan: normalized !== SYNC_PLATFORMS.WEB,
    supportsDockerBridge: isDocker,
    supportsQrPairing: normalized === SYNC_PLATFORMS.ANDROID,
    supportedTransports: isDocker
      ? [SYNC_TRANSPORTS.LOCAL_LAN, SYNC_TRANSPORTS.DOCKER_BRIDGE, SYNC_TRANSPORTS.LOCAL_FOLDER]
      : [SYNC_TRANSPORTS.LOCAL_LAN, SYNC_TRANSPORTS.LOCAL_FOLDER]
  }
}

export const createPairingInvite = ({
  deviceId = '',
  deviceName = 'Elephant device',
  platform = SYNC_PLATFORMS.WEB,
  vaultIds = [],
  pairingCode = '',
  now = new Date(),
  ttlMs = DEFAULT_PAIRING_TTL_MS
} = {}) => {
  const createdAt = now.toISOString()
  const expiresAt = new Date(now.getTime() + ttlMs).toISOString()
  const secretSeed = `${deviceId}|${pairingCode}|${createdAt}`
  return {
    version: 1,
    protocol: 'elephant-sync-local-v1',
    transport: SYNC_TRANSPORTS.LOCAL_LAN,
    backend: SYNC_BACKENDS.ELEPHANT_LOCAL,
    deviceId: String(deviceId || '').trim(),
    deviceName: String(deviceName || 'Elephant device').trim(),
    platform: normalizePlatform(platform),
    vaultIds: normalizeStringList(vaultIds),
    createdAt,
    expiresAt,
    pairingCodeHash: compactHash(secretSeed),
    fingerprint: compactHash(`${secretSeed}|fingerprint`),
    security: {
      mode: SYNC_SECURITY_MODES.LOCAL_FIRST_E2EE,
      encryptionRequired: true,
      plaintextSecretIncluded: false,
      externalRelayRequired: false
    }
  }
}

export const createTrustedSyncDevice = ({ invite = {}, remoteDevice = {}, now = new Date() } = {}) => normalizeSyncPeer({
  deviceId: remoteDevice.deviceId || remoteDevice.id || invite.deviceId,
  name: remoteDevice.name || remoteDevice.deviceName || invite.deviceName,
  address: remoteDevice.address || remoteDevice.endpoint || '',
  endpoint: remoteDevice.endpoint || remoteDevice.address || '',
  vaultIds: remoteDevice.vaultIds || invite.vaultIds || [],
  online: true,
  trusted: true,
  encrypted: true,
  pairedAt: now.toISOString(),
  lastSeenAt: now.toISOString(),
  fingerprint: remoteDevice.fingerprint || invite.fingerprint || ''
}, now)

export const createUserFriendlySyncPlan = ({
  platform = SYNC_PLATFORMS.WEB,
  devices = [],
  vaultIds = [],
  changedFiles = [],
  conflicts = [],
  now = new Date()
} = {}) => {
  const capabilities = createPlatformSyncCapabilities(platform)
  const trustedDevices = mergeSyncPeers([], devices, now).filter((device) => device.trusted && device.encrypted)
  const hasConflict = Array.isArray(conflicts) && conflicts.length > 0
  const operations = hasConflict
    ? [SYNC_OPERATIONS.EXCHANGE_HEADS, SYNC_OPERATIONS.RECONCILE, SYNC_OPERATIONS.VERIFY]
    : SYNC_USER_FRIENDLY_OPERATION_SEQUENCE
  return {
    backend: SYNC_BACKENDS.ELEPHANT_LOCAL,
    externalDependencyFree: true,
    securityMode: SYNC_SECURITY_MODES.LOCAL_FIRST_E2EE,
    encryptionRequired: true,
    requiresExternalBinary: false,
    platform: capabilities.platform,
    vaultIds: normalizeStringList(vaultIds),
    devices: trustedDevices,
    changedFiles: normalizeStringList(changedFiles),
    conflicts: Array.isArray(conflicts) ? conflicts : [],
    operations,
    items: operations.map((operation) => ({ operation, payload: { vaultIds: normalizeStringList(vaultIds) } })),
    userMessage: hasConflict
      ? 'A conflict needs your review. Elephant kept both versions safe.'
      : trustedDevices.length
        ? 'Ready to sync securely on your local network.'
        : 'Find or pair a device to start secure local sync.'
  }
}

export const classifySyncConflict = ({ path = '', localChanged = false, remoteChanged = false, sameContent = false } = {}) => {
  if (sameContent || !localChanged || !remoteChanged) {
    return {
      conflict: false,
      path,
      resolution: 'auto',
      userMessage: 'No conflict detected.'
    }
  }
  return {
    conflict: true,
    path,
    resolution: 'preserve-both-and-review',
    errorCode: SYNC_ERROR_CODES.CONFLICT_REQUIRES_REVIEW,
    userMessage: 'Both devices changed this note. Elephant kept both versions and will ask you which one to keep.'
  }
}

export const createSyncHistoryRecord = (item = {}) => ({
  id: item.id,
  operation: item.operation,
  status: item.status,
  updatedAt: item.updatedAt,
  error: item.error || ''
})

export const createSyncStatus = ({
  cwd = '',
  running = false,
  queue = [],
  history = [],
  lastRunAt = '',
  lastError = '',
  config = null,
  repository = {}
} = {}) => ({
  runtime: 'elephant-local',
  cwd,
  running,
  deviceId: config?.deviceId || '',
  folderId: config?.folderId || '',
  backend: config?.backend || SYNC_BACKENDS.ELEPHANT_LOCAL,
  remote: config?.remote || '',
  remotePath: config?.remotePath || '',
  peers: mergeSyncPeers([], config?.peers || []),
  branch: repository.branch || config?.branch || '',
  ahead: repository.ahead || 0,
  behind: repository.behind || 0,
  dirty: Boolean(repository.dirty),
  syncthing: {
    configured: false,
    connected: false,
    endpoint: '',
    localDeviceId: '',
    folderState: '',
    lastError: ''
  },
  capabilities: {
    embeddedBackend: true,
    requiresExternalBinary: false,
    requiresCloudAccount: false,
    encryptionRequired: true
  },
  queued: queue.filter((item) => item.status === SYNC_STATUSES.QUEUED).length,
  operations: queue.slice(-20),
  history: history.slice(-50),
  lastRunAt,
  lastError
})

export const createSyncUserMessage = (status = {}) => {
  if (status.lastError) return { tone: 'error', text: status.lastError }
  if (status.running) return { tone: 'busy', text: 'Syncing securely on your local network…' }
  if (!status.cwd) return { tone: 'setup', text: 'Open a vault before enabling sync.' }
  if (!Array.isArray(status.peers) || !status.peers.length) return { tone: 'setup', text: 'Find a phone, PC, Mac, or Docker device to start secure local sync.' }
  const online = status.peers.filter((peer) => peer.online).length
  if (!online) return { tone: 'offline', text: 'Your paired devices are offline. Elephant will sync when they return.' }
  return { tone: 'ready', text: `${online} paired device${online === 1 ? '' : 's'} ready for secure local sync.` }
}
