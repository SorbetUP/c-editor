import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  mergeEvents,
  normalizeEvents,
  parseIcs
} from '../../../../addons/official/calendar/main.js'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('Calendar physical package ownership', () => {
  it('parses timed, all-day, escaped and folded VEVENT fields', () => {
    const events = parseIcs([
      'BEGIN:VCALENDAR',
      'BEGIN:VEVENT',
      'UID:event-1',
      'DTSTART:20260714T090000Z',
      'DTEND:20260714T100000Z',
      'SUMMARY:Architecture review',
      'DESCRIPTION:First line\\nSecond line with a long ',
      ' continuation',
      'LOCATION:Paris\\, France',
      'END:VEVENT',
      'BEGIN:VEVENT',
      'UID:event-2',
      'DTSTART;VALUE=DATE:20260715',
      'SUMMARY:Release day',
      'END:VEVENT',
      'END:VCALENDAR'
    ].join('\r\n'))

    expect(events).toHaveLength(2)
    expect(events[0]).toMatchObject({
      id: 'event-1',
      title: 'Architecture review',
      startsAt: '2026-07-14T09:00:00Z',
      endsAt: '2026-07-14T10:00:00Z',
      location: 'Paris, France'
    })
    expect(events[0].description).toContain('First line\nSecond line with a long continuation')
    expect(events[1].startsAt).toBe('2026-07-15')
    expect(events[1].endsAt).toBe('2026-07-15')
  })

  it('normalizes and deterministically replaces duplicate UIDs', () => {
    const current = normalizeEvents([{ id: 'same', title: 'Old', startsAt: '2026-07-14' }])
    const merged = mergeEvents(current, [{ id: 'same', title: 'New', startsAt: '2026-07-14' }])

    expect(merged).toHaveLength(1)
    expect(merged[0].title).toBe('New')
  })

  it('publishes a vault-scoped resource without removed calendar commands', () => {
    const source = read('addons/official/calendar/main.js')
    const manifest = JSON.parse(read('addons/official/calendar/manifest.json'))

    expect(manifest.version).toBe('1.3.0')
    expect(manifest.permissions.storage).toBe(true)
    expect(manifest.permissions.network).toBeUndefined()
    expect(source).toContain("const PROVIDER_RESOURCE = 'calendar.provider'")
    expect(source).toContain('apiVersion: 1')
    expect(source).toContain('EVENTS_KEY_PREFIX')
    expect(source).toContain('activeVaultId()')
    expect(source).toContain('importIcs:')
    expect(source).not.toContain('tauri_calendar_')
    expect(source).not.toContain('syncGoogle')
  })
})
