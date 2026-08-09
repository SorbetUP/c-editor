import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'

const root = process.cwd()
const metaPath = path.join(root, 'agent/tauri-parity/elephant_tauri', 'parity', 'muya_source_snapshots_meta.json')
const snapshotsPath = path.join(root, 'agent/tauri-parity/elephant_tauri', 'parity', 'muya_source_snapshots.json')

const fail = (message) => {
  console.error(message)
  process.exit(1)
}

if (!fs.existsSync(metaPath)) fail(`Missing ${path.relative(root, metaPath)}`)
if (!fs.existsSync(snapshotsPath)) fail(`Missing ${path.relative(root, snapshotsPath)}`)

const meta = JSON.parse(fs.readFileSync(metaPath, 'utf8'))
const snapshots = JSON.parse(fs.readFileSync(snapshotsPath, 'utf8'))

if (meta.mode !== 'real-electron-muya-renderer') {
  fail(`Muya snapshots are not real renderer snapshots. mode=${meta.mode || 'unknown'}`)
}

if (!Array.isArray(snapshots) || snapshots.length === 0) {
  fail('Muya snapshot file must be a non-empty array')
}

const nonReal = snapshots.filter((snapshot) => snapshot.source !== 'real-electron-muya-renderer')
if (nonReal.length) {
  fail(`Found ${nonReal.length} non-real Muya snapshots`)
}

console.log(`Real Muya snapshots verified: ${snapshots.length} snapshots`)
