import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('neutral Rust editor namespace', () => {
  it('keeps the Rust engine neutral while restoring the Muya compatibility facade', () => {
    const runtime = read('Elephant/frontend/src/renderer/src/editor-rust/runtime.js')
    const bridge = read('Elephant/frontend/src/renderer/src/editor-rust/bridge.js')
    const wasmFactory = read('Elephant/frontend/src/renderer/src/editor-rust/wasmFactory.js')
    const component = read('Elephant/frontend/src/renderer/src/muya/RustMuyaRuntimeEditor.vue')
    expect(runtime).toContain('ElephantRustBridge')
    expect(runtime).not.toContain('MuyaRustBridge')
    expect(bridge).toContain('Elephant Rust')
    expect(wasmFactory).toContain('muya-rust-wasm-bundle')
    expect(wasmFactory).not.toContain('editorState-rust-wasm-bundle')
    expect(component).toContain('../editor-rust/runtime')
    expect(component).toContain('completeMuyaRustAdapter.js.wrapper.js')
    expect(read('Elephant/frontend/src/renderer/src/muya/completeMuyaRustAdapter.js'))
      .toContain('extends RustOwnedMuya')
  })
})
