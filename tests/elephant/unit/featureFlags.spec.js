import { describe, expect, it } from 'vitest'
import {
  ELEPHANTNOTE_FEATURE_FLAGS,
  normalizeFeatureFlags,
  setFeatureFlag
} from 'common/elephantnote/featureFlags'

describe('ElephantNote feature flags', () => {
  it('normalizes missing and unknown feature values', () => {
    expect(normalizeFeatureFlags({ gitSync: true, unknown: true })).toEqual({
      ...ELEPHANTNOTE_FEATURE_FLAGS,
      gitSync: true
    })
  })

  it('rejects unknown flags', () => {
    expect(() => setFeatureFlag({}, 'missing', true)).toThrow('Unknown ElephantNote feature flag')
  })
})
