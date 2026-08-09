import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import { trustedAddonModulePathForTest } from '../../../../Elephant/frontend/src/renderer/src/addons/trustedAddonModuleLoader'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('trusted addon module graphs', () => {
  it('normalizes confined JavaScript module paths', () => {
    expect(trustedAddonModulePathForTest('runtime/provider')).toBe('runtime/provider.js')
    expect(trustedAddonModulePathForTest('./main.js')).toBe('main.js')
    expect(() => trustedAddonModulePathForTest('../main.js')).toThrow(/escapes/)
    expect(() => trustedAddonModulePathForTest('manifest.json')).toThrow(/JavaScript/)
  })

  it('reads only installed package-contained modules', () => {
    const rust = read('Elephant/backend/tauri/src/addons.rs')
    const lib = read('Elephant/backend/tauri/src/lib_min.rs')
    expect(rust).toContain('pub fn tauri_addons_read_module')
    expect(rust).toContain('Addon module escapes its package directory')
    expect(rust).toContain('require_installed(&registry, &addon_id)')
    expect(lib).toContain('addons::tauri_addons_read_module')
  })

  it('loads and revokes the complete trusted package graph', () => {
    const runtime = read('Elephant/frontend/src/renderer/src/addons/trustedAddonRuntime.js')
    expect(runtime).toContain('loadTrustedAddonModuleGraph')
    expect(runtime).toContain('tauri_addons_read_module')
    expect(runtime).toContain('revokeTrustedAddonModuleGraph(this.moduleUrls)')
    expect(runtime).not.toContain("const entry = await invoke('tauri_addons_read_entry'")
  })
})
