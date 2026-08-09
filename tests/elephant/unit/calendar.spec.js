import { describe, expect, it } from 'vitest'
import {
  bucketCalendarEvents,
  mergeCalendarEvents,
  normalizeCalendarDate,
  parseIcsCalendar
} from 'common/elephantnote/calendar'

describe('ElephantNote calendar import', () => {
  it('normalizes Google Calendar ICS date values', () => {
    expect(normalizeCalendarDate('20260525')).toBe('2026-05-25')
    expect(normalizeCalendarDate('20260525T083000Z')).toBe('2026-05-25T08:30:00Z')
  })

  it('parses Google Calendar ICS events', () => {
    const events = parseIcsCalendar([
      'BEGIN:VCALENDAR',
      'BEGIN:VEVENT',
      'UID:event-1@example.com',
      'DTSTART:20260525T083000Z',
      'DTEND:20260525T090000Z',
      'SUMMARY:Planning',
      'LOCATION:Office',
      'DESCRIPTION:Discuss roadmap\\nWith citations',
      'END:VEVENT',
      'END:VCALENDAR'
    ].join('\n'))

    expect(events).toEqual([
      {
        id: 'event-1@example.com',
        title: 'Planning',
        startsAt: '2026-05-25T08:30:00Z',
        endsAt: '2026-05-25T09:00:00Z',
        location: 'Office',
        description: 'Discuss roadmap\nWith citations',
        source: 'google-calendar',
        calendarId: 'default'
      }
    ])
  })

  it('merges imported events by stable id', () => {
    expect(mergeCalendarEvents([
      { id: 'a', title: 'Old', startsAt: '2026-05-25T08:00:00Z' }
    ], [
      { id: 'a', title: 'New', startsAt: '2026-05-25T08:00:00Z' },
      { id: 'b', title: 'Second', startsAt: '2026-05-26T08:00:00Z' }
    ])).toMatchObject([
      { id: 'a', title: 'New' },
      { id: 'b', title: 'Second' }
    ])
  })

  it('buckets events by start date', () => {
    expect(bucketCalendarEvents([
      { id: 'b', title: 'B', startsAt: '2026-05-26T08:00:00Z' },
      { id: 'a', title: 'A', startsAt: '2026-05-25T08:00:00Z' }
    ])).toEqual([
      { date: '2026-05-25', events: [{ id: 'a', title: 'A', startsAt: '2026-05-25T08:00:00Z', endsAt: '', location: '', description: '', source: 'local', calendarId: 'default' }] },
      { date: '2026-05-26', events: [{ id: 'b', title: 'B', startsAt: '2026-05-26T08:00:00Z', endsAt: '', location: '', description: '', source: 'local', calendarId: 'default' }] }
    ])
  })
})
