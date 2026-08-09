import { getSyncPlatformCapabilities } from './syncPlatform.js'

export class RcloneManager {
  constructor({ binaryPath = '', executor = null, platform = globalThis.process?.platform || '' } = {}) {
    this.binaryPath = binaryPath
    this.executor = executor
    this.platform = platform
    this.lastError = ''
    this.lastVersion = ''
  }

  configure({ binaryPath = '' } = {}) {
    if (binaryPath !== undefined) this.binaryPath = String(binaryPath || '').trim()
  }

  async resolveBinary() {
    return this.binaryPath || 'rclone'
  }

  async run(args = [], options = {}) {
    if (!this.executor) {
      throw new Error('Rclone execution backend is not configured.')
    }
    const capabilities = getSyncPlatformCapabilities(this.platform)
    if (!capabilities.desktopRclone) {
      throw new Error(`Rclone binary execution is not available on ${capabilities.platform || 'this platform'}. Use a remote sync backend for mobile sync.`)
    }
    const binary = await this.resolveBinary()
    try {
      const result = await this.executor(binary, args, options)
      this.lastError = ''
      return {
        binary,
        args,
        stdout: result?.stdout || '',
        stderr: result?.stderr || ''
      }
    } catch (error) {
      this.lastError = error?.message || 'rclone command failed.'
      throw error
    }
  }

  async version() {
    const result = await this.run(['version'], { timeout: 30000 })
    this.lastVersion = result.stdout.split('\n')[0] || result.stdout.trim()
    return this.lastVersion
  }

  status() {
    return {
      configured: Boolean(this.executor),
      binaryPath: this.binaryPath,
      version: this.lastVersion,
      lastError: this.lastError,
      capabilities: getSyncPlatformCapabilities(this.platform)
    }
  }
}
