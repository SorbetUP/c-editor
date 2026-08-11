import test from 'node:test'
import assert from 'node:assert/strict'

import { selectNativeWindow, selectTauriChildProcess } from '../native-window.mjs'

const candidate = (overrides = {}) => ({
  windowId: 12,
  ownerPid: 42,
  ownerName: 'Elephant',
  title: 'Elephant',
  layer: 0,
  onScreen: true,
  bounds: { x: 40, y: 60, width: 1280, height: 840 },
  ...overrides
})

test('selects the visible main window for the launched Tauri pid', () => {
  const result = selectNativeWindow({
    pid: 42,
    names: ['Elephant'],
    windows: [candidate(), candidate({ windowId: 99, ownerPid: 9, ownerName: 'Other' })]
  })
  assert.equal(result.windowId, 12)
  assert.equal(result.ownerPid, 42)
})

test('rejects ambiguous visible main windows instead of guessing', () => {
  assert.throws(() => selectNativeWindow({
    pid: 42,
    names: ['Elephant'],
    windows: [candidate(), candidate({ windowId: 13 })]
}), /ambiguous/i)
})

test('selects the actual Tauri executable below the launcher, never build_dev.sh', () => {
  const result = selectTauriChildProcess({
    launcherPid: 10,
    appPath: '/repo/build/scripts/build_dev.sh',
    table: [
      { pid: 10, ppid: 1, comm: '/bin/bash', args: '/repo/build/scripts/build_dev.sh' },
      { pid: 11, ppid: 10, comm: '/usr/bin/cargo', args: 'cargo tauri dev --no-watch' },
      { pid: 12, ppid: 11, comm: '/repo/target/debug/Elephant', args: '/repo/target/debug/Elephant' }
    ]
  })
  assert.equal(result.pid, 12)
  assert.equal(result.descendant, true)
  assert.match(result.comm, /Elephant$/)
})

test('accepts macOS ps comm truncation only when argv proves the Tauri executable', () => {
  const result = selectTauriChildProcess({
    launcherPid: 10,
    table: [
      { pid: 10, ppid: 1, comm: '/bin/bash', args: '/repo/build/scripts/build_dev.sh' },
      { pid: 12, ppid: 10, comm: 'target/debug/Ele', args: 'target/debug/Elephant' }
    ]
  })
  assert.equal(result.pid, 12)
})

test('rejects a launcher-only match as insufficient Tauri identity proof', () => {
  assert.throws(() => selectTauriChildProcess({
    launcherPid: 10,
    appPath: '/repo/build/scripts/build_dev.sh',
    table: [{ pid: 10, ppid: 1, comm: '/bin/bash', args: '/repo/build/scripts/build_dev.sh' }]
  }), /actual Tauri child/i)
})
