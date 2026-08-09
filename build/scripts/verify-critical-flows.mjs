import fs from 'node:fs'
import path from 'node:path'

const root = process.cwd()
const failures = []
const target = (relativePath) => path.join(root, relativePath)

const read = (relativePath) => {
  const absolute = target(relativePath)
  if (!fs.existsSync(absolute)) {
    failures.push(`Missing critical-flow file: ${relativePath}`)
    return ''
  }
  return fs.readFileSync(absolute, 'utf8')
}

const has = (relativePath, needle, description = needle) => {
  if (!read(relativePath).includes(needle)) failures.push(`${relativePath}: missing ${description}`)
}

const lacks = (relativePath, needle, description = needle) => {
  if (read(relativePath).includes(needle)) failures.push(`${relativePath}: unexpected ${description}`)
}

const missing = (relativePath, description = relativePath) => {
  if (fs.existsSync(target(relativePath))) failures.push(`${relativePath}: unexpected ${description}`)
}

const ordered = (relativePath, needles, description) => {
  const content = read(relativePath)
  let cursor = -1
  for (const needle of needles) {
    const index = content.indexOf(needle, cursor + 1)
    if (index < 0) {
      failures.push(`${relativePath}: missing ordered invariant "${needle}" for ${description}`)
      return
    }
    cursor = index
  }
}

const physicalPackages = [
  ['dashboard', 'elephant.dashboard', 'main.js'],
  ['ai', 'elephant.ai', 'main.js'],
  ['ai-chat', 'elephant.ai-chat', 'main.js'],
  ['ai-search', 'elephant.ai-search', 'main.js'],
  ['ai-ocr', 'elephant.ai-ocr', 'main.js'],
  ['wiki', 'elephant.wiki', 'main.v2.js'],
  ['graph', 'elephant.graph', 'main.js'],
  ['knowledge', 'elephant.knowledge', 'main.js'],
  ['open-models', 'elephant.open-models', 'main.js'],
  ['codex-connection', 'elephant.codex-connection', 'main.js'],
  ['sync', 'elephant.sync', 'main.service.js'],
  ['calendar', 'elephant.calendar', 'main.js'],
  ['sites', 'elephant.sites', 'main.js'],
  ['code-execution', 'elephant.code-execution', 'main.js'],
  ['google-keep-import', 'elephant.google-keep-import', 'main.js'],
  ['recently-edited', 'elephant.recently-edited', 'main.js']
]

for (const file of [
  'package.json',
  '.github/workflows/ci.yml',
  '.github/workflows/tauri-ci.yml',
  '.github/workflows/addon-platform-validation.yml',
  'build/scripts/verify-security-guardrails.mjs',
  'Elephant/frontend/src/renderer/src/main.js',
  'Elephant/frontend/src/renderer/src/addons/builtin/index.js',
  'Elephant/frontend/src/renderer/src/addons/externalAddonRuntime.js',
  'Elephant/frontend/src/renderer/src/addons/officialAddonCatalogBridge.js',
  'Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue',
  'Elephant/frontend/app/components/shell/MainContent.vue',
  'Elephant/frontend/app/components/navigation/IconRail.vue',
  'Elephant/frontend/app/components/editor/NoteEditorHost.vue',
  'Elephant/frontend/app/components/editor/ExcalidrawDialog.vue',
  'Elephant/backend/tauri/Cargo.toml',
  'Elephant/backend/tauri/src/lib_min.rs',
  'Elephant/backend/tauri/src/core_commands.rs',
  'Elephant/backend/tauri/src/addon_services.rs',
  'Elephant/backend/tauri/src/addon_runtime_access.rs',
  'Elephant/backend/tauri/src/addon_http_access.rs',
  'Elephant/backend/tauri/src/official_addon_catalog.rs',
  'tests/app/e2e/search-inspect.spec.js',
  'tests/app/unit/addons/baseOfficialAddonRuntime.spec.js'
]) read(file)

missing('Elephant/backend/tauri/src/tauri_extra_commands.rs', 'legacy optional command module')
missing('Elephant/backend/tauri/src/sync_commands.rs', 'legacy core Sync commands')
missing('Elephant/backend/tauri/src/sync', 'legacy core Iroh runtime directory')
missing('Elephant/backend/tauri/src/vault/sync_iroh', 'legacy core Sync backend directory')
missing('Elephant/frontend/app/components/views/DashboardView.vue', 'core Dashboard view')

ordered(
  '.github/workflows/ci.yml',
  [
    '- name: Critical ElephantNote flow guard',
    'run: node build/scripts/verify-critical-flows.mjs',
    '- name: Security guardrails',
    'run: pnpm security:guard',
    '- name: Unit suite'
  ],
  'main CI guard, security and unit order'
)
has(
  '.github/workflows/ci.yml',
  'cargo check --manifest-path Elephant/backend/tauri/Cargo.toml --all-targets --no-default-features',
  'blocking Tauri Cargo check'
)
has(
  '.github/workflows/tauri-ci.yml',
  'cargo test --manifest-path Elephant/backend/tauri/Cargo.toml --lib --no-default-features',
  'blocking Tauri library tests'
)
has('.github/workflows/addon-platform-validation.yml', 'Package every integrated physical addon', 'physical addon packaging gate')
has('package.json', '"security:guard": "node build/scripts/verify-security-guardrails.mjs"', 'security guard command')

ordered(
  'Elephant/frontend/src/renderer/src/main.js',
  [
    "import './addons/officialAddonCatalogBridge'",
    'clearBootstrapFileUtilsFallbackForTauri()',
    'installTauriRuntimeBridge()',
    'ensureRendererPathFacade()',
    'installTauriElephantNoteBridge()'
  ],
  'official catalogue and Tauri renderer bridge installation order'
)
has('Elephant/frontend/src/renderer/src/main.js', "const runtime = 'tauri'", 'Tauri-only runtime selection')
lacks('Elephant/frontend/src/renderer/src/main.js', 'tauri-compatible', 'compatibility runtime fallback')

has('Elephant/frontend/src/renderer/src/addons/builtin/index.js', 'builtinAddons = Object.freeze([])', 'empty builtin addon catalogue')
lacks('Elephant/frontend/src/renderer/src/addons/builtin/index.js', 'import(', 'bundled optional addon import')

has('Elephant/frontend/src/renderer/src/addons/externalAddonRuntime.js', 'const isOfficialRecord', 'official package classification')
has('Elephant/frontend/src/renderer/src/addons/externalAddonRuntime.js', "if (!official && !await externalAddonApi.getCommunityEnabled())", 'community consent boundary')
has('Elephant/frontend/src/renderer/src/addons/isolatedAddonWorkerSource.js', "['fetch','WebSocket','EventSource','XMLHttpRequest'", 'isolated worker network surface removal')
has('Elephant/frontend/src/renderer/src/addons/isolatedAddonWorkerSource.js', "rpc('storage.get'", 'brokered addon storage')
has('Elephant/frontend/src/renderer/src/addons/officialAddonCatalogBridge.js', 'tauri_official_addons_catalog_list', 'official catalogue bridge')
has('Elephant/backend/tauri/src/lib_min.rs', 'addons::tauri_addons_list,', 'external addon registry compatibility command')

lacks('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', 'en-addon-browser-overview', 'obsolete addon overview surface')
lacks('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', 'All addons', 'obsolete catalogue headline')
lacks('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', 'Back to catalogue', 'obsolete detail-only navigation')
has('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', 'class="en-addons-toolbar"', 'shared addon and pack toolbar')
has('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', 'placeholder="Search addons"', 'addon search control')
has('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', 'placeholder="Search addon packs"', 'addon pack search control')
has('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', 'class="en-addon-browser"', 'persistent split addon browser')
has('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', ':data-addon-id="entry.id"', 'stable addon list identity')
has('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', 'class="en-installed-only-control"', 'installed addon filter')
for (const addonId of ['elephant.ai-chat', 'elephant.ai-search', 'elephant.ai-ocr', 'elephant.wiki', 'elephant.graph']) {
  has('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue', `'${addonId}'`, `visible AI module ${addonId}`)
}

has('Elephant/frontend/app/components/shell/MainContent.vue', '<addon-workspace-router', 'physical addon workspace router')
has('Elephant/frontend/app/components/shell/MainContent.vue', "entry?.contribution?.zone === 'workspace.notes'", 'package-owned workspace panels')
lacks('Elephant/frontend/app/components/shell/MainContent.vue', 'DashboardView', 'core Dashboard implementation')
lacks('Elephant/frontend/app/components/shell/MainContent.vue', 'SigmaCanvas', 'core Graph implementation')
lacks('Elephant/frontend/app/components/shell/MainContent.vue', 'WikiView', 'core Wiki implementation')
lacks('Elephant/frontend/app/components/shell/MainContent.vue', 'ModelsView', 'core model library implementation')
lacks('Elephant/frontend/app/components/navigation/IconRail.vue', "id: 'dashboard', title: 'Dashboard'", 'core Dashboard rail item')
has('Elephant/frontend/app/components/navigation/IconRail.vue', 'dashboard: LayoutDashboard', 'addon Dashboard icon mapping')

has('Elephant/backend/tauri/src/addon_services.rs', 'const SERVICE_PROTOCOL: &str = "elephant-addon-service-v1"', 'versioned addon service protocol')
has('Elephant/backend/tauri/src/addon_services.rs', 'Addon native permission was not granted', 'native permission gate')
has('Elephant/backend/tauri/src/addon_services.rs', 'Path traversal is not allowed', 'service executable traversal rejection')
has('Elephant/backend/tauri/src/addon_services.rs', 'Persistent process services require a desktop addon package', 'mobile process rejection')
has('Elephant/backend/tauri/src/addon_services.rs', 'MAX_RESPONSE_BYTES', 'bounded service responses')

for (const command of [
  'tauri_addons_service_status',
  'tauri_addons_service_start',
  'tauri_addons_service_call',
  'tauri_addons_service_stop'
]) has('Elephant/backend/tauri/src/lib_min.rs', command, `registered ${command} command`)

has('Elephant/backend/tauri/src/lib_min.rs', '#[path = "core_commands.rs"]', 'minimal core command module')
for (const leakedCoreMarker of [
  'pub mod ocr;',
  'pub mod model_domain;',
  'pub mod sync;',
  'tauri_ocr_',
  'tauri_embedding_',
  'tauri_models_',
  'tauri_codex_',
  'tauri_ai_config_',
  'tauri_search_inspect',
  'iroh_sync_'
]) lacks('Elephant/backend/tauri/src/lib_min.rs', leakedCoreMarker, `optional runtime leakage ${leakedCoreMarker}`)

for (const leakedDependency of [
  'iroh =',
  'iroh-mdns-address-lookup',
  'tokenizers =',
  'fastembed ='
]) lacks('Elephant/backend/tauri/Cargo.toml', leakedDependency, `optional dependency ${leakedDependency}`)

has('Elephant/backend/tauri/Cargo.toml', 'reqwest =', 'generic permission-scoped addon HTTP client')
has('Elephant/backend/tauri/src/addon_http_access.rs', 'read_enabled_addon', 'enabled-package HTTP permission check')
has('Elephant/backend/tauri/src/addon_http_access.rs', 'Network access to a local or private address', 'addon HTTP anti-SSRF guard')
has('Elephant/backend/tauri/src/addon_http_access.rs', 'External addon HTTPS requests are restricted to port 443', 'addon HTTPS port restriction')

for (const leakedCoreImplementation of [
  'tauri_ai_config_get',
  'tauri_models_get_selection',
  'tauri_search_inspect',
  'portable-markdown-index',
  'codex --version',
  'tauri-rust://'
]) lacks('Elephant/backend/tauri/src/core_commands.rs', leakedCoreImplementation, `optional implementation ${leakedCoreImplementation}`)

for (const [directory, addonId, entry] of physicalPackages) {
  const base = `addons/official/${directory}`
  const manifest = `${base}/manifest.json`
  const main = `${base}/${entry}`
  has(manifest, `"id": "${addonId}"`, `${addonId} manifest id`)
  has(manifest, '"runtime"', `${addonId} runtime declaration`)
  read(main)
}

has('addons/official/dashboard/main.js', "const DASHBOARD_DIRECTORY = '.elephantnote'", 'hidden Elephant Dashboard directory')
has('addons/official/dashboard/main.js', "const DASHBOARD_FILENAME = 'Dashboard.md'", 'Dashboard note filename')
has('addons/official/dashboard/main.js', 'api.workspace.registerSidebarItem', 'package-owned Dashboard rail item')
has('addons/official/dashboard/main.js', 'api.commands.register', 'package-owned Dashboard command')
has('addons/official/dashboard/main.js', 'await store.openNote(note, { record: false })', 'normal Dashboard note opening')
lacks('addons/official/dashboard/main.js', 'api.workspace.registerView', 'custom Dashboard workspace view')
lacks('addons/official/dashboard/main.js', 'dashboard.provider', 'custom Dashboard provider')
lacks('addons/official/dashboard/main.js', 'Recently edited', 'custom Dashboard recent-note surface')
has('addons/catalog.json', '"id": "elephant.dashboard"', 'downloadable Dashboard package')
has('packs/base.enaddonpack', '"id": "elephant.dashboard"', 'Dashboard in base pack')

has('addons/official/ai/main.js', "const CONFIG_KEY = 'provider-config'", 'package-owned AI configuration')
has('addons/official/ai/main.js', 'this.api.storage.get(CONFIG_KEY)', 'AI configuration storage read')
has('addons/official/ai/main.js', 'this.api.storage.set(CONFIG_KEY, payload)', 'AI configuration storage write')
has('addons/official/ai/main.js', "api.resources.provide('ai.config'", 'AI configuration resource')
lacks('addons/official/ai/main.js', 'tauri_ai_config_', 'core AI configuration bridge')
lacks('addons/official/ai/main.js', 'ollama:', 'implicit local model provider')
lacks('addons/official/ai/main.js', 'lmstudio:', 'implicit local model provider')
lacks('addons/official/ai/main.js', 'llamacpp:', 'implicit local model provider')

has('addons/official/ai-search/main.js', "const PROVIDER_RESOURCE = 'search.provider'", 'package-owned search provider')
has('addons/official/ai-search/main.js', "engine: 'package-owned-bm25'", 'package-owned lexical index')
has('addons/official/wiki/main.v2.js', 'wiki.provider', 'package-owned Wiki provider')
has('addons/official/graph/main.js', 'api.workspace.registerView', 'package-owned Graph view')
has('addons/official/open-models/manifest.json', '"runner": "service"', 'Open Models native service')
has('addons/official/codex-connection/manifest.json', '"runner": "service"', 'Codex native service')
has('addons/official/sync/manifest.json', '"protocol": "elephant-addon-service-v1"', 'Sync native service protocol')
has('addons/official/sync/main.service.js', "this.callNativeService('sync.run'", 'active package Sync path')
has('addons/official/sync/native/src/main.rs', '"sync.run" => service.run_sync().await', 'package-owned Sync sessions')
has('addons/official/sync/native/tests/two_endpoint_sync.rs', 'physical_package_pairs_and_synchronizes_two_real_iroh_endpoints', 'real package Iroh validation')

has('Elephant/backend/tauri/src/official_addon_catalog/source.rs', 'collect_local_files', 'complete local official package installation')
has('Elephant/backend/tauri/src/official_addon_catalog/install.rs', 'collect_remote_files', 'complete remote official package installation')
has('tests/app/unit/addons/baseOfficialAddonRuntime.spec.js', 'for (const packedAddon of parityPack.addons)', 'all first-party addon runtime probes')
has('tests/app/unit/addons/baseOfficialAddonRuntime.spec.js', 'view.component.__mount', 'addon view usage probes')
has('tests/app/unit/addons/baseOfficialAddonRuntime.spec.js', 'command.run({ probe: true })', 'addon command usage probes')

has('tests/app/e2e/search-inspect.spec.js', 'does not expose semantic inspection without the Search addon', 'Search physical absence E2E contract')
has('tests/app/e2e/search-inspect.spec.js', 'expect(result.ok).toBe(false)', 'unavailable core Search command')

ordered(
  'Elephant/frontend/app/components/editor/NoteEditorHost.vue',
  [
    "import { elephantnoteClient } from '../../services/elephantnoteClient'",
    'const AUTOSAVE_POLL_MS',
    'elephantnoteClient.notes.write({',
    'noteSaveInterval = window.setInterval'
  ],
  'editor autosave persistence'
)
ordered(
  'Elephant/frontend/app/components/editor/ExcalidrawDialog.vue',
  [
    'const blobToBytes = async(blob) => new Uint8Array(await blob.arrayBuffer())',
    'imageBlob: await blobToBytes(blob)',
    'sceneBlob: await sceneBlob.text()'
  ],
  'Excalidraw writable payload'
)

if (failures.length) {
  console.error('Critical ElephantNote flow guard failed:')
  for (const failure of failures) console.error(`- ${failure}`)
  process.exit(1)
}

console.log('Critical ElephantNote flow guard passed.')
