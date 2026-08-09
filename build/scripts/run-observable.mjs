import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url))
const repositoryRoot = path.resolve(scriptDirectory, '../..')
const separatorIndex = process.argv.indexOf('--')

if (separatorIndex < 0 || separatorIndex === process.argv.length - 1) {
  console.error('Usage: node build/scripts/run-observable.mjs [--label name] -- command [arguments...]')
  process.exit(2)
}

const optionArguments = process.argv.slice(2, separatorIndex)
const commandArguments = process.argv.slice(separatorIndex + 1)
const labelIndex = optionArguments.indexOf('--label')
const label = String(labelIndex >= 0 ? optionArguments[labelIndex + 1] : path.basename(commandArguments[0]))
  .trim()
  .replace(/[^a-zA-Z0-9._-]+/g, '-') || 'command'
const command = commandArguments[0]
const args = commandArguments.slice(1)
const startedAt = new Date()
const runId = `${startedAt.toISOString().replaceAll(':', '-').replaceAll('.', '-')}-${label}-${process.pid}`
const outputRoot = path.resolve(
  repositoryRoot,
  process.env.ELEPHANT_OBSERVABILITY_DIR || 'test-results/observability'
)
const rawLogPath = path.join(outputRoot, `${runId}.log`)
const eventLogPath = path.join(outputRoot, `${runId}.ndjson`)

fs.mkdirSync(outputRoot, { recursive: true })
const rawLog = fs.createWriteStream(rawLogPath, { flags: 'a' })
const eventLog = fs.createWriteStream(eventLogPath, { flags: 'a' })
let sequence = 0
let closed = false

const secretNamePattern = /(token|secret|password|passwd|passphrase|private.?key|api.?key|cookie|authorization|credential|session|dsn)/i
const secretValues = Object.entries(process.env)
  .filter(([name, value]) => secretNamePattern.test(name) && String(value || '').length >= 4)
  .map(([, value]) => String(value))
  .sort((left, right) => right.length - left.length)
const redactText = (value) => {
  let output = String(value ?? '')
  for (const secret of secretValues) output = output.replaceAll(secret, '[REDACTED_SECRET_VALUE]')
  return output
}
const visibleEnvironment = Object.fromEntries(
  Object.entries(process.env)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([name, value]) => [name, secretNamePattern.test(name) ? '[REDACTED_SECRET_VALUE]' : redactText(value)])
)

const writeEvent = (type, detail = {}) => {
  const event = {
    sequence: ++sequence,
    timestamp: new Date().toISOString(),
    runId,
    label,
    type,
    ...detail
  }
  eventLog.write(`${JSON.stringify(event)}\n`)
  return event
}

const writeOutput = (stream, chunk) => {
  const text = redactText(Buffer.isBuffer(chunk) ? chunk.toString('utf8') : chunk)
  const buffer = Buffer.from(text)
  if (stream === 'stdout') process.stdout.write(buffer)
  else process.stderr.write(buffer)
  rawLog.write(buffer)
  writeEvent('output', {
    stream,
    bytes: buffer.byteLength,
    text
  })
}

const startMetadata = {
  pid: process.pid,
  parentPid: process.ppid,
  cwd: process.cwd(),
  repositoryRoot,
  command,
  args,
  argv: commandArguments,
  platform: process.platform,
  architecture: process.arch,
  node: process.version,
  environment: visibleEnvironment,
  redactedCredentialValueCount: secretValues.length,
  rawLogPath,
  eventLogPath
}

writeEvent('run-start', startMetadata)
console.log(`[elephant-observe] START ${label}`)
console.log(`[elephant-observe] cwd=${startMetadata.cwd}`)
console.log(`[elephant-observe] command=${JSON.stringify(commandArguments)}`)
console.log(`[elephant-observe] raw=${rawLogPath}`)
console.log(`[elephant-observe] events=${eventLogPath}`)
console.log(`[elephant-observe] environment=${JSON.stringify(visibleEnvironment)}`)

const child = spawn(command, args, {
  cwd: process.cwd(),
  env: {
    ...process.env,
    CARGO_TERM_COLOR: process.env.CARGO_TERM_COLOR || 'always',
    FORCE_COLOR: process.env.FORCE_COLOR || '1'
  },
  shell: false,
  stdio: ['inherit', 'pipe', 'pipe']
})

writeEvent('child-spawned', { childPid: child.pid })
child.stdout.on('data', (chunk) => writeOutput('stdout', chunk))
child.stderr.on('data', (chunk) => writeOutput('stderr', chunk))

const forwardedSignals = ['SIGINT', 'SIGTERM', 'SIGHUP']
for (const signal of forwardedSignals) {
  process.on(signal, () => {
    writeEvent('signal-received', { signal, childPid: child.pid })
    if (!child.killed) child.kill(signal)
  })
}

const closeLogs = (finalEvent) => {
  if (closed) return
  closed = true
  writeEvent(finalEvent.type, finalEvent.detail)
  rawLog.end()
  eventLog.end()
}

child.on('error', (error) => {
  const detail = {
    name: error?.name || 'Error',
    message: redactText(error?.message || String(error)),
    stack: redactText(error?.stack || '')
  }
  console.error(`[elephant-observe] SPAWN ERROR ${JSON.stringify(detail)}`)
  closeLogs({ type: 'run-spawn-error', detail })
  process.exitCode = 1
})

child.on('exit', (code, signal) => {
  const finishedAt = new Date()
  const detail = {
    childPid: child.pid,
    code,
    signal,
    startedAt: startedAt.toISOString(),
    finishedAt: finishedAt.toISOString(),
    durationMs: finishedAt.getTime() - startedAt.getTime(),
    success: code === 0 && !signal
  }
  closeLogs({ type: 'run-finish', detail })
  console.log(`[elephant-observe] END ${label} ${JSON.stringify(detail)}`)
  if (signal) {
    process.kill(process.pid, signal)
    return
  }
  process.exitCode = Number.isInteger(code) ? code : 1
})
