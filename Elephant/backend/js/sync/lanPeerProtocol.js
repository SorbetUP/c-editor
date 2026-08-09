export const LAN_SYNC_PROTOCOL = 'elephant-lan-sync'
export const LAN_SYNC_VERSION = 1
export const LAN_DISCOVERY_PORT = 47852
export const LAN_DISCOVERY_GROUP = '239.48.48.48'

const asList = (value) => Array.isArray(value) ? value : []

export const createPeerDescriptor = ({
  deviceId = '',
  deviceName = '',
  host = '',
  port = 0,
  vaults = [],
  capabilities = {}
} = {}) => ({
  protocol: LAN_SYNC_PROTOCOL,
  version: LAN_SYNC_VERSION,
  kind: 'peer-announcement',
  deviceId: String(deviceId || '').trim(),
  deviceName: String(deviceName || '').trim() || 'Elephant device',
  host: String(host || '').trim(),
  port: Number(port || 0),
  vaults: asList(vaults).map((vault) => ({
    id: String(vault.id || '').trim(),
    name: String(vault.name || '').trim(),
    shared: vault.shared !== false
  })).filter((vault) => vault.id || vault.name),
  capabilities: {
    encryptedSync: true,
    rcloneTransport: true,
    localNetwork: true,
    ...capabilities
  },
  announcedAt: new Date().toISOString()
})

export const serializePeerDescriptor = (descriptor = {}) => JSON.stringify(createPeerDescriptor(descriptor))

export const parsePeerDescriptor = (message = '') => {
  const parsed = typeof message === 'string' ? JSON.parse(message) : message
  if (parsed?.protocol !== LAN_SYNC_PROTOCOL || parsed?.version !== LAN_SYNC_VERSION) {
    throw new Error('Unsupported Elephant LAN sync announcement.')
  }
  if (parsed.kind !== 'peer-announcement') {
    throw new Error('Unsupported Elephant LAN sync message kind.')
  }
  return createPeerDescriptor(parsed)
}

export const createPairingInvite = ({
  fromDeviceId = '',
  fromDeviceName = '',
  vaultScope = 'active',
  vaults = [],
  endpoint = '',
  publicKey = '',
  salt = ''
} = {}) => ({
  type: 'elephant-lan-pairing-invite',
  version: LAN_SYNC_VERSION,
  fromDeviceId: String(fromDeviceId || '').trim(),
  fromDeviceName: String(fromDeviceName || '').trim() || 'Elephant device',
  vaultScope: vaultScope === 'all' ? 'all' : 'active',
  vaults: asList(vaults),
  endpoint: String(endpoint || '').trim(),
  publicKey: String(publicKey || '').trim(),
  salt: String(salt || '').trim(),
  encrypted: true
})

export const parsePairingInvite = (value = '') => {
  const invite = typeof value === 'string' ? JSON.parse(value) : value
  if (invite?.type !== 'elephant-lan-pairing-invite' || invite?.version !== LAN_SYNC_VERSION) {
    throw new Error('Invalid Elephant LAN pairing invite.')
  }
  return createPairingInvite(invite)
}

export const createPeerState = ({ descriptor = {}, online = false, lastSeenAt = '' } = {}) => ({
  ...descriptor,
  online: Boolean(online),
  lastSeenAt: lastSeenAt || new Date().toISOString()
})
