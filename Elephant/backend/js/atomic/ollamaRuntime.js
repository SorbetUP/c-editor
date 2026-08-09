/* eslint-disable no-control-regex */
import path from 'path'
import os from 'os'
import { execFile, spawn } from 'child_process'
import { promisify } from 'util'

const execFileAsync = promisify(execFile)
const ANSI_PATTERN = /\u001B\[[0-?]*[ -/]*[@-~]/g

const uniq = (items = []) => [...new Set(items.filter(Boolean))]
const cleanOutput = (value = '') => String(value || '').replace(ANSI_PATTERN, '').trim()

const executableCandidates = () => {
  const home = os.homedir()
  return uniq([
    process.env.OLLAMA_BIN,
    'ollama',
    '/opt/homebrew/bin/ollama',
    '/usr/local/bin/ollama',
    home ? path.join(home, '.local', 'bin', 'ollama') : '',
    '/Applications/Ollama.app/Contents/Resources/ollama',
    '/Applications/Ollama.app/Contents/MacOS/Ollama'
  ])
}

export const formatOllamaError = (error = {}) => {
  const parts = [error?.message, error?.stderr, error?.stdout]
    .map((part) => cleanOutput(part))
    .filter(Boolean)
  return parts.join('\n') || 'Unknown Ollama error.'
}

export const buildOllamaInstallHint = (attempts = []) => {
  const broken = attempts
    .filter((attempt) => attempt.error && /no such file or directory|not found|ENOENT/i.test(attempt.error))
    .map((attempt) => `${attempt.command}: ${attempt.error}`)
    .slice(0, 4)

  const lines = [
    'Ollama CLI is not usable from ElephantNote.',
    'Install or repair Ollama, then restart ElephantNote.',
    'macOS options: install Ollama.app from ollama.com, or run `brew install ollama` and `brew services start ollama`.',
    'If `/Users/.../.local/bin/ollama` is a broken wrapper, remove it or point OLLAMA_BIN to a valid Ollama binary.'
  ]
  if (broken.length) lines.push(`Detected broken candidates: ${broken.join(' | ')}`)
  return lines.join(' ')
}

export const resolveOllamaBinary = async({ executor = execFileAsync } = {}) => {
  const attempts = []
  for (const command of executableCandidates()) {
    try {
      const result = await executor(command, ['--version'], { timeout: 5000 })
      const version = cleanOutput(result?.stdout || result?.stderr)
      return { available: true, command, version, attempts }
    } catch (error) {
      attempts.push({ command, error: formatOllamaError(error) })
    }
  }

  const error = new Error(buildOllamaInstallHint(attempts))
  error.code = 'OLLAMA_UNAVAILABLE'
  error.attempts = attempts
  throw error
}

export const parseOllamaList = (output = '') => String(output || '')
  .split(/\r?\n/)
  .slice(1)
  .map((line) => line.trim())
  .filter(Boolean)
  .map((line) => {
    const [name, id, size, ...modifiedParts] = line.split(/\s{2,}/)
    return { name: name || '', id: id || '', size: size || '', modified: modifiedParts.join(' ') }
  })
  .filter((model) => model.name)

export const listOllamaModels = async({ executor = execFileAsync } = {}) => {
  const runtime = await resolveOllamaBinary({ executor })
  const result = await executor(runtime.command, ['list'], { timeout: 10000 })
  return {
    provider: 'ollama',
    available: true,
    command: runtime.command,
    version: runtime.version,
    raw: result.stdout || '',
    models: parseOllamaList(result.stdout || '')
  }
}

const progressFromLine = (line = '', fallbackPercent = 0) => {
  const value = cleanOutput(line)
  if (!value) return null

  const percentMatch = value.match(/(\d{1,3})%/)
  if (percentMatch) {
    return {
      phase: 'downloading',
      percent: Math.max(fallbackPercent, Math.min(95, Number(percentMatch[1]) || fallbackPercent)),
      message: value,
      line: value
    }
  }

  if (/pulling manifest/i.test(value)) return { phase: 'manifest', percent: Math.max(fallbackPercent, 5), message: 'Pulling model manifest…', line: value }
  if (/pulling .*config/i.test(value)) return { phase: 'metadata', percent: Math.max(fallbackPercent, 12), message: value, line: value }
  if (/verifying/i.test(value)) return { phase: 'verifying', percent: Math.max(fallbackPercent, 96), message: 'Verifying model checksum…', line: value }
  if (/writing/i.test(value)) return { phase: 'writing', percent: Math.max(fallbackPercent, 98), message: 'Writing model manifest…', line: value }
  if (/success/i.test(value)) return { phase: 'done', percent: 100, message: 'Model installed successfully.', line: value }

  return { phase: 'running', percent: fallbackPercent, message: value, line: value }
}

export const pullOllamaModel = async({ model, onProgress, executor = execFileAsync, timeoutMs = 30 * 60 * 1000 } = {}) => {
  const modelName = String(model || '').trim()
  if (!modelName) throw new Error('Ollama model name is required.')

  const runtime = await resolveOllamaBinary({ executor })
  let lastPercent = 0
  let output = ''
  let errorOutput = ''

  const emit = (patch = {}) => {
    const payload = {
      provider: 'ollama',
      model: modelName,
      command: runtime.command,
      phase: 'running',
      percent: lastPercent,
      message: '',
      ...patch,
      updatedAt: new Date().toISOString()
    }
    if (Number.isFinite(Number(payload.percent))) lastPercent = Math.max(lastPercent, Number(payload.percent))
    onProgress?.(payload)
  }

  emit({ phase: 'starting', percent: 2, message: `Starting Ollama pull for ${modelName}…` })

  return new Promise((resolve, reject) => {
    let settled = false
    const settleResolve = (value) => {
      if (settled) return
      settled = true
      resolve(value)
    }
    const settleReject = (error) => {
      if (settled) return
      settled = true
      reject(error)
    }
    const child = spawn(runtime.command, ['pull', modelName], { env: process.env, windowsHide: true })

    const timeout = setTimeout(() => {
      child.kill('SIGTERM')
      const error = new Error(`Ollama pull timed out after ${Math.round(timeoutMs / 60000)} minutes.`)
      error.code = 'OLLAMA_PULL_TIMEOUT'
      settleReject(error)
    }, timeoutMs)

    const handleChunk = (chunk, streamName) => {
      const text = String(chunk || '')
      if (streamName === 'stderr') errorOutput += text
      else output += text
      for (const rawLine of text.split(/[\r\n]+/)) {
        const parsed = progressFromLine(rawLine, lastPercent)
        if (parsed) emit(parsed)
      }
    }

    child.stdout?.on('data', (chunk) => handleChunk(chunk, 'stdout'))
    child.stderr?.on('data', (chunk) => handleChunk(chunk, 'stderr'))

    child.on('error', (error) => {
      clearTimeout(timeout)
      emit({ phase: 'error', error: formatOllamaError(error), message: formatOllamaError(error) })
      settleReject(error)
    })

    child.on('close', (code) => {
      clearTimeout(timeout)
      if (code === 0) {
        emit({ phase: 'done', percent: 100, message: `${modelName} is installed and ready.` })
        settleResolve({
          provider: 'ollama',
          command: runtime.command,
          model: modelName,
          stdout: output,
          stderr: errorOutput
        })
        return
      }

      const details = cleanOutput(errorOutput || output)
      const error = new Error(details || `Ollama pull failed with exit code ${code}.`)
      error.code = 'OLLAMA_PULL_FAILED'
      error.exitCode = code
      error.stdout = output
      error.stderr = errorOutput
      emit({ phase: 'error', error: formatOllamaError(error), message: formatOllamaError(error) })
      settleReject(error)
    })
  })
}
