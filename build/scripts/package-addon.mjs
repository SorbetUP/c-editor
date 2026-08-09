import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'

const [, , sourceArg, outputArg] = process.argv
if (!sourceArg || !outputArg) {
  console.error('Usage: node build/scripts/package-addon.mjs <addon-directory> <output.enaddon>')
  process.exit(2)
}

const sourceDir = path.resolve(sourceArg)
const outputPath = path.resolve(outputArg)
const outputRelative = path.relative(sourceDir, outputPath).split(path.sep).join('/')

const crcTable = Array.from({ length: 256 }, (_, index) => {
  let value = index
  for (let bit = 0; bit < 8; bit += 1) {
    value = (value & 1) ? (0xedb88320 ^ (value >>> 1)) : (value >>> 1)
  }
  return value >>> 0
})

const crc32 = (buffer) => {
  let crc = 0xffffffff
  for (const byte of buffer) crc = crcTable[(crc ^ byte) & 0xff] ^ (crc >>> 8)
  return (crc ^ 0xffffffff) >>> 0
}

const dosDateTime = (date = new Date()) => {
  const year = Math.max(1980, date.getFullYear())
  const dosTime = (date.getHours() << 11) | (date.getMinutes() << 5) | Math.floor(date.getSeconds() / 2)
  const dosDate = ((year - 1980) << 9) | ((date.getMonth() + 1) << 5) | date.getDate()
  return { dosTime, dosDate }
}

const listFiles = async (directory, prefix = '') => {
  const entries = await fs.readdir(directory, { withFileTypes: true })
  const files = []
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    const absolute = path.join(directory, entry.name)
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name
    if (relative === outputRelative) continue
    if (entry.isDirectory()) files.push(...await listFiles(absolute, relative))
    else if (entry.isFile()) files.push({ absolute, relative })
  }
  return files
}

const sourceStat = await fs.stat(sourceDir).catch(() => null)
if (!sourceStat?.isDirectory()) throw new Error(`Addon source directory does not exist: ${sourceDir}`)

const manifestPath = path.join(sourceDir, 'manifest.json')
const manifest = JSON.parse(await fs.readFile(manifestPath, 'utf8'))
if (!manifest?.id || !manifest?.runtime?.entry) throw new Error('manifest.json must define id and runtime.entry')
await fs.access(path.join(sourceDir, manifest.runtime.entry))

const files = await listFiles(sourceDir)
if (!files.some((file) => file.relative === 'manifest.json')) throw new Error('manifest.json must be at the package root')

const localParts = []
const centralParts = []
let localOffset = 0
const timestamp = dosDateTime()

for (const file of files) {
  const data = await fs.readFile(file.absolute)
  const name = Buffer.from(file.relative.split(path.sep).join('/'), 'utf8')
  const checksum = crc32(data)

  const localHeader = Buffer.alloc(30)
  localHeader.writeUInt32LE(0x04034b50, 0)
  localHeader.writeUInt16LE(20, 4)
  localHeader.writeUInt16LE(0, 6)
  localHeader.writeUInt16LE(0, 8)
  localHeader.writeUInt16LE(timestamp.dosTime, 10)
  localHeader.writeUInt16LE(timestamp.dosDate, 12)
  localHeader.writeUInt32LE(checksum, 14)
  localHeader.writeUInt32LE(data.length, 18)
  localHeader.writeUInt32LE(data.length, 22)
  localHeader.writeUInt16LE(name.length, 26)
  localHeader.writeUInt16LE(0, 28)
  localParts.push(localHeader, name, data)

  const centralHeader = Buffer.alloc(46)
  centralHeader.writeUInt32LE(0x02014b50, 0)
  centralHeader.writeUInt16LE(20, 4)
  centralHeader.writeUInt16LE(20, 6)
  centralHeader.writeUInt16LE(0, 8)
  centralHeader.writeUInt16LE(0, 10)
  centralHeader.writeUInt16LE(timestamp.dosTime, 12)
  centralHeader.writeUInt16LE(timestamp.dosDate, 14)
  centralHeader.writeUInt32LE(checksum, 16)
  centralHeader.writeUInt32LE(data.length, 20)
  centralHeader.writeUInt32LE(data.length, 24)
  centralHeader.writeUInt16LE(name.length, 28)
  centralHeader.writeUInt16LE(0, 30)
  centralHeader.writeUInt16LE(0, 32)
  centralHeader.writeUInt16LE(0, 34)
  centralHeader.writeUInt16LE(0, 36)
  centralHeader.writeUInt32LE(0, 38)
  centralHeader.writeUInt32LE(localOffset, 42)
  centralParts.push(centralHeader, name)

  localOffset += localHeader.length + name.length + data.length
}

const centralDirectory = Buffer.concat(centralParts)
const end = Buffer.alloc(22)
end.writeUInt32LE(0x06054b50, 0)
end.writeUInt16LE(0, 4)
end.writeUInt16LE(0, 6)
end.writeUInt16LE(files.length, 8)
end.writeUInt16LE(files.length, 10)
end.writeUInt32LE(centralDirectory.length, 12)
end.writeUInt32LE(localOffset, 16)
end.writeUInt16LE(0, 20)

await fs.mkdir(path.dirname(outputPath), { recursive: true })
const archive = Buffer.concat([...localParts, centralDirectory, end])
await fs.writeFile(outputPath, archive)

console.log(`[addons:package] id=${manifest.id} files=${files.length} bytes=${archive.length} output=${outputPath}`)
