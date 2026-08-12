#!/usr/bin/env node
import { execFileSync } from 'node:child_process'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
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

const actualSha = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repoRoot, encoding: 'utf8' }).trim()
if (!expectedSha) throw new Error('INFRA_ERROR: --expected-sha is required for Tauri reference normalization')
if (actualSha !== expectedSha) {
  throw new Error(`INFRA_ERROR: refusing to normalize unexpected Tauri reference: expected ${expectedSha}, got ${actualSha}`)
}

const originalText = await readFile(configPath, 'utf8')
const config = JSON.parse(originalText)
const expectedFrontendDist = '/Users/sorbet/Desktop/Dev/c-editor/build/out/renderer'
const expectedOfficialAddons = '/Users/sorbet/Desktop/Dev/c-editor/Elephant/backend/tauri/resources/official-addons'

if (config.build?.frontendDist !== expectedFrontendDist) {
  throw new Error(`INFRA_ERROR: immutable reference frontendDist changed unexpectedly: ${JSON.stringify(config.build?.frontendDist)}`)
}
if (!Array.isArray(config.bundle?.resources) || config.bundle.resources.length !== 1 || config.bundle.resources[0] !== expectedOfficialAddons) {
  throw new Error(`INFRA_ERROR: immutable reference bundle.resources changed unexpectedly: ${JSON.stringify(config.bundle?.resources)}`)
}

// The immutable reference contains developer-machine absolute packaging paths.
// `cargo tauri dev --config tauri.linux.conf.json` disables Linux bundling, so the
// untracked bundle resource is not part of the runtime surface being compared.
// Normalize only these non-portable config fields; application/rendering sources
// remain untouched and HEAD remains pinned to expectedSha (the worktree becomes dirty).
config.build.frontendDist = '../../../build/out/renderer'
config.bundle.resources = []

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
  throw new Error(`INFRA_ERROR: reference normalization dirtied unexpected files: ${JSON.stringify(changedFiles)}`)
}

const manifest = {
  schemaVersion: 1,
  referenceSha: actualSha,
  purpose: 'CI portability normalization only; no renderer or application source changes',
  changedFile: 'Elephant/backend/tauri/tauri.conf.json',
  changes: {
    frontendDist: { from: expectedFrontendDist, to: config.build.frontendDist },
    bundleResources: { from: [expectedOfficialAddons], to: [] }
  },
  rationale: {
    frontendDist: 'Replace an absolute developer-machine path with the same repository-relative build output.',
    bundleResources: 'The referenced official-addons directory is not tracked at the immutable reference SHA and Linux dev bundling is disabled by tauri.linux.conf.json.'
  },
  gitDiff: diff
}
await mkdir(path.dirname(output), { recursive: true })
await writeFile(output, `${JSON.stringify(manifest, null, 2)}\n`)
console.log(`[freya-parity] normalized immutable Tauri reference ${actualSha} for portable Linux dev capture`)
console.log(diff)
