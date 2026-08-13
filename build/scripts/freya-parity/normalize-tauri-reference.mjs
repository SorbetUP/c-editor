#!/usr/bin/env node
import { execFileSync } from 'node:child_process'
import { mkdir, readFile, stat, writeFile } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'

const args = process.argv.slice(2)
const stringArg = (flag, fallback = null) => {
  const index = args.indexOf(flag)
  return index >= 0 ? args[index + 1] : fallback
}

const repoRoot = path.resolve(stringArg('--repo-root', '.tauri-reference'))
const expectedSha = stringArg('--expected-sha', process.env.TAURI_REFERENCE_SHA || '')
const output = path.resolve(stringArg('--output', 'test-results/freya-parity/tauri-reference-normalization.json'))
const configPath = path.join(repoRoot, 'Elephant/backend/tauri/tauri.conf.json')
const materializedOfficialAddons = path.join(repoRoot, 'Elephant/backend/tauri/resources/official-addons')

const actualSha = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repoRoot, encoding: 'utf8' }).trim()
if (!expectedSha) throw new Error('INFRA_ERROR: --expected-sha is required for Tauri reference normalization')
if (actualSha !== expectedSha) {
  throw new Error(`INFRA_ERROR: refusing to normalize unexpected Tauri reference: expected ${expectedSha}, got ${actualSha}`)
}

const materializedStat = await stat(materializedOfficialAddons).catch(() => null)
if (!materializedStat?.isDirectory()) {
  throw new Error(`INFRA_ERROR: official addon resources were not materialized before Tauri config normalization: ${materializedOfficialAddons}`)
}
for (const required of ['catalog.json', 'official']) {
  const requiredStat = await stat(path.join(materializedOfficialAddons, required)).catch(() => null)
  if (!requiredStat) throw new Error(`INFRA_ERROR: materialized official addon resources are missing ${required}`)
}

const originalText = await readFile(configPath, 'utf8')
const config = JSON.parse(originalText)
const expectedFrontendDist = '/Users/sorbet/Desktop/Dev/c-editor/build/out/renderer'
const expectedOfficialAddons = '/Users/sorbet/Desktop/Dev/c-editor/Elephant/backend/tauri/resources/official-addons'
const portableOfficialAddons = 'resources/official-addons'

if (config.build?.frontendDist !== expectedFrontendDist) {
  throw new Error(`INFRA_ERROR: immutable reference frontendDist changed unexpectedly: ${JSON.stringify(config.build?.frontendDist)}`)
}
if (!Array.isArray(config.bundle?.resources) || config.bundle.resources.length !== 1 || config.bundle.resources[0] !== expectedOfficialAddons) {
  throw new Error(`INFRA_ERROR: immutable reference bundle.resources changed unexpectedly: ${JSON.stringify(config.bundle?.resources)}`)
}

// The immutable reference contains developer-machine absolute paths. Keep the
// exact same renderer output and official addon payload, but make both paths
// repository-relative after the official reference materialization scripts ran.
// No renderer/application source is changed and HEAD remains pinned to expectedSha.
config.build.frontendDist = '../../../build/out/renderer'
config.bundle.resources = [portableOfficialAddons]

const normalizedText = `${JSON.stringify(config, null, 2)}\n`
await writeFile(configPath, normalizedText)
const diff = execFileSync('git', ['diff', '--', 'Elephant/backend/tauri/tauri.conf.json'], {
  cwd: repoRoot,
  encoding: 'utf8'
})
if (!diff.includes('frontendDist') || !diff.includes('resources')) {
  throw new Error(`INFRA_ERROR: Tauri reference normalization did not produce the expected config-only diff:\n${diff}`)
}

const changedFiles = execFileSync('git', ['status', '--short'], { cwd: repoRoot, encoding: 'utf8' })
  .trim().split('\n').filter(Boolean)
if (changedFiles.length !== 1 || !changedFiles[0].endsWith('Elephant/backend/tauri/tauri.conf.json')) {
  throw new Error(`INFRA_ERROR: reference normalization dirtied unexpected tracked files: ${JSON.stringify(changedFiles)}`)
}

const manifest = {
  schemaVersion: 2,
  referenceSha: actualSha,
  purpose: 'CI portability normalization only; no renderer or application source changes',
  changedFile: 'Elephant/backend/tauri/tauri.conf.json',
  materializedOfficialAddons: 'Elephant/backend/tauri/resources/official-addons',
  changes: {
    frontendDist: { from: expectedFrontendDist, to: config.build.frontendDist },
    bundleResources: { from: [expectedOfficialAddons], to: [portableOfficialAddons] }
  },
  rationale: {
    frontendDist: 'Replace an absolute developer-machine path with the same repository-relative build output.',
    bundleResources: 'Preserve the official addon payload produced by the reference prepare-tauri-addon-resources script while replacing its absolute developer-machine path.'
  },
  gitDiff: diff
}
await mkdir(path.dirname(output), { recursive: true })
await writeFile(output, `${JSON.stringify(manifest, null, 2)}\n`)
console.log(`[freya-parity] normalized immutable Tauri reference ${actualSha} for portable Linux dev capture`)
console.log(diff)
