import { describe, expect, it, vi } from 'vitest'
import { RcloneManager } from 'main_renderer/elephantnote/sync/RcloneManager.js'
import { createRcloneExecutor } from 'main_renderer/elephantnote/sync/rcloneNodeRunner.js'

describe('RcloneManager', () => {
  it('uses the system rclone binary by default', async() => {
    const manager = new RcloneManager({ executor: vi.fn(async() => ({ stdout: '', stderr: '' })) })
    expect(await manager.resolveBinary()).toBe('rclone')
  })

  it('uses an explicit binary path when provided', async() => {
    const manager = new RcloneManager({
      binaryPath: '/opt/rclone',
      executor: vi.fn(async() => ({ stdout: '', stderr: '' }))
    })
    expect(await manager.resolveBinary()).toBe('/opt/rclone')
  })

  it('can update the configured binary path', async() => {
    const manager = new RcloneManager({ executor: vi.fn(async() => ({ stdout: '', stderr: '' })) })
    manager.configure({ binaryPath: '/tmp/rclone' })
    expect(await manager.resolveBinary()).toBe('/tmp/rclone')
  })

  it('passes command arguments and options to the injected executor', async() => {
    const executor = vi.fn(async() => ({ stdout: 'ok', stderr: '' }))
    const manager = new RcloneManager({ binaryPath: '/bin/rclone', executor })

    const result = await manager.run(['version'], { cwd: '/vault', timeout: 1000 })

    expect(result).toMatchObject({ binary: '/bin/rclone', args: ['version'], stdout: 'ok' })
    expect(executor).toHaveBeenCalledWith('/bin/rclone', ['version'], { cwd: '/vault', timeout: 1000 })
  })

  it('resets lastError after a successful command', async() => {
    const manager = new RcloneManager({ executor: vi.fn(async() => ({ stdout: 'ok', stderr: '' })) })
    manager.lastError = 'old error'

    await manager.run(['version'])

    expect(manager.status().lastError).toBe('')
  })

  it('stores diagnostics when execution fails', async() => {
    const manager = new RcloneManager({
      executor: vi.fn(async() => { throw new Error('rclone missing') })
    })

    await expect(manager.run(['version'])).rejects.toThrow('rclone missing')
    expect(manager.status().lastError).toBe('rclone missing')
  })

  it('returns the first line of rclone version output', async() => {
    const manager = new RcloneManager({
      executor: vi.fn(async() => ({ stdout: 'rclone v1.66.0\nother line', stderr: '' }))
    })

    await expect(manager.version()).resolves.toBe('rclone v1.66.0')
    expect(manager.status().version).toBe('rclone v1.66.0')
  })

  it('throws a clear error if no executor is configured', async() => {
    const manager = new RcloneManager()
    await expect(manager.run(['version'])).rejects.toThrow('execution backend')
  })

  it('creates a node executor that calls execFile-compatible binaries', async() => {
    expect(typeof createRcloneExecutor()).toBe('function')
  })
})
