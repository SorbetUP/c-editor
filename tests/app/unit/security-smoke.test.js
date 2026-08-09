import { describe, expect, it } from 'vitest'

import {
  assertPathInsideVault,
  isAllowedSiteFile,
  isIgnoredForSite
} from '../../../Elephant/backend/js/sitePreview/pathSafety.js'
import {
  isIgnoredVaultEntry
} from '../../../Elephant/shared/workspace.js'

describe('preview and vault file safety', () => {
  it('keeps generated previews away from hidden app and dependency folders', () => {
    expect(isIgnoredForSite('.elephantnote/index.json')).toBe(true)
    expect(isIgnoredForSite('node_modules/pkg/index.js')).toBe(true)
    expect(isIgnoredForSite('Notes/index.md')).toBe(false)
  })

  it('limits static site files to explicitly supported extensions', () => {
    expect(isAllowedSiteFile('Notes/index.md')).toBe(true)
    expect(isAllowedSiteFile('assets/photo.webp')).toBe(true)
    expect(isAllowedSiteFile('build/scripts/install.sh')).toBe(false)
    expect(isAllowedSiteFile('bin/native')).toBe(false)
  })

  it('rejects preview source checks when no active vault root is available', () => {
    expect(() => assertPathInsideVault('', process.cwd())).toThrow(/outside the active vault/)
    expect(() => assertPathInsideVault('   ', process.cwd())).toThrow(/outside the active vault/)
  })

  it('hides internal vault folders from user-facing listings', () => {
    expect(isIgnoredVaultEntry('.git')).toBe(true)
    expect(isIgnoredVaultEntry('.elephantnote')).toBe(true)
    expect(isIgnoredVaultEntry('Projects')).toBe(false)
  })
})
