import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const readTauriConfig = () => JSON.parse(
  readFileSync(resolve(process.cwd(), 'Elephant/backend/tauri/tauri.conf.json'), 'utf8')
)

describe('tauri bootstrap config', () => {
  it('exposes the Tauri global API required by the renderer bootstrap', () => {
    const config = readTauriConfig()

    expect(config.app?.withGlobalTauri).toBe(true)
  })
})
