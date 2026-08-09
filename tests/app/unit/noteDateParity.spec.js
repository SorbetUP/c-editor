import { describe, expect, it } from 'vitest'

import { formatShortDate } from '../../../Elephant/frontend/app/services/markdownMetaService.js'

describe('note date parity', () => {
  it('does not throw for missing or invalid dates', () => {
    expect(formatShortDate(null)).toBe('')
    expect(formatShortDate(undefined)).toBe('')
    expect(formatShortDate('')).toBe('')
    expect(formatShortDate('invalid date value')).toBe('')
  })

  it('formats valid dates deterministically', () => {
    expect(formatShortDate('2026-06-22T10:00:00.000Z')).toMatch(/^2026-06-22$/)
  })
})
