import { deflateSync } from 'node:zlib'
import { writeFile } from 'node:fs/promises'

const GLYPHS = {
  A: ['01110', '10001', '10001', '11111', '10001', '10001', '10001'],
  C: ['01111', '10000', '10000', '10000', '10000', '10000', '01111'],
  E: ['11111', '10000', '10000', '11110', '10000', '10000', '11111'],
  H: ['10001', '10001', '10001', '11111', '10001', '10001', '10001'],
  L: ['10000', '10000', '10000', '10000', '10000', '10000', '11111'],
  N: ['10001', '11001', '10101', '10011', '10001', '10001', '10001'],
  O: ['01110', '10001', '10001', '10001', '10001', '10001', '01110'],
  P: ['11110', '10001', '10001', '11110', '10000', '10000', '10000'],
  R: ['11110', '10001', '10001', '11110', '10100', '10010', '10001'],
  T: ['11111', '00100', '00100', '00100', '00100', '00100', '00100']
}

const crc32 = (buffer) => {
  let crc = 0xffffffff
  for (const byte of buffer) {
    crc ^= byte
    for (let bit = 0; bit < 8; bit += 1) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0)
  }
  const output = Buffer.alloc(4)
  output.writeUInt32BE((crc ^ 0xffffffff) >>> 0)
  return output
}

const chunk = (type, data) => {
  const name = Buffer.from(type)
  const size = Buffer.alloc(4)
  size.writeUInt32BE(data.length)
  return Buffer.concat([size, name, data, crc32(Buffer.concat([name, data]))])
}

const makePng = (text) => {
  const scale = 20
  const lines = String(text).split('\n')
  const glyphWidth = 5 * scale
  const gap = 2 * scale
  const width = Math.max(...lines.map((line) => line.length * (glyphWidth + gap))) + 2 * scale
  const lineHeight = 7 * scale + 4 * scale
  const height = lines.length * lineHeight + 2 * scale
  const pixels = Buffer.alloc(width * height * 3, 255)
  const paint = (x, y) => {
    const offset = (y * width + x) * 3
    pixels[offset] = 0
    pixels[offset + 1] = 0
    pixels[offset + 2] = 0
  }
  lines.forEach((line, lineIndex) => {
    let cursor = scale
    for (const character of line) {
      const glyph = GLYPHS[character]
      if (!glyph) {
        cursor += glyphWidth + gap
        continue
      }
      glyph.forEach((row, rowIndex) => row.split('').forEach((value, columnIndex) => {
        if (value !== '1') return
        for (let dy = 0; dy < scale; dy += 1) {
          for (let dx = 0; dx < scale; dx += 1) paint(cursor + columnIndex * scale + dx, 2 * scale + lineIndex * lineHeight + rowIndex * scale + dy)
        }
      }))
      cursor += glyphWidth + gap
    }
  })
  const scanlines = Buffer.alloc((width * 3 + 1) * height)
  for (let row = 0; row < height; row += 1) {
    scanlines[row * (width * 3 + 1)] = 0
    pixels.copy(scanlines, row * (width * 3 + 1) + 1, row * width * 3, (row + 1) * width * 3)
  }
  const header = Buffer.alloc(13)
  header.writeUInt32BE(width, 0)
  header.writeUInt32BE(height, 4)
  header[8] = 8
  header[9] = 2
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(scanlines, { level: 9 })),
    chunk('IEND', Buffer.alloc(0))
  ])
}

export const writeOcrFixture = async (filePath) => {
  await writeFile(filePath, makePng('ELEPHANT\nELEPHANT'))
  return filePath
}
