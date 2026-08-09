import { describe, expect, it, vi } from 'vitest'
import fs from 'fs-extra'
import os from 'os'
import path from 'path'
import { RcloneVaultEngine } from 'main_renderer/elephantnote/sync/rcloneVaultEngine.js'
import { RcloneManager } from 'main_renderer/elephantnote/sync/RcloneManager.js'

const tempVault = async() => fs.mkdtemp(path.join(os.tmpdir(), 'elephant-rclone-engine-'))

const fakeRclone = (calls = [], executor = async() => ({ stdout: 'ok', stderr: '' })) =>
  new RcloneManager({
    executor: vi.fn(async(binary, args, options) => {
      calls.push({ binary, args, options })
      return executor(binary, args, options)
    })
  })

describe('RcloneVaultEngine', () => {
  it('queues and runs compatibility init/snapshot operations through rclone bisync', async() => {
    const cwd = await tempVault()
    const calls = []
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone(calls) })

    const operation = engine.enqueue({ operation: 'init', payload: { remotePath: 'remote:vault' } })
    engine.enqueue({ operation: 'snapshot' })
    const status = await engine.run()

    expect(operation.status).toBe('done')
    expect(status.queued).toBe(0)
    expect(status.remotePath).toBe('remote:vault')
    expect(calls).toHaveLength(1)
    expect(calls[0].args.slice(0, 3)).toEqual(['bisync', cwd, 'remote:vault'])
    expect(calls[0].args).toContain('--resync')
    expect(calls[0].args).toContain('--filters-file')
  })

  it('persists remote path and first run state after a successful sync', async() => {
    const cwd = await tempVault()
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone() })

    await engine.run({ init: { remotePath: 'remote:vault' }, snapshot: {} })

    const config = await fs.readJson(path.join(cwd, '.elephantnote', 'sync-config.json'))
    expect(config).toMatchObject({ backend: 'rclone', remotePath: 'remote:vault', firstRunDone: true })
    await expect(fs.pathExists(path.join(cwd, '.elephantnote', 'sync', 'rclone-filter.txt'))).resolves.toBe(true)
  })

  it('loads persisted config before a later sync run', async() => {
    const cwd = await tempVault()
    const calls = []
    await fs.ensureDir(path.join(cwd, '.elephantnote'))
    await fs.writeJson(path.join(cwd, '.elephantnote', 'sync-config.json'), {
      backend: 'rclone',
      remotePath: 'remote:vault',
      firstRunDone: true
    })
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone(calls) })

    engine.enqueue({ operation: 'snapshot' })
    await engine.run()

    expect(calls).toHaveLength(1)
    expect(calls[0].args.slice(0, 3)).toEqual(['bisync', cwd, 'remote:vault'])
    expect(calls[0].args).not.toContain('--resync')
  })

  it('keeps failed operations and diagnostics when rclone fails', async() => {
    const cwd = await tempVault()
    const engine = new RcloneVaultEngine({
      cwd,
      rclone: fakeRclone([], async() => { throw new Error('boom') })
    })

    await expect(engine.run({ init: { remotePath: 'remote:vault' }, snapshot: {} })).rejects.toThrow('boom')
    expect(engine.status().lastError).toContain('boom')
    expect(engine.status().history.at(-1)).toMatchObject({ operation: 'snapshot', status: 'error' })
  })

  it('rejects sync without an active vault path', async() => {
    const engine = new RcloneVaultEngine({ rclone: fakeRclone() })
    await expect(engine.run({ init: { remotePath: 'remote:vault' }, snapshot: {} })).rejects.toThrow('active vault')
  })

  it('does not throw when a remote path has not been configured yet', async() => {
    const cwd = await tempVault()
    const calls = []
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone(calls) })

    await expect(engine.run({ snapshot: {} })).resolves.toMatchObject({ configured: false })

    expect(calls).toHaveLength(0)
    expect(engine.status().lastError).toContain('remote is not configured')
  })

  it('does not enqueue duplicate default snapshots when the queue is already populated', async() => {
    const cwd = await tempVault()
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone() })
    engine.enqueue({ operation: 'init', payload: {} })
    engine.enqueue({ operation: 'snapshot', payload: {} })

    expect(engine.enqueuePlan({})).toEqual([])
    expect(engine.status().queued).toBe(2)
  })

  it('accepts a top-level remotePath payload for run', async() => {
    const cwd = await tempVault()
    const calls = []
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone(calls) })

    await engine.run({ remotePath: 'remote:vault', snapshot: {} })

    expect(calls).toHaveLength(1)
    expect(calls[0].args.slice(0, 3)).toEqual(['bisync', cwd, 'remote:vault'])
  })

  it('clears in-memory config when changing vaults', async() => {
    const first = await tempVault()
    const second = await tempVault()
    const engine = new RcloneVaultEngine({ cwd: first, rclone: fakeRclone() })
    await engine.run({ init: { remotePath: 'remote:first' } })

    engine.setCwd(second)

    expect(engine.status().cwd).toBe(second)
    expect(engine.status().remotePath).toBe('')
    expect(engine.status().firstRunDone).toBe(false)
  })

  it('maps pull and push compatibility operations to the same rclone bisync transport', async() => {
    const cwd = await tempVault()
    const calls = []
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone(calls) })

    await engine.run({ init: { remotePath: 'remote:vault' }, pull: {}, push: {} })

    expect(calls).toHaveLength(2)
    expect(calls[0].args[0]).toBe('bisync')
    expect(calls[1].args[0]).toBe('bisync')
  })

  it('rejects unknown operations instead of marking them done', async() => {
    const cwd = await tempVault()
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone() })
    engine.enqueue({ operation: 'unknown' })

    await expect(engine.run()).rejects.toThrow('Unknown sync operation')
    expect(engine.status().history.at(-1)).toMatchObject({ operation: 'unknown', status: 'error' })
  })

  it('returns a status snapshot with queue and history slices', async() => {
    const cwd = await tempVault()
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone() })
    engine.enqueue({ operation: 'init', payload: { remotePath: 'remote:vault' } })

    const status = engine.status()

    expect(status.queued).toBe(1)
    expect(status.operations).toHaveLength(1)
    expect(status.history).toEqual([])
    expect(status.rclone.configured).toBe(true)
    expect(status.capabilities.mobileSyncRequiresBackend).toBe(true)
  })

  it('uses an empty snapshot plan when run receives no payload but queue is prefilled', async() => {
    const cwd = await tempVault()
    const calls = []
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone(calls) })
    engine.enqueuePlan({ init: { remotePath: 'remote:vault' }, snapshot: {} })

    await engine.run()

    expect(calls).toHaveLength(1)
  })

  it('returns current status immediately while already running', async() => {
    const cwd = await tempVault()
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone() })
    engine.running = true

    await expect(engine.run({ snapshot: {} })).resolves.toMatchObject({ running: true, cwd })
  })

  it('supports direct sync with a remote path override', async() => {
    const cwd = await tempVault()
    const calls = []
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone(calls) })

    await engine.sync({ remotePath: 'remote:override' })

    expect(calls[0].args.slice(0, 3)).toEqual(['bisync', cwd, 'remote:override'])
  })

  it('does not use resync after the first successful run', async() => {
    const cwd = await tempVault()
    const calls = []
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone(calls) })

    await engine.run({ init: { remotePath: 'remote:vault' }, snapshot: {} })
    engine.enqueue({ operation: 'snapshot' })
    await engine.run()

    expect(calls).toHaveLength(2)
    expect(calls[0].args).toContain('--resync')
    expect(calls[1].args).not.toContain('--resync')
  })

  it('exposes the generated config path', async() => {
    const cwd = await tempVault()
    const engine = new RcloneVaultEngine({ cwd, rclone: fakeRclone() })

    expect(engine.configPath()).toBe(path.join(cwd, '.elephantnote', 'sync-config.json'))
  })
})
