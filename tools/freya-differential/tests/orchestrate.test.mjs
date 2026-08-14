import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { spawn } from 'node:child_process'
import test from 'node:test'
import { validateScenario } from '../orchestrate.mjs'

const projectRoot = path.resolve(import.meta.dirname, '../../..')
const runnerPath = path.join(projectRoot, 'tools/freya-differential/orchestrate.mjs')
const captureCommandPath = path.join(projectRoot, 'tools/freya-differential/tests/fixtures/capture-command.mjs')
const scenarioPath = path.join(projectRoot, 'migration/freya/differential-scenarios.json')

function shellQuote (value) {
  return `'${String(value).replaceAll("'", "'\\''")}'`
}

function captureCommand (runtime, mutation) {
  return `${shellQuote(process.execPath)} ${shellQuote(captureCommandPath)} --configured-runtime ${shellQuote(runtime)} --mutation ${shellQuote(mutation)}`
}

async function runOrchestrator ({ tauriMutation = 'valid', freyaMutation = 'valid' } = {}) {
  const output = await mkdtemp(path.join(tmpdir(), 'freya-differential-orchestrator-'))
  const args = [
    runnerPath,
    '--scenario', scenarioPath,
    '--tauri-command', captureCommand('tauri', tauriMutation),
    '--freya-command', captureCommand('freya', freyaMutation),
    '--output', output,
    '--timeout-ms', '30000'
  ]
  const result = await new Promise((resolve) => {
    const child = spawn(process.execPath, args, { cwd: projectRoot })
    let stdout = ''
    let stderr = ''
    child.stdout.on('data', (chunk) => { stdout += chunk })
    child.stderr.on('data', (chunk) => { stderr += chunk })
    child.on('close', (code, signal) => resolve({ code, signal, stdout, stderr }))
  })
  let report = null
  try {
    report = JSON.parse(await readFile(path.join(output, 'orchestration-report.json'), 'utf8'))
  } catch (error) {
    if (result.code === 0) throw error
  }
  return { ...result, output, report }
}

test('rejects a capture with a missing logical action', async () => {
  const result = await runOrchestrator({ freyaMutation: 'missing-action' })
  try {
    assert.notEqual(result.code, 0)
    assert.ok(result.report.issues.some((issue) => issue.type === 'action-missing'))
  } finally {
    await rm(result.output, { recursive: true, force: true })
  }
})

test('rejects temporal frames with a missing or misaligned index', async () => {
  const result = await runOrchestrator({ tauriMutation: 'misaligned-frame' })
  try {
    assert.notEqual(result.code, 0)
    assert.ok(result.report.issues.some((issue) => issue.type === 'frame-sequence'))
  } finally {
    await rm(result.output, { recursive: true, force: true })
  }
})

test('rejects synthetic or copied evidence provenance', async () => {
  const result = await runOrchestrator({ freyaMutation: 'faked-evidence' })
  try {
    assert.notEqual(result.code, 0)
    assert.ok(result.report.issues.some((issue) => issue.type === 'synthetic-evidence'))
  } finally {
    await rm(result.output, { recursive: true, force: true })
  }
})

test('rejects frames that point at the same source artifact', async () => {
  const result = await runOrchestrator({ freyaMutation: 'copied-evidence' })
  try {
    assert.notEqual(result.code, 0)
    assert.ok(result.report.issues.some((issue) => issue.type === 'copied-evidence'))
    assert.ok(result.report.issues.some((issue) => issue.type === 'source-path-escape'))
  } finally {
    await rm(result.output, { recursive: true, force: true })
  }
})

test('rejects a required stale not-exposed mapping when source proof exists', async () => {
  const scenario = JSON.parse(await readFile(scenarioPath, 'utf8'))
  const action = scenario.actions.find((candidate) => candidate.id === 'edit-alpha-note')
  action.target.freya = {
    status: 'not-exposed',
    selector: null,
    strategy: 'accessibility-label',
    label: 'Paragraph',
    source: 'Elephant/freya/src/app/editor_view.rs#EditableInlineBlock',
    test: 'Elephant/freya/tests/shell_freya_testing.rs#editor_keystrokes_update_the_real_muya_document_and_save_to_the_vault'
  }
  const issues = validateScenario(scenario)
  assert.ok(issues.some((issue) => issue.type === 'stale-mapping'))
})

test('rejects a Freya target label that differs from the proven source label', async () => {
  const scenario = JSON.parse(await readFile(scenarioPath, 'utf8'))
  const action = scenario.actions.find((candidate) => candidate.id === 'close-alpha-note')
  action.target.freya.label = 'Close'
  const issues = validateScenario(scenario)
  assert.ok(issues.some((issue) => issue.type === 'source-label-mismatch'))
})

test('rejects viewport, state, and vault mismatches before accepting raster equality', async () => {
  for (const [runtime, mutation, expectedType] of [
    ['tauri', 'viewport-mismatch', 'viewport-mismatch'],
    ['freya', 'state-mismatch', 'state-mismatch'],
    ['freya', 'editor-text-mismatch', 'state-mismatch'],
    ['freya', 'vault-mismatch', 'fixture-mismatch']
  ]) {
    const result = await runOrchestrator({
      tauriMutation: runtime === 'tauri' ? mutation : 'valid',
      freyaMutation: runtime === 'freya' ? mutation : 'valid'
    })
    try {
      assert.notEqual(result.code, 0)
      assert.ok(result.report.issues.some((issue) => issue.type === expectedType), `${mutation} must be rejected`)
    } finally {
      await rm(result.output, { recursive: true, force: true })
    }
  }
})

test('rejects captures whose PNG dimensions do not match the declared logical viewport', async () => {
  const output = await mkdtemp(path.join(tmpdir(), 'freya-differential-frame-viewport-'))
  try {
    const scenario = JSON.parse(await readFile(scenarioPath, 'utf8'))
    const fixtureRoot = path.join(output, 'fixture')
    const runtimeRoot = path.join(output, 'freya')
    await mkdir(path.join(fixtureRoot, 'vault'), { recursive: true })
    await mkdir(runtimeRoot, { recursive: true })
    const child = spawn(process.execPath, [captureCommandPath, '--configured-runtime', 'freya', '--mutation', 'valid'], {
      cwd: projectRoot,
      env: {
        ...process.env,
        DIFFERENTIAL_RUNTIME: 'freya',
        DIFFERENTIAL_OUTPUT_DIR: runtimeRoot,
        DIFFERENTIAL_SCENARIO_PATH: scenarioPath,
        DIFFERENTIAL_FIXTURE_ROOT: fixtureRoot,
        DIFFERENTIAL_RUN_ID: 'frame-viewport-test',
        DIFFERENTIAL_COMMAND_SHA256: 'frame-viewport-command',
        DIFFERENTIAL_CAPTURE_NONCE: 'frame-viewport-nonce'
      }
    })
    await new Promise((resolve, reject) => {
      child.once('error', reject)
      child.once('close', (code) => code === 0 ? resolve() : reject(new Error(`fixture command exited with ${code}`)))
    })
    const manifest = JSON.parse(await readFile(path.join(runtimeRoot, 'manifest.json'), 'utf8'))
    const firstFrame = manifest.checkpoints[0].frames[0]
    firstFrame.path = firstFrame.path.replace('.png', '-wrong.png')
    const original = await readFile(path.join(runtimeRoot, firstFrame.path.replace('-wrong.png', '.png')))
    const wrongPng = Buffer.from(original)
    wrongPng.writeUInt32BE(scenario.viewport.width - 1, 16)
    await writeFile(path.join(runtimeRoot, firstFrame.path), wrongPng)
    firstFrame.sha256 = createHash('sha256').update(wrongPng).digest('hex')
    await writeFile(path.join(runtimeRoot, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`)

    const { validateManifest } = await import('../lib/evidence.mjs')
    const evidence = await validateManifest(manifest, {
      runtime: 'freya',
      outputRoot: runtimeRoot,
      runId: 'frame-viewport-test',
      commandSha256: 'frame-viewport-command',
      captureNonce: 'frame-viewport-nonce',
      scenario,
      fixture: { id: scenario.fixture.id, files: [] }
    })
    assert.ok(evidence.issues.some((issue) => issue.type === 'frame-viewport-mismatch'))
  } finally {
    await rm(output, { recursive: true, force: true })
  }
})

test('fails when aligned PNG pixels differ even though action/state metadata matches', async () => {
  const result = await runOrchestrator({ freyaMutation: 'pixel-difference' })
  try {
    assert.notEqual(result.code, 0)
    assert.ok(result.report.issues.some((issue) => issue.type === 'pixel-comparison'))
    assert.equal(result.report.comparison.status, 'mismatch')
  } finally {
    await rm(result.output, { recursive: true, force: true })
  }
})

test('runs two independent captures and compares aligned valid evidence', async () => {
  const result = await runOrchestrator()
  try {
    assert.equal(result.code, 0, `${result.stdout}\n${result.stderr}`)
    assert.equal(result.report.status, 'ok')
    assert.equal(result.report.comparison.status, 'ok')
    assert.equal(result.report.summary.actions.compared, result.report.scenario.actions)
    assert.equal(result.report.summary.frames.compared, result.report.comparison.summary.files.compared)
  } finally {
    await rm(result.output, { recursive: true, force: true })
  }
})
