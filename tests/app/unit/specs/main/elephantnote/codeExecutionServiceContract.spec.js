import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')
const readJson = (relativePath) => JSON.parse(read(relativePath))

describe('physical Code execution service contract', () => {
  it('publishes one versioned persistent service across manifest, catalogue and protected packs', () => {
    const manifest = readJson('addons/official/code-execution/manifest.json')
    const build = readJson('addons/official/code-execution/addon.build.json')
    const catalog = readJson('addons/catalog.json')
    const base = readJson('packs/base.enaddonpack')
    const develop = readJson('packs/develop-parity.enaddonpack')
    const catalogEntry = catalog.addons.find((entry) => entry.id === manifest.id)
    const baseEntry = base.addons.find((entry) => entry.id === manifest.id)
    const developEntry = develop.addons.find((entry) => entry.id === manifest.id)

    expect(manifest).toMatchObject({
      id: 'elephant.code-execution',
      version: '2.2.0',
      native: {
        runner: 'service',
        protocol: 'elephant-addon-service-v1'
      }
    })
    expect(build.runner).toBe('service')
    expect(catalogEntry.version).toBe(manifest.version)
    expect(baseEntry.version).toBe(manifest.version)
    expect(developEntry.version).toBe(manifest.version)
  })

  it('keeps execution, status, cancellation and timeout inside the addon package', () => {
    const renderer = read('addons/official/code-execution/main.js')
    const native = read('addons/official/code-execution/native/src/main.rs')
    const tauriCore = read('Elephant/backend/tauri/src/lib_min.rs')

    expect(renderer).toContain('this.api.native.service.call')
    expect(renderer).toContain("this.service('execute'")
    expect(renderer).toContain("this.service('execution.status'")
    expect(renderer).toContain("this.service('execution.cancel'")
    expect(renderer).toContain('await api.native.service.start()')
    expect(renderer).toContain('await this.api.native.service.stop()')
    expect(renderer).not.toContain('this.api.native.call(')
    expect(renderer).toContain('DEFAULT_TIMEOUT_MS = 15_000')
    expect(renderer).toContain('button.textContent = \'Stop\'')
    expect(renderer).toContain('this.api.editor.watch')

    expect(native).toContain('thread::Builder::new()')
    expect(native).toContain('cancel.load(Ordering::Acquire)')
    expect(native).toContain('started.elapsed() >= Duration::from_millis(prepared.timeout_ms)')
    expect(native).toContain('Refusing to execute code outside the active vault')
    expect(native).toContain('const MAX_OUTPUT_BYTES: usize = 2 * 1024 * 1024;')
    expect(native).toContain('"execution.cancel" => self.cancel_execution(params)')

    expect(tauriCore).not.toContain('code_execution')
    expect(tauriCore).not.toContain('execution.cancel')
    expect(tauriCore).not.toContain('execution.status')
  })

  it('declares mobile execution as a separately built package-owned runtime', () => {
    const manifest = readJson('addons/official/code-execution/manifest.json')

    expect(manifest.native.mobile.android.supported).toBe(false)
    expect(manifest.native.mobile.ios.supported).toBe(false)
    expect(manifest.native.mobile.android.reason).toContain('Web Worker variant')
    expect(manifest.native.mobile.ios.reason).toContain('Web Worker variant')
  })
})
