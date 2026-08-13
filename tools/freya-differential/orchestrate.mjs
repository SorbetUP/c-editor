#!/usr/bin/env node

import { randomUUID } from 'node:crypto'
import { spawn } from 'node:child_process'
import { copyFile, link, mkdir, readdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { compareMetadata, detectCopiedEvidence, validateManifest } from './lib/evidence.mjs'
import { sha256, writeJson } from './lib/common.mjs'
import { loadScenario, materializeFixture, validateScenario } from './lib/scenario.mjs'

const PROJECT_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const DEFAULT_SCENARIO = path.join(PROJECT_ROOT, 'migration/freya/differential-scenarios.json')
const DEFAULT_COMPARE = path.join(PROJECT_ROOT, 'tools/freya-differential/compare.mjs')

function usage () {
  return `Usage:
  node tools/freya-differential/orchestrate.mjs [options]

Required:
  --tauri-command <command>     Real Tauri capture command
  --freya-command <command>     Real Freya/freya-testing capture command

Options:
  --scenario <file>              Scenario JSON (default: migration/freya/differential-scenarios.json)
  --output <directory>           Empty output directory for both runs
  --compare <file>               PNG comparator (default: tools/freya-differential/compare.mjs)
  --threshold <0..1>             Explicit per-channel pixel tolerance (default: 0)
  --max-different-ratio <0..1>   Explicit changed-pixel tolerance (default: 0)
  --timeout-ms <milliseconds>    Per-runtime command timeout (default: 120000)
  --help                         Show this help
`
}

function parseNumber (name, value, minimum = 0, maximum = 1) {
  const number = Number(value)
  if (!Number.isFinite(number) || number < minimum || number > maximum) throw new Error(`${name} must be between ${minimum} and ${maximum}`)
  return number
}

export function parseArgs (argv) {
  const options = {
    scenario: DEFAULT_SCENARIO,
    compare: DEFAULT_COMPARE,
    output: path.join(PROJECT_ROOT, 'test-results/freya-differential/orchestrated'),
    threshold: 0,
    maxDifferentRatio: 0,
    timeoutMs: 120000,
    tauriCommand: null,
    freyaCommand: null
  }
  for (let index = 0; index < argv.length; index += 1) {
    const name = argv[index]
    if (name === '--help' || name === '-h') return { help: true }
    const value = argv[index + 1]
    if (!value || value.startsWith('--')) throw new Error(`Missing value for ${name}`)
    if (name === '--scenario') options.scenario = value
    else if (name === '--tauri-command') options.tauriCommand = value
    else if (name === '--freya-command') options.freyaCommand = value
    else if (name === '--output') options.output = value
    else if (name === '--compare') options.compare = value
    else if (name === '--threshold') options.threshold = parseNumber(name, value)
    else if (name === '--max-different-ratio') options.maxDifferentRatio = parseNumber(name, value)
    else if (name === '--timeout-ms') options.timeoutMs = parseNumber(name, value, 1, Number.MAX_SAFE_INTEGER)
    else throw new Error(`Unknown option ${name}`)
    index += 1
  }
  if (!options.tauriCommand || !options.freyaCommand) throw new Error('--tauri-command and --freya-command are required')
  return options
}

async function ensureEmptyOutput (directory) {
  await mkdir(directory, { recursive: true })
  const entries = await readdir(directory)
  if (entries.length) throw new Error(`Output directory must be empty: ${directory}`)
}

async function runCommand (command, runtime, context) {
  const { outputRoot, scenarioPath, fixtureRoot, runId, timeoutMs } = context
  const commandSha256 = sha256(Buffer.from(command, 'utf8'))
  const env = {
    ...process.env,
    DIFFERENTIAL_RUNTIME: runtime,
    DIFFERENTIAL_SCENARIO_PATH: scenarioPath,
    DIFFERENTIAL_OUTPUT_DIR: outputRoot,
    DIFFERENTIAL_FIXTURE_ROOT: fixtureRoot,
    DIFFERENTIAL_RUN_ID: runId,
    DIFFERENTIAL_COMMAND_SHA256: commandSha256,
    DIFFERENTIAL_CAPTURE_NONCE: randomUUID(),
    DIFFERENTIAL_EXPECTED_VIEWPORT_JSON: JSON.stringify(context.scenario.viewport)
  }
  const startedAt = Date.now()
  const result = await new Promise((resolve) => {
    const child = spawn(command, { cwd: PROJECT_ROOT, env, shell: true })
    let stdout = ''
    let stderr = ''
    let timedOut = false
    const timer = setTimeout(() => {
      timedOut = true
      child.kill('SIGTERM')
    }, timeoutMs)
    child.stdout.on('data', (chunk) => { stdout += chunk })
    child.stderr.on('data', (chunk) => { stderr += chunk })
    child.on('close', (code, signal) => {
      clearTimeout(timer)
      resolve({ code, signal, timedOut, stdout, stderr, durationMs: Date.now() - startedAt })
    })
    child.on('error', (error) => {
      clearTimeout(timer)
      resolve({ code: null, signal: null, timedOut, stdout, stderr: `${stderr}${error.stack ?? error.message}`, durationMs: Date.now() - startedAt })
    })
  })
  await writeFile(path.join(context.root, `${runtime}.command.log`), [
    `command=${command}`,
    `runtime=${runtime}`,
    `runId=${runId}`,
    `exitCode=${result.code ?? 'null'}`,
    `signal=${result.signal ?? 'none'}`,
    `timedOut=${result.timedOut}`,
    `durationMs=${result.durationMs}`,
    '--- stdout ---',
    result.stdout,
    '--- stderr ---',
    result.stderr
  ].join('\n'), 'utf8')
  return { ...result, commandSha256, captureNonce: env.DIFFERENTIAL_CAPTURE_NONCE }
}

async function loadManifest (outputRoot, runtime) {
  try {
    return JSON.parse(await readFile(path.join(outputRoot, 'manifest.json'), 'utf8'))
  } catch (error) {
    return { __error: `${runtime} manifest.json is missing or invalid: ${error.message}` }
  }
}

async function stageFrames (root, evidence, runtime) {
  const comparisonRoot = path.join(root, 'comparison', runtime)
  for (const frame of evidence.frames) {
    const destination = path.join(comparisonRoot, frame.checkpoint, 'frames', `frame-${frame.index.toString().padStart(3, '0')}.png`)
    await mkdir(path.dirname(destination), { recursive: true })
    try {
      await link(frame.path, destination)
    } catch {
      await copyFile(frame.path, destination)
    }
  }
  return comparisonRoot
}

async function runCompare (comparePath, tauriRoot, freyaRoot, context) {
  const reportPath = path.join(context.root, 'comparison-report.json')
  const diffDirectory = path.join(context.root, 'diffs')
  const args = [
    comparePath,
    tauriRoot,
    freyaRoot,
    '--reference-label', 'tauri',
    '--candidate-label', 'freya',
    '--threshold', String(context.threshold),
    '--max-different-ratio', String(context.maxDifferentRatio),
    '--report', reportPath,
    '--diff-dir', diffDirectory
  ]
  const result = await new Promise((resolve) => {
    const child = spawn(process.execPath, args, { cwd: PROJECT_ROOT })
    let stdout = ''
    let stderr = ''
    child.stdout.on('data', (chunk) => { stdout += chunk })
    child.stderr.on('data', (chunk) => { stderr += chunk })
    child.on('close', (code, signal) => resolve({ code, signal, stdout, stderr }))
    child.on('error', (error) => resolve({ code: null, signal: null, stdout, stderr: error.message }))
  })
  let report = null
  try { report = JSON.parse(await readFile(reportPath, 'utf8')) } catch (error) {
    report = { status: 'not-comparable', issues: [{ type: 'comparator', message: error.message }] }
  }
  return { ...result, report }
}

function commandIssue (runtime, result) {
  if (result.code === 0) return null
  return {
    type: 'command-failed',
    runtime,
    message: `${runtime} capture command exited with ${result.code ?? 'null'}${result.signal ? ` (${result.signal})` : ''}${result.timedOut ? ' after timeout' : ''}`
  }
}

export async function orchestrate (options) {
  const root = path.resolve(options.output)
  const scenarioPath = path.resolve(options.scenario)
  const comparePath = path.resolve(options.compare)
  const runId = `differential-${Date.now()}-${process.pid}-${randomUUID()}`
  const report = {
    schemaVersion: 1,
    status: 'invalid-input',
    runId,
    scenario: { path: scenarioPath, id: null, actions: 0 },
    runtimes: {},
    summary: { actions: { expected: 0, compared: 0 }, frames: { compared: 0 }, issues: 0 },
    options: { threshold: options.threshold, maxDifferentRatio: options.maxDifferentRatio },
    issues: [],
    comparison: null,
    artifacts: { root }
  }
  try {
    await ensureEmptyOutput(root)
    const scenario = await loadScenario(scenarioPath)
    report.scenario = { path: scenarioPath, id: scenario.id, actions: scenario.actions.length }
    report.summary.actions.expected = scenario.actions.length
    const runtimeEvidence = {}
    for (const runtime of ['tauri', 'freya']) {
      const outputRoot = path.join(root, runtime)
      const fixtureRoot = path.join(outputRoot, 'fixture')
      await mkdir(outputRoot, { recursive: true })
      const fixture = await materializeFixture(scenario, fixtureRoot)
      const run = await runCommand(options[`${runtime}Command`], runtime, {
        root, outputRoot, fixtureRoot, scenarioPath, runId: `${runId}-${runtime}`,
        timeoutMs: options.timeoutMs, scenario
      })
      report.runtimes[runtime] = { commandExitCode: run.code, signal: run.signal, durationMs: run.durationMs, log: path.join(root, `${runtime}.command.log`) }
      const commandFailure = commandIssue(runtime, run)
      if (commandFailure) report.issues.push(commandFailure)
      const manifest = await loadManifest(outputRoot, runtime)
      if (manifest.__error) {
        report.issues.push({ type: 'manifest', runtime, message: manifest.__error })
        runtimeEvidence[runtime] = { issues: [], normalized: { viewport: null, actions: [], checkpoints: [] }, frames: [], provenance: null }
        continue
      }
      runtimeEvidence[runtime] = await validateManifest(manifest, {
        runtime, outputRoot, runId: `${runId}-${runtime}`,
        commandSha256: run.commandSha256, captureNonce: run.captureNonce, scenario, fixture
      })
      report.runtimes[runtime].manifest = path.join(outputRoot, 'manifest.json')
      report.issues.push(...runtimeEvidence[runtime].issues.map((item) => ({ runtime, ...item })))
    }
    const tauri = runtimeEvidence.tauri
    const freya = runtimeEvidence.freya
    if (tauri && freya) {
      report.issues.push(...compareMetadata(tauri, freya, scenario))
      report.issues.push(...await detectCopiedEvidence(tauri, freya))
      report.summary.actions.compared = Math.min(tauri.normalized.actions.length, freya.normalized.actions.length)
      const tauriComparisonRoot = await stageFrames(root, tauri, 'tauri')
      const freyaComparisonRoot = await stageFrames(root, freya, 'freya')
      const comparison = await runCompare(comparePath, tauriComparisonRoot, freyaComparisonRoot, { root, threshold: options.threshold, maxDifferentRatio: options.maxDifferentRatio })
      report.comparison = comparison.report
      report.summary.frames.compared = comparison.report.summary?.files?.compared ?? 0
      if (comparison.code !== 0) report.issues.push({ type: 'pixel-comparison', message: `compare.mjs exited with ${comparison.code ?? 'null'}` })
    }
    report.summary.issues = report.issues.length
    report.status = report.issues.length === 0 && report.comparison?.status === 'ok' ? 'ok' : 'mismatch'
  } catch (error) {
    report.issues.push({ type: 'orchestrator', message: error.message })
    report.summary.issues = report.issues.length
  }
  await writeJson(path.join(root, 'orchestration-report.json'), report)
  return report
}

export { validateScenario }

async function main () {
  try {
    const options = parseArgs(process.argv.slice(2))
    if (options.help) {
      process.stdout.write(usage())
      return
    }
    const report = await orchestrate(options)
    process.stdout.write(`${JSON.stringify({ status: report.status, report: path.join(path.resolve(options.output), 'orchestration-report.json'), issues: report.issues.length }, null, 2)}\n`)
    process.exitCode = report.status === 'ok' ? 0 : report.issues.some((item) => ['orchestrator', 'command-failed', 'manifest', 'provenance', 'control-plane', 'fixture-mismatch'].includes(item.type)) ? 2 : 1
  } catch (error) {
    process.stderr.write(`freya-differential orchestrator: ${error.message}\n\n${usage()}`)
    process.exitCode = 2
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) await main()
