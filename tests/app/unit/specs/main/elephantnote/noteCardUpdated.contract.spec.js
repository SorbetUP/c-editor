import { describe, expect, it } from 'vitest'
import { formatShortDate } from '@/elephantnote/services/markdownMetaService'
import { getNoteCardUpdatedLabel } from '@/elephantnote/utils/noteCardView'

describe('note card updated label contract', () => {
  it.each(['2026-07-10', '2026-07-10T12:00:00.000Z', '1783684800000', '1783684800'])(
    'delegates supported date input %s to the canonical formatter',
    (updatedAt) => {
      expect(getNoteCardUpdatedLabel({ updatedAt })).toBe(formatShortDate(updatedAt))
    }
  )

  it.each([{}, null, { updatedAt: '' }, { updatedAt: 'not-a-date' }])(
    'returns an empty label for unsupported metadata %#',
    (entry) => {
      expect(getNoteCardUpdatedLabel(entry)).toBe('')
    }
  )
})
