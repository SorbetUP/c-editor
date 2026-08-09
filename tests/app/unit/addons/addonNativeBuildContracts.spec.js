import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('physical native addon build contracts', () => {
  it('packages process and persistent service runners through the same generic builder', () => {
    const script = read('build/scripts/build-physical-addon.mjs')
    expect(script).toContain("new Set(['process', 'service'])")
    expect(script).toContain("runner === 'service' ? 'elephant-addon-service-v1' : 'elephant-addon-sidecar-v1'")
    expect(script).toContain('manifest native.runner must match addon.build.json runner')
    expect(script).toContain('runner,')
  })

  it('copies the complete confined trusted module graph into native archives', () => {
    const script = read('build/scripts/build-physical-addon.mjs')
    expect(script).toContain('const copyTrustedModule =')
    expect(script).toContain('staticImportSpecifiers(source)')
    expect(script).toContain('trusted module escapes the addon package')
    expect(script).toContain('trusted module imports an external dependency')
    expect(script).toContain('modules: copiedModules.size')
  })
})
