import { execFile } from 'child_process'
import { promisify } from 'util'

const execFileAsync = promisify(execFile)

const normalizeAllowRun = (value) => value === true || value === 'true'

export const normalizeProgramEnvironments = (environments = {}) => {
  return Object.fromEntries(Object.entries(environments || {}).map(([id, env = {}]) => [
    String(id).trim(),
    {
      name: String(env.name || id).trim(),
      commandPrefix: Array.isArray(env.commandPrefix)
        ? env.commandPrefix.map((part) => String(part)).filter(Boolean)
        : [],
      allowRun: normalizeAllowRun(env.allowRun),
      env: env.env && typeof env.env === 'object' && !Array.isArray(env.env)
        ? Object.fromEntries(Object.entries(env.env).map(([key, value]) => [key, String(value)]))
        : {}
    }
  ]).filter(([id]) => id))
}

const normalizeRuntimeEnvironment = (environment = {}) => normalizeProgramEnvironments({ runtime: environment }).runtime

export class ProgramRuntime {
  constructor({ executor = execFileAsync } = {}) {
    this.executor = executor
  }

  async run({ environment, command, cwd = '', baseEnv = process.env }) {
    const runtimeEnvironment = normalizeRuntimeEnvironment(environment)
    if (runtimeEnvironment?.allowRun !== true) {
      throw new Error('Program execution is disabled for this environment until explicitly allowed.')
    }
    if (!Array.isArray(runtimeEnvironment.commandPrefix) || runtimeEnvironment.commandPrefix.length === 0) {
      throw new Error('Program execution requires a configured command prefix allowlist.')
    }

    const normalizedCommand = String(command || '').trim()
    if (!normalizedCommand) throw new Error('Command is required.')
    const parts = [...runtimeEnvironment.commandPrefix, ...normalizedCommand.split(/\s+/).filter(Boolean)]
    const binary = parts.shift()
    if (!binary) throw new Error('Command is required.')
    const result = await this.executor(binary, parts, {
      cwd: cwd || process.cwd(),
      env: {
        ...baseEnv,
        ...(runtimeEnvironment.env || {})
      },
      timeout: 10 * 60 * 1000
    })
    return {
      stdout: result.stdout || '',
      stderr: result.stderr || ''
    }
  }
}
