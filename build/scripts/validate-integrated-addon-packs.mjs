import fs from 'node:fs/promises'
import path from 'node:path'

const root = process.cwd()
const catalogPath = path.join(root, 'addons', 'catalog.json')
const packsRoot = path.join(root, 'addons', 'packs')
const catalog = JSON.parse(await fs.readFile(catalogPath, 'utf8'))
const entries = new Map(catalog.addons.map((entry) => [entry.id, entry]))
const forbidden = new Set(['elephant.addon-packs', 'elephant.excalidraw'])

const packFiles = (await fs.readdir(packsRoot))
  .filter((name) => name.endsWith('.enaddonpack'))
  .sort()

if (!packFiles.length) throw new Error('No integrated addon packs were found')

for (const fileName of packFiles) {
  const pack = JSON.parse(await fs.readFile(path.join(packsRoot, fileName), 'utf8'))
  if (pack.format !== 'elephantnote-addon-pack' || pack.version !== 1) {
    throw new Error(`${fileName}: unsupported pack format`)
  }
  if (!Array.isArray(pack.addons) || !pack.addons.length) {
    throw new Error(`${fileName}: pack must contain addons`)
  }

  const positions = new Map()
  for (const [index, item] of pack.addons.entries()) {
    if (forbidden.has(item.id)) throw new Error(`${fileName}: core feature serialized as addon: ${item.id}`)
    if (positions.has(item.id)) throw new Error(`${fileName}: duplicate addon ${item.id}`)
    positions.set(item.id, index)

    const catalogEntry = entries.get(item.id)
    if (!catalogEntry) throw new Error(`${fileName}: addon absent from integrated catalogue: ${item.id}`)
    if (item.source !== 'official') throw new Error(`${fileName}: first-party addon must use source=official: ${item.id}`)
    if (item.version !== catalogEntry.version) {
      throw new Error(`${fileName}: version mismatch for ${item.id}: pack=${item.version} catalog=${catalogEntry.version}`)
    }

    const manifestPath = path.join(root, 'addons', catalogEntry.manifestPath)
    const manifest = JSON.parse(await fs.readFile(manifestPath, 'utf8'))
    if (manifest.id !== item.id || manifest.version !== item.version) {
      throw new Error(`${fileName}: manifest mismatch for ${item.id}`)
    }
  }

  for (const item of pack.addons) {
    const entry = entries.get(item.id)
    const manifest = JSON.parse(await fs.readFile(path.join(root, 'addons', entry.manifestPath), 'utf8'))
    for (const dependencyId of Object.keys(manifest.requires || {})) {
      if (!positions.has(dependencyId)) throw new Error(`${fileName}: ${item.id} requires missing ${dependencyId}`)
      if (positions.get(dependencyId) > positions.get(item.id)) {
        throw new Error(`${fileName}: ${dependencyId} must be ordered before ${item.id}`)
      }
    }
  }

  console.log(`[addon-pack] valid file=${fileName} addons=${pack.addons.length}`)
}
