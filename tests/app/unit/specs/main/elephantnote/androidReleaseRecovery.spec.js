import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('Android release recovery', () => {
  it('uses Android-only aliases instead of unresolved Vite Node stubs', () => {
    const vite = read('vite.tauri.config.mjs')
    const shim = read(
      'Elephant/frontend/src/renderer/src/platform/mobileNodeBuiltinsShim.js'
    )
    const build = read('build/scripts/build_dev_apk.sh')

    expect(vite).toContain("process.env.ELEPHANTNOTE_ANDROID_BUILD === '1'")
    for (const builtin of ['child_process', 'fs/promises', 'crypto', 'zlib']) {
      expect(vite).toContain(`'${builtin}'`)
    }
    expect(vite).toContain('__ELEPHANTNOTE_ANDROID_BUILD__')
    expect(shim).toContain('Use the Tauri bridge instead')
    expect(shim).toContain('export const spawn')
    expect(shim).toContain('export const statSync')
    expect(shim).toContain('export const createHash')
    expect(build).toContain("'__vite-browser-external'")
    expect(build).toContain('renderer contains no unresolved Vite Node builtin stubs')
  })

  it('installs the browser process facade before any renderer module evaluates', () => {
    const html = read('Elephant/frontend/src/renderer/index.html')
    const processFacade = html.indexOf('globalThis.process ||=')
    const addonEntry = html.indexOf('src="/src/addons/addonVueBridge.js"')
    const mainEntry = html.indexOf('src="/src/main.js"')

    expect(processFacade).toBeGreaterThan(0)
    expect(processFacade).toBeLessThan(addonEntry)
    expect(processFacade).toBeLessThan(mainEntry)
    expect(html).toContain("platform: 'android'")
    expect(html).toContain("NODE_ENV: 'production'")
    expect(html).toContain('nextTick: (callback, ...args)')
  })

  it('never leaves Android on a silent white renderer surface', () => {
    const html = read('Elephant/frontend/src/renderer/index.html')
    const main = read('Elephant/frontend/src/renderer/src/main.js')

    expect(html).toContain('name="viewport"')
    expect(html).toContain('id="elephant-startup"')
    expect(html).toContain('Elephant')
    expect(html).toContain('Démarrage…')
    expect(main).toContain('const renderStartupFailure')
    expect(main).toContain("surface.dataset.error = 'true'")
    expect(main).toContain('Elephant n’a pas pu démarrer')
    expect(main).toContain('renderStartupFailure(error)')
    expect(main).toContain("dataset.elephantMounted = 'true'")
    expect(main).not.toContain('setTimeout(() => { throw error }, 0)')
  })

  it('uses one mobile vault shell and opens documents through the real NoteEditorHost', () => {
    const page = read('Elephant/frontend/src/renderer/src/pages/app.vue')
    const mainContent = read('Elephant/frontend/app/components/shell/MainContent.vue')
    const noteEditor = read('Elephant/frontend/app/components/editor/NoteEditorHost.vue')
    const runtimeEditor = read(
      'Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditor.vue'
    )

    expect(page).toContain('<app-shell v-if="init" />')
    expect(page).not.toContain('MuyaRuntimeEditor')
    expect(page).not.toContain('muya-runtime-production-editor')
    expect(mainContent).toContain('<note-editor-host')
    expect(mainContent).toContain('v-if="hasOpenNote"')
    expect(noteEditor).toContain('<note-editor-top-bar')
    expect(noteEditor).toContain('<editor-with-tabs')
    expect(runtimeEditor).toContain("import Muya from 'muya/lib'")
    expect(runtimeEditor).toContain('editor.value = new Muya(ele, options)')
  })

  it('registers every vault binary command used by the Android file facade', () => {
    const facade = read('Elephant/frontend/src/renderer/src/platform/tauriFileUtilsPathGuards.js')
    const commands = read('Elephant/backend/tauri/src/vault_file_commands.rs')
    const runtime = read('Elephant/backend/tauri/src/lib_min.rs')

    for (const command of [
      'tauri_vault_read_binary',
      'tauri_vault_write_binary',
      'tauri_vault_ensure_dir',
      'tauri_vault_remove_path',
      'tauri_vault_rename_path'
    ]) {
      expect(facade).toContain(command)
      expect(commands).toContain(`pub fn ${command}`)
      expect(runtime).toContain(`vault_file_commands::${command}`)
    }
    expect(commands).toContain('Refusing to access a path outside the active vault')
    expect(commands).toContain('Refusing to write outside the active vault')
  })

  it('installs a stable mobile search lifecycle only in the live Tauri runtime', () => {
    const clientFactory = read('Elephant/frontend/app/services/elephantnoteClient/domainClients.js')
    const bridge = read('Elephant/frontend/src/renderer/src/platform/tauriSearchLifecycleBridge.js')
    const main = read('Elephant/frontend/src/renderer/src/main.js')
    const runtime = read('Elephant/backend/tauri/src/lib_min.rs')

    for (const method of ['initVault', 'inspect', 'rebuild', 'clear', 'disable', 'enable']) {
      expect(clientFactory).not.toContain(`${method}:`)
    }
    expect(bridge).toContain('Object.assign(clientSearch, lifecycle)')
    expect(bridge).toContain('Object.assign(bridgeSearch, lifecycle)')
    expect(bridge).toContain('requires the optional Search addon')
    expect(bridge).toContain("status: status?.status || 'ready'")
    expect(main).toContain('installTauriSearchLifecycleBridge({ target: globalThis, client: elephantnoteClient })')
    expect(runtime).toContain('vault::commands::tauri_search_query')
    expect(runtime).toContain('vault::commands::tauri_search_status')
    expect(runtime).not.toContain('tauri_search_inspect')
    expect(runtime).not.toContain('search_commands::')
    expect(fs.existsSync(path.join(root, 'Elephant/backend/tauri/src/search_commands.rs'))).toBe(false)
  })

  it('keeps the mobile drawer over the workspace and settings in a phone layout', () => {
    const mobileCss = read('Elephant/frontend/src/renderer/src/mobile-recovery.css')
    const interactionRuntime = read('Elephant/frontend/src/renderer/src/platform/mobileInteractionRuntime.js')

    expect(interactionRuntime).toContain("import '../mobile-recovery.css'")
    expect(mobileCss).toContain('.en-mobile-shell .en-body.en-sidebar-hidden')
    expect(mobileCss).toContain('grid-template-columns: minmax(0, 1fr) !important')
    expect(mobileCss).toContain('.en-mobile-shell .en-sidebar')
    expect(mobileCss).toContain('position: fixed !important')
    expect(mobileCss).toContain('.en-settings-nav')
    expect(mobileCss).toContain('grid-template-columns: repeat(4, minmax(0, 1fr)) !important')
    expect(mobileCss).toContain('.en-settings-row:not(.en-settings-row-stacked)')
  })

  it('publishes the integrated official addon catalogue instead of an empty mobile list', () => {
    const catalog = read('addons/catalog.json')
    const backend = read('Elephant/backend/tauri/src/official_addon_catalog.rs')
    const renderer = read('Elephant/frontend/src/renderer/src/addons/officialAddonCatalogBridge.js')
    const runtime = read('Elephant/backend/tauri/src/lib_min.rs')

    expect(catalog).toContain('elephant.code-execution')
    expect(catalog).toContain('elephant.ai')
    expect(backend).toContain('tauri_official_addons_catalog_list')
    expect(backend).toContain('tauri_official_addons_catalog_install')
    expect(renderer).toContain('tauri_official_addons_catalog_list')
    expect(renderer).toContain('tauri_official_addons_catalog_install')
    expect(runtime).toContain('official_addon_catalog::tauri_official_addons_catalog_list')
    expect(runtime).toContain('official_addon_catalog::tauri_official_addons_catalog_install')
  })

  it('builds one optimized signed ARM64 release APK under 24 MiB', () => {
    const build = read('build/scripts/build_dev_apk.sh')
    const activity = read('build/android/MainActivity.kt')
    const workflow = read('.github/workflows/android-apk.yml')
    const cargo = read('Elephant/backend/tauri/Cargo.toml')

    expect(build).toContain('ANDROID_TARGET="${ANDROID_TARGET:-aarch64}"')
    expect(build).toContain('ANDROID_BUILD_PROFILE="${ANDROID_BUILD_PROFILE:-release}"')
    expect(build).toContain('ANDROID_APK_MAX_MIB="${ANDROID_APK_MAX_MIB:-24}"')
    expect(build).toContain('ELEPHANTNOTE_ANDROID_BUILD=1')
    expect(build).not.toContain('ELEPHANTNOTE_SKIP_LLAMA_BUNDLE')
    expect(build).toContain('android:icon')
    expect(build).toContain('useLegacyPackaging = true')
    expect(build).toContain('verify_single_target_abi')
    expect(build).toContain('apksigner')
    expect(build).toContain('verify --verbose')
    expect(cargo).toContain('[profile.release]')
    expect(cargo).toContain('codegen-units = 1')
    expect(cargo).toContain('lto = "thin"')
    expect(cargo).toContain('opt-level = "z"')
    expect(cargo).toContain('strip = "symbols"')
    expect(activity).not.toContain('requestPermissions(')
    expect(activity).toContain('decorView.post')
    expect(activity).toContain('WindowInsets.Type.systemBars()')
    expect(workflow).toContain('ANDROID_APK_MAX_MIB: 24')
    expect(workflow).toContain('pnpm tauri:android:build')
    expect(workflow).toContain('Elephant-develop_next-arm64-release-review.apk')
  })
})
