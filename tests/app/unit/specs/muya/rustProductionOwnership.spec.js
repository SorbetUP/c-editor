import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('Muya Rust editor production ownership', () => {
  it('uses the Rust runtime at the real editor-with-tabs boundary', () => {
    const tabs = read('Elephant/frontend/src/renderer/src/components/editorWithTabs/index.vue')
    const runtime = read('Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditor.vue')
    expect(tabs).toContain("import RuntimeEditor from './runtimeEditor.vue'")
    expect(tabs).not.toContain("import Editor from './editor.vue'")
    expect(runtime).toContain("import { RustMuyaRuntimeEditor } from '@/muya'")
    expect(runtime).toContain("import { createRustEditorRuntimeBinding } from '@/muya/editorRuntimeResource'")
    expect(runtime).toContain('<RustMuyaRuntimeEditor')
    expect(runtime).not.toContain("import Muya from 'muya/lib'")
    expect(runtime).not.toContain('new Muya(')
    expect(runtime).not.toContain('CodeMirror')
  })

  it('builds and aliases the Rust WASM runtime in production web commands', () => {
    const vite = read('vite.tauri.config.mjs')
    const packageJson = JSON.parse(read('package.json'))
    expect(vite).toContain("'muya-rust-wasm-bundle': muyaWasmGenerated")
    expect(packageJson.scripts['tauri:web:build']).toContain('pnpm muya:wasm:build')
    expect(packageJson.scripts['tauri:web:dev']).toContain('pnpm muya:wasm:build')
  })
})
