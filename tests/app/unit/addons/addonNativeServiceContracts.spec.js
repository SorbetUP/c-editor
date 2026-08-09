import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('persistent native addon services', () => {
  it('exposes status, start, call and stop to trusted packages', () => {
    const runtime = read('Elephant/frontend/src/renderer/src/addons/trustedAddonRuntime.js')
    for (const command of [
      'tauri_addons_service_status',
      'tauri_addons_service_start',
      'tauri_addons_service_call',
      'tauri_addons_service_stop'
    ]) {
      expect(runtime).toContain(command)
    }
    expect(runtime).toContain('service: Object.freeze')
  })

  it('keeps the native host generic and package-owned', () => {
    const rust = read('Elephant/backend/tauri/src/addon_services.rs')
    expect(rust).toContain('elephant-addon-service-v1')
    expect(rust).toContain('ELEPHANT_ADDON_PACKAGE_DIR')
    expect(rust).toContain('ELEPHANT_ADDON_DATA_DIR')
    expect(rust).toContain('ELEPHANT_VAULT_DIR')
    expect(rust).toContain('kill_on_drop(true)')
    expect(rust).not.toContain('iroh::')
  })
})
