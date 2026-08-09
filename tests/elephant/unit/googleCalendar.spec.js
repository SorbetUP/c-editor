import {
  calendarEventToGoogleEvent,
  googleEventToCalendarEvent,
  normalizeGoogleCalendarConfig
} from 'common/elephantnote/googleCalendar'

describe('Google Calendar bridge', () => {
  it('normalizes OAuth config without leaking optional fields into runtime assumptions', () => {
    expect(normalizeGoogleCalendarConfig({ enabled: true, calendarId: '' })).toMatchObject({
      enabled: true,
      calendarId: 'primary'
    })
  })

  it('maps Google events to local calendar events and back', () => {
    const local = googleEventToCalendarEvent({
      id: 'evt-1',
      summary: 'Planning',
      start: { dateTime: '2026-05-25T09:00:00Z' },
      end: { dateTime: '2026-05-25T10:00:00Z' },
      location: 'Office'
    }, 'primary')

    expect(local).toMatchObject({
      id: 'evt-1',
      title: 'Planning',
      startsAt: '2026-05-25T09:00:00Z',
      source: 'google-calendar'
    })
    expect(calendarEventToGoogleEvent(local)).toMatchObject({
      summary: 'Planning',
      start: { dateTime: '2026-05-25T09:00:00Z' }
    })
  })
})
