import test from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { tmpdir } from 'node:os'
import { deflateSync } from 'node:zlib'

import { pngNonBackgroundBounds } from '../png-geometry.mjs'

test('PNG geometry reports a measured visible pixel box', () => {
  const root = mkdtempSync(join(tmpdir(), 'tauri-visual-png-'))
  const filename = join(root, 'pixel.png')
  const crc32 = (buffer) => {
    let crc = 0xffffffff
    for (const byte of buffer) {
      crc ^= byte
      for (let bit = 0; bit < 8; bit += 1) crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0)
    }
    return (crc ^ 0xffffffff) >>> 0
  }
  const chunk = (name, value) => {
    const type = Buffer.from(name)
    const result = Buffer.alloc(value.length + 12)
    result.writeUInt32BE(value.length, 0)
    type.copy(result, 4)
    value.copy(result, 8)
    result.writeUInt32BE(crc32(Buffer.concat([type, value])), value.length + 8)
    return result
  }
  const header = Buffer.from([0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0])
  writeFileSync(filename, Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', header), chunk('IDAT', deflateSync(Buffer.from([0, 255, 0, 0, 255]))), chunk('IEND', Buffer.alloc(0))]))
  assert.deepEqual(pngNonBackgroundBounds(filename), { x: 0, y: 0, width: 1, height: 1 })
})
