import { describe, expect, it } from 'vitest'
import {
  assertPathInsideVault,
  isIgnoredForSite
} from '../../../../../../Elephant/backend/js/sitePreview/pathSafety.js'

describe('preview path safety contract', () => {
  it('rejects empty vault roots before resolving paths against an implicit cwd', () => {
    expect(() => assertPathInsideVault('', '/tmp/vault')).toThrow(/outside the active vault/)
    expect(() => assertPathInsideVault('   ', '/tmp/vault')).toThrow(/outside the active vault/)
  })

  it('rejects target folders outside the active vault', () => {
    expect(() => assertPathInsideVault('/vault/root', '/vault/root/Notes')).not.toThrow()
    expect(() => assertPathInsideVault('/vault/root', '/vault/other')).toThrow(/outside the active vault/)
  })

  it('keeps internal metadata folders out of site preview traversal', () => {
    expect(isIgnoredForSite('.elephantnote/sync/sync-config.json')).toBe(true)
    expect(isIgnoredForSite('Notes/Project.md')).toBe(false)
  })
})
