import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

const workflowDirectory = path.join(root, '.github/workflows')

describe('develop_next final architecture', () => {
  it('contains no temporary migration or diagnostic workflows', () => {
    const temporary = fs.readdirSync(workflowDirectory)
      .filter((name) => name.startsWith('develop-next-'))
    expect(temporary).toEqual([])
  })

  it('owns Android vault access in the native mobile plugin', () => {
    expect(fs.existsSync(path.join(
      root,
      'Elephant/backend/tauri-plugin-elephant-android-vault/Cargo.toml'
    ))).toBe(true)
    expect(read('Elephant/backend/tauri/Cargo.toml'))
      .toContain('tauri-plugin-elephant-android-vault')
    expect(read('Elephant/backend/tauri/src/lib_min.rs'))
      .toContain('tauri_plugin_elephant_android_vault::init()')
  })

  it('keeps production image handling outside the legacy Muya package', () => {
    const runtimeImages = read(
      'Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditorImages.js'
    )
    expect(runtimeImages).not.toContain("from 'muya")
    expect(runtimeImages).not.toContain('muya/lib')
    expect(runtimeImages).toContain('isRuntimeImageUrl')
    expect(runtimeImages).toContain('checkRuntimeImageContentType')
  })

  it('keeps the Rust engine neutral and the Muya compatibility facade explicit', () => {
    expect(fs.existsSync(path.join(
      root,
      'Elephant/frontend/src/renderer/src/editor-rust/wasmFactory.js'
    ))).toBe(true)
    const component = read(
      'Elephant/frontend/src/renderer/src/muya/RustMuyaRuntimeEditor.vue'
    )
    expect(component).toContain('../editor-rust/runtime')
    expect(component).toContain('completeMuyaRustAdapter.js.wrapper.js')
    expect(read('Elephant/frontend/src/renderer/src/muya/completeMuyaRustAdapter.js'))
      .toContain('extends RustOwnedMuya')
  })
})
