import { inflateSync } from 'node:zlib'
import { readFileSync } from 'node:fs'

const paeth = (a, b, c) => {
  const p = a + b - c
  const pa = Math.abs(p - a)
  const pb = Math.abs(p - b)
  const pc = Math.abs(p - c)
  return pa <= pb && pa <= pc ? a : pb <= pc ? b : c
}

export const pngNonBackgroundBounds = (filename) => {
  const bytes = readFileSync(filename)
  const idat = []
  let width = 0
  let height = 0
  let bitDepth = 0
  let colorType = 0
  for (let offset = 8; offset < bytes.length;) {
    const length = bytes.readUInt32BE(offset)
    const type = bytes.toString('ascii', offset + 4, offset + 8)
    const data = bytes.subarray(offset + 8, offset + 8 + length)
    if (type === 'IHDR') {
      width = data.readUInt32BE(0)
      height = data.readUInt32BE(4)
      bitDepth = data[8]
      colorType = data[9]
      if (bitDepth !== 8 || ![0, 2, 4, 6].includes(colorType)) throw new Error(`PNG geometry requires 8-bit grayscale/RGB/RGBA, got depth=${bitDepth} colorType=${colorType}`)
    }
    if (type === 'IDAT') idat.push(data)
    offset += 12 + length
    if (type === 'IEND') break
  }
  const encoded = inflateSync(Buffer.concat(idat))
  const bytesPerPixel = { 0: 1, 2: 3, 4: 2, 6: 4 }[colorType]
  const stride = width * bytesPerPixel
  const rows = []
  let inputOffset = 0
  let previous = Buffer.alloc(stride)
  for (let y = 0; y < height; y += 1) {
    const filter = encoded[inputOffset++]
    const row = Buffer.from(encoded.subarray(inputOffset, inputOffset + stride))
    inputOffset += stride
    for (let index = 0; index < stride; index += 1) {
      const left = index >= bytesPerPixel ? row[index - bytesPerPixel] : 0
      const up = previous[index]
      const upperLeft = index >= bytesPerPixel ? previous[index - bytesPerPixel] : 0
      if (filter === 1) row[index] = (row[index] + left) & 255
      else if (filter === 2) row[index] = (row[index] + up) & 255
      else if (filter === 3) row[index] = (row[index] + Math.floor((left + up) / 2)) & 255
      else if (filter === 4) row[index] = (row[index] + paeth(left, up, upperLeft)) & 255
      else if (filter !== 0) throw new Error(`unsupported PNG filter ${filter}`)
    }
    rows.push(row)
    previous = row
  }
  let left = width
  let top = height
  let right = -1
  let bottom = -1
  for (let y = 0; y < height; y += 1) {
    const row = rows[y]
    for (let x = 0; x < width; x += 1) {
      const index = x * bytesPerPixel
      const red = row[index]
      const green = colorType === 0 || colorType === 4 ? red : row[index + 1]
      const blue = colorType === 0 || colorType === 4 ? red : row[index + 2]
      const alpha = colorType === 4 ? row[index + 1] : colorType === 6 ? row[index + 3] : 255
      const visible = alpha > 0 && red + green + blue > 6
      if (!visible) continue
      left = Math.min(left, x)
      top = Math.min(top, y)
      right = Math.max(right, x)
      bottom = Math.max(bottom, y)
    }
  }
  if (right < left || bottom < top) throw new Error(`PNG has no non-background pixels: ${filename}`)
  return { x: left, y: top, width: right - left + 1, height: bottom - top + 1 }
}
