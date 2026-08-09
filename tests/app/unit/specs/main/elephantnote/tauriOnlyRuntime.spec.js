import fs from 'node:fs'
import path from 'node:path'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('Tauri-only renderer runtime', () => {
  it('does not silently bootstrap a browser or Electron-compatible renderer fallback', () => {
    const main = read('Elephant/frontend/src/renderer/src/main.js')

    expect(main).toContain("import { installTauriRuntimeBridge } from './platform/tauriRuntimeBridge'")
    expect(main).toContain('installTauriRuntimeBridge()')
    expect(main).toContain("const runtime = 'tauri'")
    expect(main).toContain('unsupported runtime')
    expect(main).not.toContain('tauri-compatible')
    expect(main).not.toContain('} else {\n    bootstrapRenderer()')
  })

  it('keeps the public renderer entrypoint guarded by a strict Tauri bridge wrapper', () => {
    const bridge = read('Elephant/frontend/src/renderer/src/platform/tauriRuntimeBridge.js')

    expect(bridge).toContain('export const installTauriRuntimeBridge')
    expect(bridge).toContain('requires the Tauri runtime bridge')
    expect(bridge).toContain("mode !== 'tauri'")
    expect(bridge).not.toContain('tauri-compatible')
  })

  it('keeps renderer path helpers extracted from the Tauri bootstrap entrypoint', () => {
    const main = read('Elephant/frontend/src/renderer/src/main.js')
    const facade = read('Elephant/frontend/src/renderer/src/platform/rendererPathFacade.js')

    expect(main).toContain("import { ensureRendererPathFacade } from './platform/rendererPathFacade'")
    expect(main).toContain('ensureRendererPathFacade()')
    expect(main).not.toContain('const ensurePathResolve =')
    expect(facade).toContain('export const ensureRendererPathFacade')
    expect(facade).toContain('scope.path.resolve')
    expect(facade).toContain('scope.path.relative')
  })

  it('keeps search concept fallback out of the renderer bootstrap entrypoint', () => {
    const main = read('Elephant/frontend/src/renderer/src/main.js')
    const fallback = read('Elephant/frontend/src/renderer/src/platform/tauriSearchConceptFallback.js')

    expect(main).toContain("import { installTauriSearchConceptFallback } from './platform/tauriSearchConceptFallback'")
    expect(main).toContain('installTauriSearchConceptFallback()')
    expect(main).not.toContain('const normalizeSearchPath =')
    expect(main).not.toContain('const titleFromPath =')
    expect(fallback).toContain('export const installTauriSearchConceptFallback')
    expect(fallback).toContain('rust-search-concepts-command-unavailable')
  })
})
