import { test } from 'node:test'
import assert from 'node:assert/strict'
import { buildManifest } from '../manifest.mjs'

const run = {
  scenario: { id: 'scenario', viewport: { width: 1280, height: 840, scaleFactor: 1, deviceScaleFactor: 1, fullPage: false, colorScheme: 'light', locale: 'en-US' } },
  viewport: { width: 1280, height: 840, scaleFactor: 1, deviceScaleFactor: 1, fullPage: false, colorScheme: 'light', locale: 'en-US' },
  outputRoot: '/private/tmp/source-playwright-test', fixture: { id: 'fixture', roots: {}, files: [] }, runId: 'run', commandSha256: 'command', captureNonce: 'nonce', orchestratorRuntime: null
}

test('manifest keeps source-playwright provenance distinct from Tauri', () => {
  const manifest = buildManifest(run, [], [], 'passed')
  assert.equal(manifest.runtime, 'source-playwright')
  assert.equal(manifest.provenance.runtime, 'source-playwright')
  assert.equal(manifest.provenance.real, true)
  assert.equal(manifest.provenance.renderer, true)
  assert.equal(manifest.provenance.tauri, false)
  assert.equal(manifest.provenance.driver, 'playwright-electron')
  assert.equal(manifest.provenance.controlPlane, 'playwright-electron')
})
