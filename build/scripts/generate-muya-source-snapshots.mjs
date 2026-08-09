import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { pathToFileURL } from 'node:url'

const root = process.cwd()
const args = new Set(process.argv.slice(2))
const optionValue = (name) => {
  const prefix = `${name}=`
  const arg = process.argv.slice(2).find((value) => value.startsWith(prefix))
  return arg ? arg.slice(prefix.length) : null
}

const strictReal = args.has('--strict-real') || process.env.MUYA_STRICT_REAL === '1'
const adapterPath = optionValue('--adapter') || process.env.MUYA_SNAPSHOT_ADAPTER || ''
const parityDir = path.join(root, 'agent/tauri-parity/elephant_tauri', 'parity')
const casesPath = path.join(parityDir, 'muya_deterministic_cases.json')
const outPath = path.join(parityDir, 'muya_source_snapshots.json')
const metaPath = path.join(parityDir, 'muya_source_snapshots_meta.json')

const cases = JSON.parse(fs.readFileSync(casesPath, 'utf8'))

const loadAdapter = async() => {
  if (!adapterPath) return null
  const absolute = path.isAbsolute(adapterPath) ? adapterPath : path.join(root, adapterPath)
  if (!fs.existsSync(absolute)) {
    throw new Error(`Muya snapshot adapter does not exist: ${absolute}`)
  }
  const mod = await import(pathToFileURL(absolute).href)
  const adapter = mod.default || mod
  if (typeof adapter.renderCase !== 'function') {
    throw new Error('Muya snapshot adapter must export renderCase(caseInput)')
  }
  return { adapter, absolute }
}

const contractSnapshotFor = (item) => {
  switch (item.name) {
    case 'rich-frontmatter':
      return {
        name: item.name,
        source: 'contract-adapter',
        expect: {
          frontmatter: {
            title: 'Demo',
            draft: false,
            score: 12,
            ratio: 1.5,
            tags: ['alpha', 'beta'],
            author: { name: 'Noam', role: 'engineer' }
          }
        }
      }
    case 'footnotes':
      return {
        name: item.name,
        source: 'contract-adapter',
        expect: {
          footnotes: {
            definitionLabels: ['a'],
            referenceLabels: ['a'],
            htmlFragments: ['footnotes', 'Footnote text']
          }
        }
      }
    case 'katex-like-math':
      return {
        name: item.name,
        source: 'contract-adapter',
        expect: {
          math: {
            inlineCount: 1,
            blockCount: 1,
            htmlFragments: ['math-inline', 'katex', 'math-block', 'katex-display', 'data-latex']
          }
        }
      }
    case 'diagram-contract':
      return {
        name: item.name,
        source: 'contract-adapter',
        expect: {
          diagrams: {
            count: 1,
            languages: ['mermaid'],
            htmlFragments: ['diagram-block', 'diagram-mermaid', 'data-diagram-language']
          }
        }
      }
    case 'nested-inline-marks':
      return {
        name: item.name,
        source: 'contract-adapter',
        expect: { inlineMarks: { nestedKinds: ['strong_emphasis', 'strike_strong', 'link_code'] } }
      }
    case 'table-alignment':
      return {
        name: item.name,
        source: 'contract-adapter',
        expect: {
          tables: {
            count: 1,
            alignments: ['left', 'center', 'right'],
            columns: 3,
            rows: 1
          }
        }
      }
    default:
      return null
  }
}

const main = async() => {
  const loaded = await loadAdapter()
  if (strictReal && !loaded) {
    throw new Error('Refusing to generate source snapshots without a real Muya adapter. Provide --adapter=path/to/adapter.mjs or MUYA_SNAPSHOT_ADAPTER, or omit --strict-real for contract snapshots.')
  }

  const snapshots = []
  for (const item of cases) {
    if (loaded) {
      const rendered = await loaded.adapter.renderCase(item)
      snapshots.push({
        name: item.name,
        source: 'real-electron-muya-renderer',
        adapter: path.relative(root, loaded.absolute),
        expect: rendered.expect || rendered
      })
      continue
    }
    const fallback = contractSnapshotFor(item)
    if (fallback) snapshots.push(fallback)
  }

  const metadata = {
    generatedAt: new Date().toISOString(),
    mode: loaded ? 'real-electron-muya-renderer' : 'contract-adapter',
    strictReal,
    adapter: loaded ? path.relative(root, loaded.absolute) : null,
    snapshotCount: snapshots.length
  }

  fs.writeFileSync(outPath, `${JSON.stringify(snapshots, null, 2)}\n`)
  fs.writeFileSync(metaPath, `${JSON.stringify(metadata, null, 2)}\n`)
  console.log(`Wrote ${snapshots.length} Muya source snapshots to ${path.relative(root, outPath)}`)
  console.log(`Wrote metadata to ${path.relative(root, metaPath)}`)
  console.log(`mode=${metadata.mode}`)
  if (!loaded) {
    console.log('Not 100% yet: snapshots were generated from contract adapters, not the real Electron/Muya renderer.')
    console.log('Use: node build/scripts/generate-muya-source-snapshots.mjs --strict-real --adapter=build/scripts/adapters/real-muya-renderer.mjs')
  }
}

main().catch((error) => {
  console.error(error.message)
  process.exit(1)
})
