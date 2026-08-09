import { describe, expect, it } from 'vitest'
import {
  INVITE_EXTENSION,
  INVITE_FILE_ACCEPT,
  buildSyncInviteFileName,
  parseSyncInvite,
  unwrapSyncInvite,
  validateSyncInvitePayload
} from '../../../../../../addons/official/sync/invite.js'

const invitation = (overrides = {}) => ({
  protocol: 'elephantnote-iroh-sync-v1',
  inviteId: 'invite-123',
  inviteToken: 'secret-token',
  endpointAddr: 'iroh://endpoint',
  folderId: 'folder-abc',
  expiresAt: 2_000_000_000,
  ...overrides
})

describe('package-owned Sync invitations', () => {
  it('validates a structured invitation without changing its payload', () => {
    const payload = JSON.stringify(invitation())
    expect(validateSyncInvitePayload(payload, 1_900_000_000)).toEqual({
      kind: 'structured',
      payload,
      value: invitation()
    })
  })

  it('rejects expired and incomplete structured invitations', () => {
    expect(() => validateSyncInvitePayload(JSON.stringify(invitation({ expiresAt: 10 })), 11))
      .toThrow('expired')
    expect(() => validateSyncInvitePayload(JSON.stringify(invitation({ inviteToken: '' })), 1))
      .toThrow('incomplete')
  })

  it('rejects JSON from another protocol', () => {
    expect(() => validateSyncInvitePayload(JSON.stringify(invitation({ protocol: 'other-product' })), 1))
      .toThrow('not an Elephant Sync invitation')
  })

  it('accepts a sufficiently long manual code', () => {
    expect(validateSyncInvitePayload('manual-code-123456')).toEqual({
      kind: 'manual',
      payload: 'manual-code-123456'
    })
  })

  it('unwraps base64url Elephant Sync links', () => {
    const payload = JSON.stringify(invitation())
    const encoded = Buffer.from(payload).toString('base64url')
    const link = `elephantnote://sync/invite?payload=${encoded}`
    expect(unwrapSyncInvite(link)).toBe(payload)
    expect(parseSyncInvite(link)).toEqual(invitation())
  })

  it('builds a portable invitation filename', () => {
    const fileName = buildSyncInviteFileName('Mon coffre / Démo', 'invite:123')
    expect(fileName).toBe(`Elephant-Mon-coffre-Demo-invite-123${INVITE_EXTENSION}`)
    expect(INVITE_FILE_ACCEPT).toContain(INVITE_EXTENSION)
  })
})
