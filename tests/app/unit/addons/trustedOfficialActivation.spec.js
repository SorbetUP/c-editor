import { describe, expect, it } from 'vitest'
import {
  isOfficialTrustedRecord,
  shouldEnforceCommunityTrust
} from '../../../../Elephant/frontend/src/renderer/src/addons/trustedAddonRuntime.js'

describe('trusted official addon activation policy', () => {
  it('recognizes every persisted official provenance representation', () => {
    expect(isOfficialTrustedRecord({ source: 'official' })).toBe(true)
    expect(isOfficialTrustedRecord({ official: true })).toBe(true)
    expect(isOfficialTrustedRecord({ manifest: { source: 'official' } })).toBe(true)
    expect(isOfficialTrustedRecord({ manifest: { official: true } })).toBe(true)
  })

  it('enforces community consent, safe mode and hash approval for third-party trusted packages only', () => {
    expect(shouldEnforceCommunityTrust({
      source: 'external',
      official: false,
      manifest: { source: 'external', official: false }
    })).toBe(true)

    expect(shouldEnforceCommunityTrust({
      source: 'external',
      manifest: { source: 'external', official: true }
    })).toBe(false)
  })
})
