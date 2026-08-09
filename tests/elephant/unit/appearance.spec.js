import { describe, expect, it } from 'vitest'
import {
  ELEPHANTNOTE_DEFAULT_THEME,
  ELEPHANTNOTE_THEME_FAMILIES,
  VAULT_ICON_OPTIONS,
  getOppositeThemeVariant,
  getThemeFamily,
  getThemeLabel,
  getThemeMode,
  getThemeTokens,
  getThemeVariant,
  isVaultIconName,
  normalizeThemeId,
  normalizeVaultIcon
} from 'common/elephantnote/appearance'

describe('ElephantNote appearance contract', () => {
  it('normalizes theme ids and exposes shell/editor tokens', () => {
    expect(normalizeThemeId('dark')).toBe('dark')
    expect(normalizeThemeId('apple-dark')).toBe('apple-dark')
    expect(normalizeThemeId('unknown')).toBe(ELEPHANTNOTE_DEFAULT_THEME)
    expect(getThemeTokens('dark')).toMatchObject({
      '--en-bg': '#0f141d',
      '--themeColor': '#5ea1ff',
      '--iconColor': '#98a3b6'
    })
    expect(getThemeTokens('apple-light')).toMatchObject({
      '--en-bg': '#f5f5f7',
      '--themeColor': '#007aff',
      '--codeBlockBgColor': '#f0f0f3'
    })
    expect(getThemeTokens('missing')).toMatchObject({
      '--en-bg': '#f7f9fc',
      '--themeColor': '#2563eb'
    })
  })

  it('keeps graphic theme families paired across light and dark variants', () => {
    expect(ELEPHANTNOTE_THEME_FAMILIES.length).toBeGreaterThanOrEqual(6)
    expect(getThemeMode('apple-dark')).toBe('dark')
    expect(getThemeMode('apple-light')).toBe('light')
    expect(getThemeFamily('apple-dark').id).toBe('apple')
    expect(getThemeLabel('apple-dark')).toBe('Apple Dark')
    expect(getThemeVariant('apple', 'light')).toBe('apple-light')
    expect(getThemeVariant('apple', 'dark')).toBe('apple-dark')
    expect(getOppositeThemeVariant('apple-dark')).toBe('apple-light')
    expect(getOppositeThemeVariant('apple-light')).toBe('apple-dark')
  })

  it('keeps vault icon ids portable while preserving compatibility aliases', () => {
    expect(VAULT_ICON_OPTIONS.map((option) => option.name)).toEqual([
      'home',
      'file-text',
      'database',
      'graduation-cap',
      'landmark',
      'rocket',
      'star',
      'terminal',
      'workflow'
    ])
    expect(normalizeVaultIcon('book')).toBe('file-text')
    expect(normalizeVaultIcon(' rocket ')).toBe('rocket')
    expect(normalizeVaultIcon('unknown')).toBe('')
    expect(isVaultIconName('rocket')).toBe(true)
    expect(isVaultIconName('book')).toBe(false)
  })
})
