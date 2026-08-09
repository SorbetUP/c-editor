import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const syncClient = vi.hoisted(() => ({
  status: vi.fn(),
  run: vi.fn()
}))

vi.mock('../../../../../../Elephant/frontend/app/services/irohSyncClient.js', () => ({
  irohSyncClient: syncClient
}))

import { useNavigationStore } from '../../../../../../Elephant/frontend/app/stores/navigationStore.js'

const pairedStatus = (overrides = {}) => ({
  peers: [{ endpointId: 'peer-b', name: 'Device B' }],
  running: false,
  lastError: '',
  lastRunAt: '0',
  transferredFiles: 0,
  transferredBytes: 0,
  ...overrides
})

describe('navigation Iroh synchronization state', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    syncClient.status.mockReset()
    syncClient.run.mockReset()
  })

  it('runs the real Iroh client and exposes syncing then success', async () => {
    syncClient.status.mockResolvedValue(pairedStatus())
    let finishRun
    syncClient.run.mockReturnValue(new Promise((resolve) => {
      finishRun = resolve
    }))
    const store = useNavigationStore()

    await store.refreshSyncStatus()
    const running = store.syncWorkspace('')

    expect(store.hasPairedSyncDevice).toBe(true)
    expect(store.syncStatus).toBe('syncing')
    expect(syncClient.run).toHaveBeenCalledTimes(1)

    finishRun(pairedStatus({ lastRunAt: '42' }))
    await running

    expect(store.syncStatus).toBe('synced')
    expect(store.syncError).toBe('')
  })

  it('refuses toolbar synchronization when no peer is paired', async () => {
    syncClient.status.mockResolvedValue(pairedStatus({ peers: [] }))
    const store = useNavigationStore()

    await store.refreshSyncStatus()

    await expect(store.syncWorkspace('')).rejects.toThrow(
      'No paired Iroh device is available for this vault.'
    )
    expect(syncClient.run).not.toHaveBeenCalled()
    expect(store.syncStatus).toBe('error')
  })

  it('shows a backend failure instead of a false success indicator', async () => {
    syncClient.status.mockResolvedValue(pairedStatus())
    syncClient.run.mockResolvedValue(pairedStatus({
      lastError: 'remote manifest verification failed',
      lastRunAt: '42'
    }))
    const store = useNavigationStore()

    await store.refreshSyncStatus()

    await expect(store.syncWorkspace('')).rejects.toThrow(
      'remote manifest verification failed'
    )
    expect(store.syncStatus).toBe('error')
    expect(store.syncError).toBe('remote manifest verification failed')
  })

  it('stops an animation when an explicit completed status is received', () => {
    const store = useNavigationStore()
    store.syncStatus = 'syncing'

    store.applySyncStatus(pairedStatus({ lastRunAt: '42' }), {
      preserveRunning: true
    })

    expect(store.syncStatus).toBe('synced')
  })
})
