import { describe, expect, it } from 'vitest'
import {
  SYNC_OPERATIONS,
  createDefaultSyncPlan
} from '../../../../../../Elephant/shared/sync.js'
import {
  ELEPHANTNOTE_API_ACTIONS as API,
  validateApiPayload
} from '../../../../../../Elephant/shared/apiContracts.js'

const operations = (plan) => plan.map((item) => item.operation)

describe('createDefaultSyncPlan legacy helper', () => {
  it('keeps the compatibility local snapshot plan when no explicit operation is provided', () => {
    expect(operations(createDefaultSyncPlan({}))).toEqual([
      SYNC_OPERATIONS.INIT,
      SYNC_OPERATIONS.SNAPSHOT
    ])
  })

  it('is no longer exposed through the core API contract', () => {
    const payload = { operations: ['init', 'pull'], pull: { remoteName: 'origin' } }

    expect(API.SYNC_PLAN).toBeUndefined()
    expect(API.SYNC_RUN).toBeUndefined()
    expect(validateApiPayload('sync.plan', payload)).toBe(payload)
  })

  it('ignores invalid explicit operation payload shapes instead of throwing', () => {
    expect(operations(createDefaultSyncPlan({ operations: 'pull' }))).toEqual([
      SYNC_OPERATIONS.INIT,
      SYNC_OPERATIONS.SNAPSHOT
    ])
    expect(operations(createDefaultSyncPlan({ operations: { 0: 'pull' } }))).toEqual([
      SYNC_OPERATIONS.INIT,
      SYNC_OPERATIONS.SNAPSHOT
    ])
  })

  it('adds push when a caller explicitly asks for a remote upload', () => {
    const plan = createDefaultSyncPlan({
      init: { remote: '/git/elephantnote.git' },
      snapshot: { message: 'push note' },
      push: {}
    })

    expect(operations(plan)).toEqual([
      SYNC_OPERATIONS.INIT,
      SYNC_OPERATIONS.SNAPSHOT,
      SYNC_OPERATIONS.PUSH
    ])
    expect(plan[0].payload.remote).toBe('/git/elephantnote.git')
    expect(plan[1].payload.message).toBe('push note')
  })

  it('can pull into a second device without creating a local snapshot first', () => {
    const plan = createDefaultSyncPlan({
      init: { remote: '/git/elephantnote.git' },
      pull: {}
    })

    expect(operations(plan)).toEqual([
      SYNC_OPERATIONS.INIT,
      SYNC_OPERATIONS.PULL
    ])
  })

  it('supports explicit operation lists for retained migration fixtures', () => {
    const plan = createDefaultSyncPlan({
      operations: ['init', 'pull', 'snapshot', 'push'],
      snapshot: { message: 'manual order' }
    })

    expect(operations(plan)).toEqual([
      SYNC_OPERATIONS.INIT,
      SYNC_OPERATIONS.PULL,
      SYNC_OPERATIONS.SNAPSHOT,
      SYNC_OPERATIONS.PUSH
    ])
    expect(plan[2].payload.message).toBe('manual order')
  })
})
