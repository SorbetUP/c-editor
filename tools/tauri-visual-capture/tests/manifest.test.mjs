import test from 'node:test'
import assert from 'node:assert/strict'
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { tmpdir } from 'node:os'

import { sha256File, validateCaptureManifest, validateSharedCaptureManifest } from '../manifest.mjs'

const onePixelPng = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=',
  'base64'
)

const validManifest = (root) => {
  mkdirSync(join(root, 'frames'), { recursive: true })
  writeFileSync(join(root, 'frames', 'idle.png'), onePixelPng)
  return {
    schema: 'elephant.tauri-visual-capture/v1',
    runtime: 'tauri-native-window',
    control: 'acceptance-http-bridge',
    window: { id: 7, ownerPid: 42, width: 1, height: 1 },
    frames: [{
      id: 'idle',
      kind: 'static',
      path: 'frames/idle.png',
      bytes: onePixelPng.length,
      width: 1,
      height: 1,
      sha256: sha256File(join(root, 'frames', 'idle.png'))
    }],
    actions: [{ id: 'idle', source: { file: 'fixture', symbol: 'idle' } }]
  }
}

test('manifest validation rejects missing frame files and bad hashes', () => {
  const root = mkdtempSync(join(tmpdir(), 'tauri-visual-manifest-red-'))
  const manifest = validManifest(root)
  manifest.frames[0].sha256 = 'bad-hash'
  const result = validateCaptureManifest(manifest, { outputRoot: root })
  assert.equal(result.ok, false)
  assert.match(result.errors.join('\n'), /sha256|hash/i)

  manifest.frames[0].path = 'frames/missing.png'
  const missing = validateCaptureManifest(manifest, { outputRoot: root })
  assert.equal(missing.ok, false)
  assert.match(missing.errors.join('\n'), /missing/i)
})

test('manifest validation enforces temporal cardinality and distinct-frame evidence', () => {
  const root = mkdtempSync(join(tmpdir(), 'tauri-visual-manifest-temporal-'))
  const manifest = validManifest(root)
  manifest.capturePlan = { temporalMinFrames: 2, requireDistinctTemporalFrames: true }
  manifest.frames[0].kind = 'temporal'
  const result = validateCaptureManifest(manifest, { outputRoot: root })
  assert.equal(result.ok, false)
  assert.match(result.errors.join('\n'), /temporal frame count|distinct/i)
})

test('shared manifest validation rejects a seven-step substitute and bridge control', () => {
  const scenario = { id: 'shared', actions: Array.from({ length: 14 }, (_, index) => ({ index, id: `a${index}`, logical: `logical-${index}`, checkpoint: `c${index}`, frames: [0, 50, 100] })), checkpoints: Array.from({ length: 14 }, (_, index) => ({ id: `c${index}` })) }
  const result = validateSharedCaptureManifest({
    schemaVersion: 1,
    runtime: 'tauri',
    scenarioId: 'shared',
    visualSurface: 'application-content-only',
    physicalInput: { controlPlane: 'acceptance-http-bridge', bridgeFallback: true },
    actions: [{ index: 0, id: 'a0', logical: 'logical-0', status: 'passed' }],
    checkpoints: [],
    fixture: { id: 'fixture' }
  }, { scenario, fixture: { id: 'fixture' } })
  assert.equal(result.ok, false)
  assert.match(result.errors.join('\n'), /action count|CGEvent|bridge|checkpoint count/i)
})
