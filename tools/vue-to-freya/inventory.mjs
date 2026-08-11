import { mkdir, readdir, readFile, writeFile } from 'node:fs/promises'
import { join, relative } from 'node:path'
import { analyzeVueFile } from './ast.mjs'

async function walk(directory) {
  const entries = await readdir(directory, { withFileTypes: true })
  const files = []
  for (const entry of entries) {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) files.push(...await walk(path))
    else if (entry.isFile() && path.endsWith('.vue')) files.push(path)
  }
  return files
}

const root = 'Elephant/frontend/app'
const output = 'migration/freya'
await mkdir(output, { recursive: true })
const files = await walk(root)
const reports = []
for (const path of files) {
  const source = await readFile(path, 'utf8')
  const report = analyzeVueFile(relative('.', path), source)
  const hasWeb = report.unsupported.some(({ kind }) => kind.startsWith('web-api:'))
  const hasIsland = /excalidraw|webview|iframe/i.test(path) || report.unsupported.some(({ kind }) => /canvas|document|window/.test(kind))
  const status = hasIsland ? 'WEB_ISLAND' : hasWeb ? 'MANUAL_PORT_REQUIRED' : report.unsupported.length ? 'MOSTLY_AUTO' : 'AUTO_CONVERTIBLE'
  reports.push({
    source: report.source,
    target: report.target,
    status,
    templateNodes: report.template.length,
    imports: report.script.imports,
    state: report.script.state,
    events: report.template.flatMap((node) => node.events || []),
    unsupported: report.unsupported
  })
}

reports.sort((left, right) => left.source.localeCompare(right.source))
const unsupported = reports.flatMap((report) => report.unsupported.map((item) => ({ source: report.source, ...item })))
const cssFeatures = reports.map((report) => ({ source: report.source, target: report.target, unsupported: report.unsupported.filter(({ kind }) => kind.startsWith('css-')) }))
const webApis = reports.flatMap((report) => report.unsupported.filter(({ kind }) => kind.startsWith('web-api:')).map((item) => ({ source: report.source, ...item })))

await writeFile(join(output, 'components.json'), `${JSON.stringify(reports, null, 2)}\n`)
await writeFile(join(output, 'manual-port-required.json'), `${JSON.stringify(unsupported, null, 2)}\n`)
await writeFile(join(output, 'css-features.json'), `${JSON.stringify(cssFeatures, null, 2)}\n`)
await writeFile(join(output, 'web-api-usage.json'), `${JSON.stringify(webApis, null, 2)}\n`)
console.log(JSON.stringify({ components: reports.length, unsupported: unsupported.length, webApis: webApis.length }))
