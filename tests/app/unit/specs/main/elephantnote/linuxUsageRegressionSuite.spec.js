import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')
const catalog = JSON.parse(read('tests/app/usage/linux/scenarios.json'))
const suite = read('tests/app/e2e/linux-usage-regressions.spec.js')
const helpers = read('tests/app/e2e/helpers.js')
const electronMain = read('tests/app/e2e/electron-main.js')
const preloadEntry = read('tests/app/e2e/tauri-preload-entry.js')
const workflow = read('.github/workflows/e2e.yml')
const config = read('tests/app/e2e/playwright.config.js')
const appPage = read('Elephant/frontend/src/renderer/src/pages/app.vue')

const expectedIds = [
  'startup-render',
  'vault-chooser-first-run',
  'library-layout-roundtrip',
  'library-title-sort',
  'sidebar-hide-show',
  'search-roundtrip',
  'search-no-results',
  'settings-roundtrip',
  'theme-mode-roundtrip',
  'open-existing-note',
  'editor-write-autosave-preview',
  'note-close-roundtrip',
  'pin-unpin-note',
  'addon-install-enable-action',
  'reload-preserves-vault',
  'navigation-stress'
]

describe('progressive Linux application usage simulations', () => {
  it('keeps a unique permanent scenario catalog with non-duplicated evidence policy', () => {
    expect(catalog.schemaVersion).toBe(2)
    expect(catalog.policy.requiredOnPullRequest).toBe(true)
    expect(catalog.policy.captureDistinctVisualStates).toBe(true)
    expect(catalog.policy.rejectPixelIdenticalScreenshots).toBe(true)
    expect(catalog.policy.captureStateEvidence).toBe(true)
    expect(catalog.policy.failOnPageError).toBe(true)
    expect(catalog.policy.failOnConsoleError).toBe(true)

    const ids = catalog.scenarios.map((scenario) => scenario.id)
    expect(ids).toEqual(expectedIds)
    expect(new Set(ids).size).toBe(ids.length)
    for (const scenario of catalog.scenarios) {
      expect(scenario.description.length).toBeGreaterThan(25)
      expect(scenario.regression.length).toBeGreaterThan(20)
      expect(scenario.tags.length).toBeGreaterThan(0)
    }
  })

  it('connects every catalog entry to a real Electron workflow', () => {
    for (const id of expectedIds) {
      expect(suite).toContain(`defineUsageTest('${id}'`)
      expect(suite).toContain(`[linux-usage:${'${id}'}]`)
    }
    expect(suite).toContain("require('playwright/test')")
    expect(suite).toContain("require('node:crypto')")
    expect(suite).toContain('createSeededVaultFixture')
    expect(suite).toContain('launchElectron')
    expect(suite).toContain('page.screenshot')
    expect(suite).toContain('pixel-identical')
    expect(suite).toContain('checkpointState')
    expect(suite).toContain("page.on('pageerror'")
    expect(suite).toContain("message.type() === 'error'")
    expect(suite).toContain("getByTestId('muya-runtime-editor')")
    expect(suite).toContain("getByRole('textbox', { name: 'Note title' })")
    expect(suite).toContain('Preview updated by Rust editor 9173.')
    expect(suite).toContain('E2E Note Tools')
    expect(suite).toContain('Append CI marker')
    expect(helpers).toContain('installVisibleErrorObserver')
    expect(helpers).toContain('.en-addons-feedback.error')
    expect(helpers).toContain('[e2e-visible-addon-error]')
    expect(helpers).toContain('electron-main.js')
    expect(electronMain).toContain('tauri-preload-entry.js')
    expect(preloadEntry).toContain('tauri-preload.js')
  })

  it('does not bypass the real NoteEditorHost with a second full-window Rust editor', () => {
    expect(appPage).toContain('<app-shell v-if="init" />')
    expect(appPage).not.toContain('MuyaRuntimeEditor')
    expect(appPage).not.toContain('muya-runtime-production-editor')
    expect(appPage).not.toContain('muya-runtime-underlay')
  })

  it('runs against the production renderer under Xvfb and retains diagnostics', () => {
    expect(workflow).toContain('ubuntu-24.04')
    expect(workflow).toContain('xvfb-run --auto-servernum pnpm test:e2e')
    expect(workflow).toContain('test-results/**')
    expect(workflow).toContain('playwright-report/**')
    expect(workflow).toContain('e2e-results.json')
    expect(config).toContain("['json', { outputFile: path.join(resultsRoot, 'e2e-results.json') }]")
    expect(config).toContain("screenshot: exhaustiveEvidence ? 'on' : 'only-on-failure'")
    expect(config).toContain("trace: exhaustiveEvidence ? 'on' : 'retain-on-failure'")
  })
})
