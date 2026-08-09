import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('develop_next mobile boundary', () => {
  it('owns Android document-tree access without restoring optional product backends', () => {
    const cargo = read('Elephant/backend/tauri/Cargo.toml')
    const lib = read('Elephant/backend/tauri/src/lib_min.rs')
    expect(cargo).toContain('tauri-plugin-elephant-android-vault')
    expect(cargo).not.toContain('iroh =')
    expect(cargo).not.toContain('iroh-mdns-address-lookup')
    expect(lib).toContain('android_vault_commands::tauri_android_vault_pick')
    expect(lib).not.toContain('IrohSyncState')
    for (const absent of ['knowledge.rs', 'knowledge_wikis.rs', 'ocr.rs', 'local_llama_runtime.rs']) {
      expect(fs.existsSync(path.join(root, 'Elephant/backend/tauri/src', absent))).toBe(false)
    }
  })

  it('uses the Rust editor resource for mobile note sharing', () => {
    const runtime = read('Elephant/frontend/src/renderer/src/platform/mobileEditorRuntime.js')
    expect(runtime).toContain("get?.('editor.runtime')")
    expect(runtime).not.toContain('.ag-editor')
    expect(runtime.toLowerCase()).not.toContain('muya')
  })
})
