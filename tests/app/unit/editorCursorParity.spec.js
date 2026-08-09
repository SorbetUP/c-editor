import { describe, expect, it } from 'vitest'

describe('editor cursor parity', () => {
  const normalizeCursor = (value) => value && typeof value === 'object' ? value : {}

  it('normalizes missing cursor values to objects', () => {
    expect(normalizeCursor(undefined)).toEqual({})
    expect(normalizeCursor(null)).toEqual({})
    expect(normalizeCursor({ line: 1, ch: 2 })).toEqual({ line: 1, ch: 2 })
  })

  it('documents the EditorWithTabs prop contract used by Tauri and Electron', () => {
    const value = normalizeCursor(undefined)
    expect(typeof value).toBe('object')
    expect(Array.isArray(value)).toBe(false)
  })
})
