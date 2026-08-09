import { describe, expect, it } from 'vitest'
import {
  SYNC_BACKENDS,
  SYNC_ERROR_CODES,
  SYNC_OPERATION_SEQUENCE,
  SYNC_OPERATIONS,
  SYNC_PLATFORMS,
  SYNC_SECURITY_MODES,
  SYNC_STATUSES,
  SYNC_USER_FRIENDLY_OPERATION_SEQUENCE,
  classifySyncConflict,
  createDefaultSyncPlan,
  createMissingVaultSyncError,
  createPairingInvite,
  createPlatformSyncCapabilities,
  createSyncHistoryRecord,
  createSyncQueueItem,
  createSyncStatus,
  createSyncUserMessage,
  createTrustedSyncDevice,
  createUserFriendlySyncPlan,
  mergeSyncPeers,
  normalizeSyncOperation,
  normalizeSyncPeer
} from 'common/elephantnote/sync'

describe('ElephantNote sync contract', () => {
  it('defines the portable sync operation plan', () => {
    expect(SYNC_OPERATION_SEQUENCE).toEqual([
      SYNC_OPERATIONS.INIT,
      SYNC_OPERATIONS.SNAPSHOT
    ])
    expect(createDefaultSyncPlan({ snapshot: { message: 'Manual snapshot' } })).toEqual([
      { operation: 'init', payload: {} },
      { operation: 'snapshot', payload: { message: 'Manual snapshot' } }
    ])
    expect(normalizeSyncOperation('sync')).toBe(SYNC_OPERATIONS.SYNC)
  })

  it('normalizes queue items and rejects unknown operations', () => {
    const item = createSyncQueueItem(
      { operation: ' snapshot ', payload: { message: 'A' } },
      new Date('2026-06-14T00:00:00.000Z')
    )

    expect(item).toMatchObject({
      operation: 'snapshot',
      payload: { message: 'A' },
      status: SYNC_STATUSES.QUEUED,
      createdAt: '2026-06-14T00:00:00.000Z'
    })
    expect(normalizeSyncOperation('missing')).toBe('')
    expect(() => createSyncQueueItem({ operation: 'missing' })).toThrow('Unknown sync operation')
  })

  it('normalizes sync peers for portable device pairing state', () => {
    const peer = normalizeSyncPeer({
      deviceId: ' phone-1 ',
      deviceName: 'Phone',
      peerAddress: ' tcp://192.168.1.40:22000 ',
      vaultIds: ['vault-a', '', 'vault-a'],
      online: true,
      pairedAt: '2026-06-14T00:00:00.000Z'
    }, new Date('2026-06-14T00:05:00.000Z'))

    expect(peer).toMatchObject({
      id: 'phone-1',
      deviceId: 'phone-1',
      name: 'Phone',
      address: 'tcp://192.168.1.40:22000',
      vaultIds: ['vault-a'],
      online: true,
      pairedAt: '2026-06-14T00:00:00.000Z',
      lastSeenAt: '2026-06-14T00:05:00.000Z'
    })
  })

  it('merges sync peers without duplicating re-discovered devices', () => {
    const peers = mergeSyncPeers(
      [{ deviceId: 'phone-1', name: 'Phone', vaultIds: ['vault-a'], pairedAt: '2026-06-14T00:00:00.000Z' }],
      [{ id: 'phone-1', label: 'Phone LAN', peerAddress: 'dynamic', vaultIds: ['vault-b'], online: true }],
      new Date('2026-06-14T00:10:00.000Z')
    )

    expect(peers).toHaveLength(1)
    expect(peers[0]).toMatchObject({
      id: 'phone-1',
      deviceId: 'phone-1',
      name: 'Phone LAN',
      address: 'dynamic',
      online: true,
      pairedAt: '2026-06-14T00:00:00.000Z',
      lastSeenAt: '2026-06-14T00:10:00.000Z'
    })
    expect(peers[0].vaultIds).toEqual(['vault-a', 'vault-b'])
  })

  it('creates portable status, history and error shapes', () => {
    const item = { id: 'sync-1', operation: 'pull', status: 'done', updatedAt: '2026-06-14T00:00:00.000Z' }

    expect(createSyncHistoryRecord(item)).toEqual({
      id: 'sync-1',
      operation: 'pull',
      status: 'done',
      updatedAt: '2026-06-14T00:00:00.000Z',
      error: ''
    })
    expect(createSyncStatus({
      cwd: '/vault',
      queue: [{ status: 'queued' }, { status: 'done' }],
      config: { peers: [{ deviceId: 'phone-1', online: true }] }
    })).toMatchObject({
      cwd: '/vault',
      backend: SYNC_BACKENDS.ELEPHANT_LOCAL,
      queued: 1,
      running: false,
      peers: [{ deviceId: 'phone-1', online: true }],
      capabilities: {
        embeddedBackend: true,
        requiresExternalBinary: false,
        encryptionRequired: true
      },
      syncthing: { configured: false, connected: false }
    })
    expect(createMissingVaultSyncError()).toMatchObject({ code: SYNC_ERROR_CODES.NO_VAULT })
  })

  it('describes a no-external-dependency backend on Android, desktop, macOS and Docker', () => {
    for (const platform of [SYNC_PLATFORMS.ANDROID, SYNC_PLATFORMS.WINDOWS, SYNC_PLATFORMS.MACOS, SYNC_PLATFORMS.DOCKER]) {
      expect(createPlatformSyncCapabilities(platform)).toMatchObject({
        platform,
        backend: SYNC_BACKENDS.ELEPHANT_LOCAL,
        embeddedBackend: true,
        requiresExternalBinary: false,
        requiresCloudAccount: false
      })
    }
    expect(createPlatformSyncCapabilities(SYNC_PLATFORMS.DOCKER).supportedTransports).toContain('docker-bridge')
    expect(createPlatformSyncCapabilities(SYNC_PLATFORMS.ANDROID).supportsQrPairing).toBe(true)
  })

  it('creates a safe pairing invite for QR/manual onboarding without leaking the password', () => {
    const invite = createPairingInvite({
      deviceId: 'pc-1',
      deviceName: 'Desktop',
      platform: SYNC_PLATFORMS.MACOS,
      vaultIds: ['main-vault'],
      pairingCode: '12345678',
      now: new Date('2026-06-14T10:00:00.000Z')
    })

    expect(invite).toMatchObject({
      protocol: 'elephant-sync-local-v1',
      backend: SYNC_BACKENDS.ELEPHANT_LOCAL,
      deviceId: 'pc-1',
      deviceName: 'Desktop',
      platform: SYNC_PLATFORMS.MACOS,
      vaultIds: ['main-vault'],
      createdAt: '2026-06-14T10:00:00.000Z',
      expiresAt: '2026-06-14T10:10:00.000Z',
      security: {
        mode: SYNC_SECURITY_MODES.LOCAL_FIRST_E2EE,
        encryptionRequired: true,
        plaintextSecretIncluded: false,
        externalRelayRequired: false
      }
    })
    expect(JSON.stringify(invite)).not.toContain('12345678')
  })

  it('turns a pairing invite into a trusted encrypted device', () => {
    const trusted = createTrustedSyncDevice({
      invite: createPairingInvite({ deviceId: 'phone-1', deviceName: 'Android', pairingCode: 'abcdefgh' }),
      remoteDevice: { endpoint: 'tcp://192.168.1.42:17777', vaultIds: ['main'] },
      now: new Date('2026-06-14T10:00:00.000Z')
    })

    expect(trusted).toMatchObject({
      deviceId: 'phone-1',
      name: 'Android',
      endpoint: 'tcp://192.168.1.42:17777',
      vaultIds: ['main'],
      trusted: true,
      encrypted: true,
      online: true,
      pairedAt: '2026-06-14T10:00:00.000Z'
    })
  })

  it('defines the desired user-friendly one-click local sync flow', () => {
    const plan = createUserFriendlySyncPlan({
      platform: SYNC_PLATFORMS.ANDROID,
      vaultIds: ['main'],
      devices: [{ deviceId: 'pc-1', name: 'PC', trusted: true, encrypted: true, online: true }],
      changedFiles: ['Daily.md']
    })

    expect(plan).toMatchObject({
      backend: SYNC_BACKENDS.ELEPHANT_LOCAL,
      externalDependencyFree: true,
      encryptionRequired: true,
      requiresExternalBinary: false,
      securityMode: SYNC_SECURITY_MODES.LOCAL_FIRST_E2EE,
      userMessage: 'Ready to sync securely on your local network.'
    })
    expect(plan.operations).toEqual(SYNC_USER_FRIENDLY_OPERATION_SEQUENCE)
    expect(plan.items.map((item) => item.operation)).toEqual(SYNC_USER_FRIENDLY_OPERATION_SEQUENCE)
  })

  it('never silently overwrites user notes when both devices changed the same path', () => {
    const conflict = classifySyncConflict({ path: 'Daily.md', localChanged: true, remoteChanged: true, sameContent: false })

    expect(conflict).toMatchObject({
      conflict: true,
      path: 'Daily.md',
      resolution: 'preserve-both-and-review',
      errorCode: SYNC_ERROR_CODES.CONFLICT_REQUIRES_REVIEW
    })
    expect(conflict.userMessage).toContain('kept both versions')
  })

  it('returns clear user-facing sync messages for setup, offline and ready states', () => {
    expect(createSyncUserMessage({ cwd: '' })).toEqual({ tone: 'setup', text: 'Open a vault before enabling sync.' })
    expect(createSyncUserMessage({ cwd: '/vault', peers: [{ online: false }] })).toMatchObject({ tone: 'offline' })
    expect(createSyncUserMessage({ cwd: '/vault', peers: [{ online: true }, { online: true }] })).toEqual({
      tone: 'ready',
      text: '2 paired devices ready for secure local sync.'
    })
  })
})
