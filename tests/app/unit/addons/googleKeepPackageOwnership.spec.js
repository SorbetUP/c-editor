import fs from 'node:fs'
import path from 'node:path'
import { Blob } from 'node:buffer'
import { deflateRawSync, inflateRawSync } from 'node:zlib'
import { describe, expect, it } from 'vitest'
import ElephantGoogleKeepImportAddon, {
  keepDocumentToMarkdown,
  parseKeepDocument,
  safeNoteStem
} from '../../../../addons/official/google-keep-import/main.js'
import { rssToSources, webPageToSource } from '../../../../addons/official/google-keep-import/sources.js'
import { extractZipJsonDocuments } from '../../../../addons/official/google-keep-import/zip.js'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

globalThis.DecompressionStream = class TestDecompressionStream {
    constructor(format) {
      if (format !== 'deflate-raw') throw new Error(`Unsupported test format: ${format}`)
      const chunks = []
      const transform = new TransformStream({
        transform(chunk) {
          chunks.push(Buffer.from(chunk))
        },
        flush(controller) {
          controller.enqueue(new Uint8Array(inflateRawSync(Buffer.concat(chunks))))
        }
      })
      this.readable = transform.readable
      this.writable = transform.writable
    }
}
globalThis.Blob = Blob

const namedBlob = (parts, name, type) => {
  const blob = new Blob(parts, { type })
  Object.defineProperty(blob, 'name', { value: name })
  Object.defineProperty(blob, 'webkitRelativePath', { value: name })
  return blob
}

const zipArchive = (entries, compression = 8) => {
  const localParts = []
  const centralParts = []
  let localOffset = 0

  for (const [name, value] of entries) {
    const nameBytes = Buffer.from(name, 'utf8')
    const plain = Buffer.from(value, 'utf8')
    const compressed = compression === 8 ? deflateRawSync(plain) : plain

    const local = Buffer.alloc(30)
    local.writeUInt32LE(0x04034b50, 0)
    local.writeUInt16LE(20, 4)
    local.writeUInt16LE(0x0800, 6)
    local.writeUInt16LE(compression, 8)
    local.writeUInt32LE(0, 14)
    local.writeUInt32LE(compressed.length, 18)
    local.writeUInt32LE(plain.length, 22)
    local.writeUInt16LE(nameBytes.length, 26)
    localParts.push(local, nameBytes, compressed)

    const central = Buffer.alloc(46)
    central.writeUInt32LE(0x02014b50, 0)
    central.writeUInt16LE(20, 4)
    central.writeUInt16LE(20, 6)
    central.writeUInt16LE(0x0800, 8)
    central.writeUInt16LE(compression, 10)
    central.writeUInt32LE(0, 16)
    central.writeUInt32LE(compressed.length, 20)
    central.writeUInt32LE(plain.length, 24)
    central.writeUInt16LE(nameBytes.length, 28)
    central.writeUInt32LE(localOffset, 42)
    centralParts.push(central, nameBytes)

    localOffset += local.length + nameBytes.length + compressed.length
  }

  const centralDirectory = Buffer.concat(centralParts)
  const eocd = Buffer.alloc(22)
  eocd.writeUInt32LE(0x06054b50, 0)
  eocd.writeUInt16LE(entries.length, 8)
  eocd.writeUInt16LE(entries.length, 10)
  eocd.writeUInt32LE(centralDirectory.length, 12)
  eocd.writeUInt32LE(localOffset, 16)
  return Buffer.concat([...localParts, centralDirectory, eocd])
}

describe('Import physical package ownership', () => {
  it('declares scoped writes and explicit public-web access for the restored 1.4 importer', () => {
    const manifest = JSON.parse(read('addons/official/google-keep-import/manifest.json'))

    expect(manifest.name).toBe('Google Keep Import')
    expect(manifest.version).toBe('1.2.0')
    expect(manifest.description).toMatch(/Google Keep Takeout JSON/i)
    expect(manifest.permissions.network).toBeUndefined()
    expect(manifest.permissions.notes.read).toEqual([])
    expect(manifest.permissions.notes.write).toEqual(['Imported/Google Keep/**'])
    expect(manifest.runtime.mode).toBe('trusted')
  })

  it('converts Keep text, checklist and metadata into Markdown', () => {
    const note = parseKeepDocument({
      title: 'Release checklist',
      textContent: 'Prepare the release.',
      listContent: [
        { text: 'Run tests', isChecked: true },
        { text: 'Publish build', isChecked: false }
      ],
      labels: [{ name: 'Work' }, { name: 'Elephant' }],
      attachments: [{ filePath: 'Takeout/Keep/image.png', mimetype: 'image/png' }],
      annotations: [{ webLink: { title: 'Release notes', url: 'https://example.com/release' } }],
      createdTimestampUsec: '1721000000000000',
      userEditedTimestampUsec: '1721003600000000',
      isPinned: true,
      isArchived: false,
      color: 'YELLOW'
    }, 'Release checklist.json')
    const markdown = keepDocumentToMarkdown(note)

    expect(note.title).toBe('Release checklist')
    expect(note.labels).toEqual(['Work', 'Elephant'])
    expect(markdown).toContain('source: google-keep')
    expect(markdown).toContain('tags: ["Work", "Elephant"]')
    expect(markdown).toContain('created:')
    expect(markdown).toContain('pinned: true')
    expect(markdown).toContain('- [x] Run tests')
    expect(markdown).toContain('- [ ] Publish build')
    expect(markdown).toContain('- Takeout/Keep/image.png')
    expect(markdown).toContain('# Release checklist')
  })

  it('sanitizes note filenames for every desktop platform', () => {
    expect(safeNoteStem('  A/B:C*D?  ')).toBe('A-B-C-D-')
    expect(safeNoteStem('')).toBe('Untitled Keep note')
  })

  it.each([0, 8])('extracts Google Takeout JSON from ZIP compression method %s', async(compression) => {
    const archive = zipArchive([
      ['Takeout/Keep/Release checklist.json', JSON.stringify({
        title: 'ZIP note',
        textContent: 'Imported directly from Takeout'
      })],
      ['Takeout/Keep/readme.txt', 'ignored']
    ], compression)

    const documents = await extractZipJsonDocuments(namedBlob([archive], 'keep.zip', 'application/zip'))

    expect(documents).toHaveLength(1)
    expect(documents[0].name).toBe('Takeout/Keep/Release checklist.json')
    expect(JSON.parse(documents[0].content).title).toBe('ZIP note')
  })

  it('imports individual JSON files through the addon-owned file boundary', async() => {
    const addon = new ElephantGoogleKeepImportAddon({ experimental: { window } })
    addon.writeNote = async() => ({ created: true })
    const result = await addon.importFiles([
      namedBlob([JSON.stringify({ title: 'Loose note', textContent: 'Imported' })], 'loose.json', 'application/json')
    ])
    expect(result).toMatchObject({ imported: 1, skipped: 0, failed: 0 })
    expect(result.results[0].path).toBe('Imported/Google Keep/Loose note.md')
  })

  it('retries a unique filename when the vault already contains the target note', async() => {
    const addon = new ElephantGoogleKeepImportAddon({ experimental: { window } })
    const writes = []
    addon.writeNote = async(notePath) => {
      writes.push(notePath)
      return { created: true, path: notePath }
    }

    const result = await addon.importDocuments([
      { name: 'existing.json', content: JSON.stringify({ title: 'Existing note', textContent: 'Imported' }) }
    ])

    expect(writes).toEqual(['Imported/Google Keep/Existing note.md'])
    expect(result).toMatchObject({ imported: 1, skipped: 0, failed: 0 })
    expect(result.results[0].path).toBe('Imported/Google Keep/Existing note.md')
  })

  it('converts a web page and RSS feed into useful Markdown sources', () => {
    const page = webPageToSource(window, `
      <html><head><title>Elephant article</title></head><body>
        <article><h1>Elephant article</h1><p>Hello <strong>world</strong>.</p><ul><li>First</li><li>Second</li></ul></article>
      </body></html>
    `, 'https://example.com/article')
    expect(page.title).toBe('Elephant article')
    expect(page.markdown).toContain('# Elephant article')
    expect(page.markdown).toContain('Hello **world**.')
    expect(page.markdown).toContain('- First')

    const feed = rssToSources(window, `
      <rss version="2.0"><channel><title>Elephant feed</title><item>
        <title>Release</title><link>https://example.com/release</link>
        <description><![CDATA[<p>Version <strong>1.4</strong> shipped.</p>]]></description>
      </item></channel></rss>
    `, 'https://example.com/feed.xml')
    expect(feed).toHaveLength(1)
    expect(feed[0]).toMatchObject({ title: 'Release', url: 'https://example.com/release', feedTitle: 'Elephant feed' })
    expect(feed[0].markdown).toContain('Version **1.4** shipped.')
  })

  it('restores the exact Import settings structure without moving implementations back into core', () => {
    const source = read('addons/official/google-keep-import/main.js')
    const sources = read('addons/official/google-keep-import/sources.js')
    const zipSource = read('addons/official/google-keep-import/zip.js')
    const core = read('Elephant/backend/tauri/src/lib_min.rs')
    const noteAccess = read('Elephant/backend/tauri/src/addon_note_access.rs')
    const runtimeAccess = read('Elephant/backend/tauri/src/addon_runtime_access.rs')

    expect(source).toContain("const PROVIDER_RESOURCE = 'import.google-keep'")
    expect(source).toContain("api.settings.registerSection({")
    expect(source).toContain("section: 'import'")
    expect(source).toContain('Import selected files')
    expect(source).toContain('Include trashed notes')
    expect(JSON.parse(read('addons/official/google-keep-import/manifest.json')).apiVersion).toBe(1)
    expect(source).toContain('JSON.stringify(result, null, 2)')
    expect(sources).toContain("node(documentRef, 'div', 'en-form-grid')")
    expect(sources).toContain("node(documentRef, 'span', '', 'Source URL')")
    expect(sources).toContain("node(documentRef, 'span', '', 'Destination folder')")
    expect(sources).toContain("node(documentRef, 'button', '', 'Import page')")
    expect(sources).toContain("node(documentRef, 'button', '', 'Import RSS')")
    expect(sources).toContain("invoke('tauri_addons_http_request'")
    expect(zipSource).toContain("new DecompressionStream('deflate-raw')")
    expect(source).not.toContain('elephantnote.api')
    expect(source).not.toContain('import.googleKeep')
    expect(core).toContain('addon_note_access::tauri_addons_notes_write')
    expect(noteAccess).toContain('record.manifest.permissions.notes.write')
    expect(noteAccess).toContain('prepare_write_target')
    expect(noteAccess).toContain('write_markdown_atomic')
    expect(runtimeAccess).toContain('const PUBLIC_HTTPS_CAPABILITY: &str = "public-https"')
  })
})
