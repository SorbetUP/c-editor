import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { defineConfig } from 'vite'

const root = resolve(import.meta.dirname)
const inlineWasmModuleId = 'muya-rust-wasm-inline'
const inlineWasmVirtualId = `\0${inlineWasmModuleId}`

const inlineMuyaWasmPlugin = {
  name: 'inline-muya-wasm',
  resolveId(source) {
    return source === inlineWasmModuleId ? inlineWasmVirtualId : undefined
  },
  load(id) {
    if (id !== inlineWasmVirtualId) return undefined
    const wasmPath = resolve(
      root,
      'Elephant/frontend/src/muya/lib/rust/generated/muya_wasm_bg.wasm'
    )
    const encoded = readFileSync(wasmPath).toString('base64')
    return `const encoded = '${encoded}'
const bytes = Uint8Array.from(atob(encoded), (character) => character.charCodeAt(0))
export default bytes`
  }
}

export default defineConfig({
  plugins: [inlineMuyaWasmPlugin],
  root: resolve(root, 'Elephant/frontend/src/muya/native'),
  base: './',
  build: {
    outDir: resolve(root, 'build/out/muya-native'),
    emptyOutDir: true,
    assetsDir: 'assets',
    target: 'es2022',
    minify: false,
    sourcemap: false
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify('production')
  },
  resolve: {
    alias: {
      muya: resolve(root, 'Elephant/frontend/src/muya'),
      'common/elephantnote': resolve(root, 'Elephant/shared'),
      'muya-rust-wasm-bundle': resolve(
        root,
        'Elephant/frontend/src/muya/lib/rust/generated/muya_wasm.js'
      )
    },
    extensions: ['.mjs', '.js', '.json']
  }
})
