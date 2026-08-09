#!/usr/bin/env node

import { existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { spawnSync } from 'node:child_process'

const root = resolve(import.meta.dirname, '../..')
const executableName = process.platform === 'win32' ? 'Elephant.exe' : 'Elephant'
const releaseRoot = resolve(root, 'Elephant/backend/tauri/target/release')
const bundledMacExecutable = resolve(releaseRoot, 'bundle/macos/Elephant.app/Contents/MacOS/Elephant')
const executable = process.platform === 'darwin' && existsSync(bundledMacExecutable)
  ? bundledMacExecutable
  : resolve(releaseRoot, executableName)

if (!existsSync(executable)) {
  console.error(`[packaged-acceptance] missing ${executable}`)
  console.error('[packaged-acceptance] run pnpm build:mac, pnpm build:linux or pnpm build:win first')
  process.exit(1)
}

console.log(`[packaged-acceptance] testing ${executable}`)
const result = spawnSync('pnpm', ['test:desktop:acceptance'], {
  cwd: root,
  stdio: 'inherit',
  env: {
    ...process.env,
    ELEPHANT_ACCEPTANCE_SKIP_BUILD: '1',
    ELEPHANT_ACCEPTANCE_APP_PATH: executable
  },
  shell: process.platform === 'win32'
})

if (result.error) {
  console.error(`[packaged-acceptance] failed to launch runner: ${result.error.message}`)
  process.exit(1)
}
process.exit(result.status ?? 1)
