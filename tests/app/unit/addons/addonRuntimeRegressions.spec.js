import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  isMissingNativeServiceError,
  reconcileOfficialAddonRecords
} from '../../../../Elephant/frontend/src/renderer/src/addons/externalAddonRuntime.js'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('addon runtime regression repairs', () => {
  it('restores official provenance for historical registry records', () => {
    const [record] = reconcileOfficialAddonRecords(
      [{ source: 'external', manifest: { id: 'elephant.wiki' } }],
      [{ id: 'elephant.wiki', official: true }]
    )

    expect(record.source).toBe('official')
    expect(record.official).toBe(true)
    expect(record.manifest.source).toBe('official')
    expect(record.manifest.official).toBe(true)
  })

  it('never promotes a package absent from the verified official catalogue', () => {
    const [record] = reconcileOfficialAddonRecords(
      [{ source: 'external', manifest: { id: 'elephant.fake' } }],
      [{ id: 'elephant.wiki', official: true }]
    )

    expect(record.source).toBe('external')
    expect(record.official).toBeUndefined()
  })

  it('repairs only the exact missing native executable failure', () => {
    expect(isMissingNativeServiceError(new Error(
      'Addon service executable is unavailable for elephant.sync: No such file or directory (os error 2)'
    ))).toBe(true)
    expect(isMissingNativeServiceError(new Error(
      'Addon sidecar executable is unavailable for elephant.ai-ocr: No such file or directory (os error 2)'
    ))).toBe(true)
    expect(isMissingNativeServiceError(new Error('Addon activation failed'))).toBe(false)

    const runtime = read('Elephant/frontend/src/renderer/src/addons/externalAddonRuntime.js')
    expect(runtime).toContain("officialInstall: (addonId) => invoke('tauri_official_addons_catalog_install', { addonId })")
    expect(runtime).toContain('await this.repairOfficialPackage(record)')
    expect(runtime).toContain('official addon native package repaired')
  })

  it('materializes missing native services during addon synchronization', () => {
    const sync = read('build/scripts/sync-elephant-addons.mjs')
    const builder = read('build/scripts/build-physical-addon.mjs')

    expect(sync).toContain('materializeNativeServices()')
    expect(sync).toContain('ELEPHANT_ADDON_MATERIALIZE_SOURCE')
    expect(sync).toContain('Native addon service was not materialized')
    expect(sync).toContain('native service materialization skipped explicitly')
    expect(sync).not.toContain("process.env.CI === 'true'")
    expect(builder).toContain('ELEPHANT_ADDON_MATERIALIZE_ONLY')
    expect(builder).toContain('materialized=${sourceSidecar}')
  })

  it('keeps native service stderr observable for startup failures', () => {
    const services = read('Elephant/backend/tauri/src/addon_services.rs')

    expect(services).toContain('.stderr(Stdio::piped())')
    expect(services).toContain('spawn_service_log_reader(addon_id, stderr)')
    expect(services).toContain('[addon-service:{addon_id}]')
    expect(services).not.toContain('.stderr(Stdio::null())')
  })

  it('bounds large vault listings with an explicit safety limit', () => {
    const source = read('Elephant/backend/tauri/src/addon_note_access.rs')

    expect(source).toContain('MAX_LISTED_NOTES: usize = 1_000')
    expect(source).toContain('Addon note listing exceeded the maximum of')
    expect(source).toContain('MAX_DIRECTORY_DEPTH')
    expect(source).toContain('MAX_NOTE_BYTES')
    expect(source).toContain('read_enabled_addon')
  })
})
