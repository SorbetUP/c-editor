import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useNavigationStore } from '../../../Elephant/frontend/app/stores/navigationStore.js'

describe('navigationStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('deduplicates repeated typeless workspace entries', () => {
    const navigation = useNavigationStore()

    navigation.push({ type: 'wiki' })
    navigation.push({ type: 'wiki' })
    navigation.push({ type: 'wiki' })

    expect(navigation.history).toEqual([{ type: 'wiki' }])
    expect(navigation.index).toBe(0)
  })

  it('keeps distinct entries when the same type points to another path', () => {
    const navigation = useNavigationStore()

    navigation.push({ type: 'wiki', path: '.elephantnote/wiki' })
    navigation.push({ type: 'wiki', path: '.elephantnote/wiki/Cluster' })
    navigation.push({ type: 'wiki', path: '.elephantnote/wiki/Cluster' })

    expect(navigation.history).toEqual([
      { type: 'wiki', path: '.elephantnote/wiki' },
      { type: 'wiki', path: '.elephantnote/wiki/Cluster' }
    ])
    expect(navigation.index).toBe(1)
  })

  it('maps explicit backend activity, success and failure to toolbar states', () => {
    const navigation = useNavigationStore()
    const peer = { endpointId: 'peer-a', name: 'Device A' }

    navigation.applySyncStatus({ peers: [peer], running: true, lastError: '' })
    expect(navigation.syncStatus).toBe('syncing')
    expect(navigation.hasPairedSyncDevice).toBe(true)

    navigation.applySyncStatus({
      peers: [peer],
      running: false,
      lastRunAt: 42,
      lastError: ''
    })
    expect(navigation.syncStatus).toBe('synced')

    navigation.applySyncStatus({
      peers: [peer],
      running: false,
      lastRunAt: 42,
      lastError: 'peer unavailable'
    })
    expect(navigation.syncStatus).toBe('error')
    expect(navigation.syncError).toBe('peer unavailable')
  })

  it('stops a preserved animation when the backend explicitly reports running false', () => {
    const navigation = useNavigationStore()
    navigation.syncStatus = 'syncing'

    navigation.applySyncStatus(
      { peers: [], running: false, lastRunAt: 0, lastError: '' },
      { preserveRunning: true }
    )

    expect(navigation.syncStatus).toBe('idle')
  })
})
