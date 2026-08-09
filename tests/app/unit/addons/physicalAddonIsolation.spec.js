import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')
const exists = (relativePath) => fs.existsSync(path.join(root, relativePath))

const PHYSICAL_PACKAGES = Object.freeze([
  ['ai', 'elephant.ai'],
  ['ai-chat', 'elephant.ai-chat'],
  ['ai-search', 'elephant.ai-search'],
  ['ai-ocr', 'elephant.ai-ocr'],
  ['wiki', 'elephant.wiki'],
  ['graph', 'elephant.graph'],
  ['open-models', 'elephant.open-models'],
  ['codex-connection', 'elephant.codex-connection'],
  ['sync', 'elephant.sync'],
  ['calendar', 'elephant.calendar'],
  ['sites', 'elephant.sites'],
  ['code-execution', 'elephant.code-execution'],
  ['google-keep-import', 'elephant.google-keep-import'],
  ['recently-edited', 'elephant.recently-edited']
])

const REMOVED_BUILTIN_IMPLEMENTATIONS = Object.freeze([
  'ai.js',
  'aiChat.js',
  'aiSearch.js',
  'aiOcr.js',
  'wiki.js',
  'graph.js',
  'openModels.js',
  'codexConnection.js',
  'sync.js',
  'calendar.js',
  'sites.js',
  'codeExecution.js',
  'googleKeepImport.js',
  'recentlyEdited.js',
  'aiProviderRouteOwnership.js'
])

const REMOVED_CORE_OPTIONAL_DEPENDENCIES = Object.freeze([
  '@huggingface/transformers',
  '@sigma/edge-curve',
  '@wllama/wllama',
  'graphology',
  'node-llama-cpp',
  'sigma',
  'vectra'
])

describe('physical addon isolation', () => {
  it('keeps optional renderer implementations out of the builtin addon directory', () => {
    const builtinIndex = read('Elephant/frontend/src/renderer/src/addons/builtin/index.js')
    for (const file of REMOVED_BUILTIN_IMPLEMENTATIONS) {
      expect(exists(`Elephant/frontend/src/renderer/src/addons/builtin/${file}`)).toBe(false)
    }
    for (const [, addonId] of PHYSICAL_PACKAGES) expect(builtinIndex).not.toContain(addonId)
    expect(builtinIndex).toContain('builtinAddons = Object.freeze([])')
  })

  it('keeps Addon Packs and Excalidraw in the core feature bootstrap, not the addon catalogue', () => {
    const main = read('Elephant/frontend/src/renderer/src/main.js')
    const packs = read('Elephant/frontend/src/renderer/src/addons/builtin/addonProfiles.js')
    const excalidraw = read('Elephant/frontend/src/renderer/src/addons/builtin/excalidraw.js')

    expect(main).toContain('activateCoreFeature')
    expect(main).toContain('addonPacksCoreFeature')
    expect(main).toContain('excalidrawCoreFeature')
    expect(packs).toContain('id: CORE_FEATURE_ID')
    expect(packs).not.toContain('manifest:')
    expect(excalidraw).toContain('id: CORE_FEATURE_ID')
    expect(excalidraw).not.toContain('manifest:')
  })

  it('keeps every optional implementation inside a separately packageable addon', () => {
    for (const [slug, addonId] of PHYSICAL_PACKAGES) {
      const packageRoot = `addons/official/${slug}`
      const manifestPath = `${packageRoot}/manifest.json`
      expect(exists(manifestPath)).toBe(true)
      const manifest = JSON.parse(read(manifestPath))
      const runtimeEntry = String(manifest.runtime?.entry || '')
      const entryPath = `${packageRoot}/${runtimeEntry}`
      expect(manifest.id).toBe(addonId)
      expect(manifest.runtime).toEqual(expect.objectContaining({ type: 'javascript-worker' }))
      expect(runtimeEntry).toMatch(/^[a-zA-Z0-9._-]+\.js$/)
      expect(exists(entryPath)).toBe(true)
      expect(manifest.contributes?.runtimeMode).toBe('trusted')
      const entry = read(entryPath)
      expect(entry).toMatch(/export\s+default\s+class/)
      expect(entry).toContain('onload(')
    }
  })

  it('keeps OCR native code out of Elephant core and inside its package', () => {
    const core = read('Elephant/backend/tauri/src/lib_min.rs')
    const ocrManifest = JSON.parse(read('addons/official/ai-ocr/manifest.json'))
    const ocrEntry = read('addons/official/ai-ocr/main.js')
    const sidecar = read('addons/official/ai-ocr/native/src/main.rs')

    expect(core).not.toContain('pub mod ocr;')
    expect(core).not.toContain('tauri_ocr_status')
    expect(core).not.toContain('tauri_ocr_image')
    expect(exists('Elephant/backend/tauri/src/ocr.rs')).toBe(false)
    expect(ocrManifest.permissions.native).toBe(true)
    expect(ocrManifest.requires).toEqual({ 'elephant.ai': '>=2.0.0' })
    expect(Object.keys(ocrManifest.native.sidecars)).toContain('macos-aarch64')
    expect(ocrEntry).toContain('this.api.native.call')
    expect(ocrEntry).toContain('this.api.storage.get')
    expect(ocrEntry).not.toContain('api.experimental')
    expect(sidecar).toContain('elephant-addon-sidecar-v1')
    expect(sidecar).toContain('"ocr.image"')
  })

  it('routes chat through installed provider contributions instead of a global RAG client', () => {
    const chat = read('addons/official/ai-chat/main.v2.js')
    const base = read('addons/official/ai-chat/main.js')

    expect(base).toContain("getContributions?.('ai.providers')")
    expect(base).toContain("typeof option.provider.chat !== 'function'")
    expect(chat).toContain('async runProvider(')
    expect(chat).not.toContain("this.call('rag.chat'")
    expect(chat).not.toContain('Provider id')
  })

  it('removes package-specific AI and graph libraries from the core package importer', () => {
    const packageJson = JSON.parse(read('package.json'))
    for (const dependency of REMOVED_CORE_OPTIONAL_DEPENDENCIES) {
      expect(packageJson.dependencies).not.toHaveProperty(dependency)
    }
  })

  it('removes generic core implementations now owned by AI packages', () => {
    const core = read('Elephant/backend/tauri/src/lib_min.rs')

    expect(exists('Elephant/backend/tauri/src/rag_prompt.rs')).toBe(false)
    expect(exists('Elephant/backend/tauri/src/ollama.rs')).toBe(false)
    expect(core).not.toContain('pub mod rag_prompt;')
    expect(core).not.toContain('pub mod ollama;')
    expect(core).not.toContain('tauri_rag_build_prompt')
    expect(core).not.toContain('tauri_ollama_')
  })

  it('uses only the generic native host for package-owned sidecars', () => {
    const core = read('Elephant/backend/tauri/src/lib_min.rs')
    const host = read('Elephant/backend/tauri/src/addon_sidecars.rs')
    const trustedRuntime = read('Elephant/frontend/src/renderer/src/addons/trustedAddonRuntime.js')

    expect(core).toContain('addon_sidecars::tauri_addons_sidecar_status')
    expect(core).toContain('addon_sidecars::tauri_addons_sidecar_call')
    expect(host).not.toContain('tesseract')
    expect(host).not.toContain('OCR')
    expect(host).toContain('Addon native permission was not granted')
    expect(host).toContain('Addon sidecar escapes its package directory')
    expect(trustedRuntime).toContain('native: Object.freeze({')
    expect(trustedRuntime).toContain('storage: Object.freeze({')
  })

  it('enforces parent requirements and official provenance at native and renderer boundaries', () => {
    const core = read('Elephant/backend/tauri/src/lib_min.rs')
    const dependencies = read('Elephant/backend/tauri/src/addon_dependencies.rs')
    const manager = read('Elephant/frontend/src/renderer/src/addons/AddonManager.js')
    const registryView = read('Elephant/backend/tauri/src/addon_registry_view.rs')
    const catalog = read('Elephant/backend/tauri/src/addon_catalog.rs')
    const store = read('Elephant/frontend/src/renderer/src/store/addons.js')

    expect(core).toContain('addon_dependencies::tauri_addons_set_enabled')
    expect(core).toContain('addon_dependencies::tauri_addons_uninstall')
    expect(core).toContain('addon_registry_view::tauri_addons_list')
    expect(dependencies).toContain('validate_requirements')
    expect(dependencies).toContain('VersionReq::parse')
    expect(dependencies).toContain('Cannot disable')
    expect(dependencies).toContain('Cannot uninstall')
    expect(manager).toContain('assertDependenciesEnabled')
    expect(manager).toContain('getDependents')
    expect(registryView).toContain('physical_manifest')
    expect(catalog).toContain('persist_official_source')
    expect(catalog).toContain('item.official')
    expect(store).toContain('isOfficialManifest')
    expect(store).toContain('isOfficialCatalogEntry')
  })

  it('supports source packages and complete hashed native enaddon archives', () => {
    const catalog = read('Elephant/backend/tauri/src/addon_catalog.rs')
    const buildScript = read('build/scripts/build-physical-addon.mjs')
    const packager = read('build/tools/enaddon-packager/src/main.rs')
    const validator = read('build/scripts/validate-addon-catalog.mjs')

    expect(catalog).toContain('package_path')
    expect(catalog).toContain('package_hash')
    expect(catalog).toContain('download_prebuilt_package')
    expect(catalog).toContain('blake3::hash')
    expect(buildScript).toContain('build/out/addons/releases')
    expect(packager).toContain('blake3::hash')
    expect(validator).toContain('must be explicitly marked official')
    expect(validator).toContain('trusted module escapes its package directory')
    expect(validator).toContain('trusted module imports an external dependency')
    expect(validator).toContain('trusted modules cannot use dynamic import')
  })
})
