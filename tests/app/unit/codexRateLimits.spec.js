import { describe, expect, it } from 'vitest'
import { buildCodexRateLimitRows, buildCodexResetCredits, formatCodexWindowLabel, getCodexResetAvailableCount } from '../../../Elephant/frontend/app/components/settings/codexRateLimits'

describe('Codex rate-limit display', () => {
  it('shows both the short and weekly subscription windows', () => {
    const rows = buildCodexRateLimitRows({
      rateLimitsByLimitId: {
        codex: {
          primary: { usedPercent: 41, windowDurationMins: 300, resetsAt: 1000 },
          secondary: { usedPercent: 72, windowDurationMins: 10080, resetsAt: 2000 }
        }
      }
    })
    expect(rows).toHaveLength(2)
    expect(rows.map((row) => row.label)).toEqual(['5-hour limit', 'Weekly limit'])
    expect(rows.map((row) => row.remainingPercent)).toEqual([59, 28])
  })

  it('uses the backward-compatible snapshot when no bucket map exists', () => {
    const rows = buildCodexRateLimitRows({ rateLimits: { primary: { usedPercent: 10, windowDurationMins: 60 } } })
    expect(rows).toMatchObject([{ label: '1-hour limit', usedPercent: 10, remainingPercent: 90 }])
  })

  it('does not invent weekly labels when the server omits the duration', () => {
    expect(formatCodexWindowLabel({}, true)).toBe('Secondary usage limit')
  })

  it('normalizes selectable reset credits and preserves the server count', () => {
    const payload = {
      rateLimitResetCredits: {
        availableCount: 4,
        credits: [
          { id: 'late', status: 'available', resetType: 'codexRateLimits', expiresAt: 2000, title: 'Full reset' },
          { id: 'used', status: 'redeemed', resetType: 'codexRateLimits', expiresAt: 500 },
          { id: 'early', status: 'available', resetType: 'codexRateLimits', expiresAt: 1000 }
        ]
      }
    }
    expect(getCodexResetAvailableCount(payload)).toBe(4)
    expect(buildCodexResetCredits(payload).map((credit) => credit.id)).toEqual(['early', 'late'])
  })
})
