import { existsSync } from 'node:fs'
import { readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

import { createBundledMuyaRustEngine } from '../../../../../Elephant/frontend/src/muya/lib/rust/wasmFactory'

const wasmPath = resolve(
  process.cwd(),
  'Elephant/frontend/src/muya/lib/rust/generated/muya_wasm_bg.wasm'
)
const generatedWasmTest = existsSync(wasmPath) ? it : it.skip

describe('createBundledMuyaRustEngine', () => {
  generatedWasmTest('creates the production Rust editor from the generated WASM bundle', async () => {
    const bytes = await readFile(wasmPath)
    const wasm = new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength)
    const engine = await createBundledMuyaRustEngine('alpha', wasm)
    expect(engine).toBeTruthy()
    expect(typeof engine.handle_json).toBe('function')
    expect(typeof engine.snapshot_json).toBe('function')
    const snapshot = JSON.parse(await engine.snapshot_json())
    expect(snapshot.type).toBe('snapshot')
    expect(snapshot.payload.markdown).toBe('alpha\n')
  })
})
