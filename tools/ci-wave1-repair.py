from pathlib import Path
import re


def text(path):
    return Path(path).read_text()


def save(path, value):
    Path(path).write_text(value)


def one(path, old, new):
    value = text(path)
    count = value.count(old)
    if count != 1:
        raise SystemExit(f'{path}: expected 1 occurrence, found {count}: {old[:100]!r}')
    save(path, value.replace(old, new, 1))


def many(path, old, new, expected):
    value = text(path)
    count = value.count(old)
    if count != expected:
        raise SystemExit(f'{path}: expected {expected} occurrences, found {count}: {old!r}')
    save(path, value.replace(old, new))


one('build/scripts/test-official-addon-packs.mjs', 'const { state, api, browserWindow } = runtime', 'const { state, browserWindow } = runtime')
one('migration/freya/differential-scenarios.json', '"source": "Elephant/freya/src/app/navigation.rs#rail_action",', '"implementationSource": "Elephant/freya/src/app/navigation.rs#rail_action",')

for path, count in [
    ('tests/app/e2e/addon-ui/local-runtime/real-local-runtime.spec.js', 3),
    ('tests/app/e2e/addon-ui/real-sync/real-sync-service.spec.js', 1),
    ('tests/app/e2e/example-addon-ui-usage.spec.js', 1),
]:
    many(path, 'async ({}, testInfo) => {', 'async (fixtures, testInfo) => {\n  void fixtures', count)

one('tests/app/e2e/example-addon-ui-usage.spec.js', "const path = require('node:path')\n", '')
one('tests/app/e2e/example-addon-ui/tauri-harness.js', "const runVisibleCommand = async (app, title) => {\n  await app.command('click', `.en-addon-detail-commands button`)\n  await waitForText(app, '.en-addons-feedback', 'completed', 60000)\n  return title\n}\n\n", '')
one('tools/freya-differential/orchestrate.mjs', "import { expectedActions, loadScenario, materializeFixture, validateScenario } from './lib/scenario.mjs'", "import { loadScenario, materializeFixture, validateScenario } from './lib/scenario.mjs'")
one('tools/freya-differential/tests/fixtures/capture-command.mjs', '  const checkpointDir = path.join(output, action.checkpoint)\n', '')
one('tools/playwright-source-visual-capture/renderer.mjs', '  const { app, page } = launched', '  const { page } = launched')
one('tools/tauri-visual-capture/fixture.mjs', "import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, writeFileSync } from 'node:fs'", "import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync } from 'node:fs'")
one('tools/tauri-visual-capture/index.mjs', "import { mkdirSync, rmSync, writeFileSync } from 'node:fs'", "import { rmSync } from 'node:fs'")
one('tools/tauri-visual-capture/index.mjs', "import { listNativeWindows, setWindowGeometry } from './native-window.mjs'", "import { setWindowGeometry } from './native-window.mjs'")
one('tools/tauri-visual-capture/native-window.mjs', "import { execFileSync, spawnSync } from 'node:child_process'", "import { spawnSync } from 'node:child_process'")
one('tools/tauri-wdio-embedded/runtime.mjs', "import { mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises'", "import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises'")
one('tools/tauri-wdio-embedded/runtime.mjs', "  const editor = await optional('.en-editor-host .editor-component, .en-note-editor-shell')\n", '')
one('tools/tauri-wdio-embedded/tests/differential.e2e.mjs', "import { expectedActions, expectedCheckpoints, loadScenario } from '../../../tools/freya-differential/lib/scenario.mjs'", "import { expectedActions, loadScenario } from '../../../tools/freya-differential/lib/scenario.mjs'")
one('tools/tauri-wdio-embedded/tests/differential.e2e.mjs', "import { byText, captureFrame, displayed, railOrder, requireElement, sleep, snapshotVault, stateSnapshot, visibleByText, writeJson } from '../runtime.mjs'", "import { captureFrame, railOrder, requireElement, sleep, snapshotVault, stateSnapshot, visibleByText, writeJson } from '../runtime.mjs'")
one('tools/tauri-wdio-embedded/tests/differential.e2e.mjs', 'const checkpoints = expectedCheckpoints(scenario)\n', '')
one('tools/tauri-wdio-embedded/wdio.conf.mjs', "import { mkdir, mkdtemp, stat } from 'node:fs/promises'", "import { mkdir, mkdtemp } from 'node:fs/promises'")

ast_path = 'tools/vue-to-freya/ast.mjs'
ast = text(ast_path)
ast, count = re.subn(r"\nfunction attr\(node, name\) \{\n  return node\.props\?\.find\(\(prop\) => prop\.type === NodeTypes\.ATTRIBUTE && prop\.name === name\)\n\}\n", '\n', ast, count=1)
if count != 1:
    raise SystemExit(f'{ast_path}: attr helper removal count={count}')
save(ast_path, ast)

sync_path = 'tests/app/e2e/addon-ui/real-sync/real-sync-service.js'
sync = text(sync_path)
old_decl = "  let scenarioError = null\n  let cleanup = {}\n  let pairing = null\n  const transfers = []\n"
new_decl = "  let scenarioError = null\n  let cleanup = {}\n  let pairing = null\n  let artifact = null\n  const transfers = []\n"
if sync.count(old_decl) != 1:
    raise SystemExit(f'{sync_path}: declaration anchor mismatch')
sync = sync.replace(old_decl, new_decl, 1)
if sync.count('    const artifact = {') != 1:
    raise SystemExit(f'{sync_path}: artifact declaration mismatch')
sync = sync.replace('    const artifact = {', '    artifact = {', 1)
footer_start = sync.find("    if (scenarioError) throw new Error(`${scenarioError.message} (artifact: ${artifactPath})`)")
footer_end_marker = "    return artifact\n  }\n}\n\nmodule.exports = { runRealSyncScenario }"
footer_end = sync.find(footer_end_marker)
if footer_start < 0 or footer_end < footer_start:
    raise SystemExit(f'{sync_path}: finally footer anchors mismatch')
replacement = """  }
  if (scenarioError) throw new Error(`${scenarioError.message} (artifact: ${artifactPath})`)
  assert.equal(cleanup.a.clean, true, 'device A service must exit cleanly after service.stop')
  assert.equal(cleanup.b.clean, true, 'device B service must exit cleanly after service.stop')
  assert.equal(cleanup.processesExited, true, 'both real service processes must exit')
  assert.equal(cleanup.vaultsRemoved, true, 'temporary vaults must be removed')
  return artifact
}

module.exports = { runRealSyncScenario }"""
sync = sync[:footer_start] + replacement + sync[footer_end + len(footer_end_marker):]
save(sync_path, sync)

one('Elephant/frontend/app/components/settings/SettingsPanel.vue', '              <span v-if="activeSection === \'addons\'" id="en-addons-title-actions" class="en-settings-title-actions" />\n', '')

save('tests/app/unit/specs/muya/rustProductionOwnership.spec.js', """import fs from 'node:fs'
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
""")

android_path = 'tests/app/unit/specs/main/elephantnote/androidReleaseRecovery.spec.js'
android = text(android_path)
old = """    expect(runtimeEditor).toContain("import Muya from 'muya/lib'")
    expect(runtimeEditor).toContain('editor.value = new Muya(ele, options)')
"""
new = """    expect(runtimeEditor).toContain("import { RustMuyaRuntimeEditor } from '@/muya'")
    expect(runtimeEditor).toContain("import { createRustEditorRuntimeBinding } from '@/muya/editorRuntimeResource'")
    expect(runtimeEditor).not.toContain("import Muya from 'muya/lib'")
    expect(runtimeEditor).not.toContain('new Muya(')
"""
if android.count(old) != 1:
    raise SystemExit(f'{android_path}: legacy Muya expectation anchor mismatch')
save(android_path, android.replace(old, new, 1))
