/* @vitest-environment node */

import { describe, expect, it, vi } from 'vitest'
import {
  ProgramRuntime,
  normalizeProgramEnvironments
} from 'main_renderer/elephantnote/programRuntime'

describe('ProgramRuntime', () => {
  it('normalizes and runs commands inside configured environments', async() => {
    const environments = normalizeProgramEnvironments({
      python: {
        commandPrefix: ['uv', 'run'],
        allowRun: true,
        env: { TOKEN: 42 }
      }
    })
    const executor = vi.fn(async() => ({ stdout: 'ok', stderr: '' }))
    const runtime = new ProgramRuntime({ executor })

    await expect(runtime.run({
      environment: environments.python,
      command: 'pytest tests',
      cwd: '/vault'
    })).resolves.toMatchObject({ stdout: 'ok' })

    expect(executor).toHaveBeenCalledWith('uv', ['run', 'pytest', 'tests'], expect.objectContaining({
      cwd: '/vault',
      env: expect.objectContaining({ TOKEN: '42' })
    }))
  })
})
