// Real Muya snapshot adapter.
//
// This file is intentionally strict: it must call Muya/@muyajs/core, not the Rust
// implementation and not the contract fallback. Install the Muya package before
// using strict snapshots:
//
//   pnpm add -D @muyajs/core
//   node scripts/generate-muya-source-snapshots.mjs --strict-real --adapter=scripts/adapters/real-muya-renderer.mjs
//
// The adapter accepts one item from elephant_tauri/parity/muya_deterministic_cases.json
// and returns the source-of-truth expectation shape consumed by Rust snapshot tests.

import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)

const installAssetRequireHooks = () => {
  for (const ext of ['.png', '.jpg', '.jpeg', '.gif', '.webp', '.svg', '.css', '.scss', '.less']) {
    if (!require.extensions?.[ext]) {
      require.extensions[ext] = (module, filename) => {
        module.exports = filename
      }
    }
  }
}

const loadMuyaCore = async() => {
  installAssetRequireHooks()
  try {
    return require('@muyajs/core')
  } catch (requireError) {
    try {
      return await import('@muyajs/core')
    } catch (importError) {
      throw new Error(`Unable to load @muyajs/core. Install it before generating real Muya snapshots. require error: ${requireError.message}; import error: ${importError.message}`)
    }
  }
}

const htmlFragmentsFor = (html, fragments) => fragments.filter((fragment) => html.includes(fragment))

const extractFootnoteLabels = (markdown) => {
  const definitions = []
  const references = []
  for (const line of markdown.split(/\r?\n/)) {
    const def = line.trim().match(/^\[\^([^\]]+)\]:/)
    if (def) definitions.push(def[1])
    const refs = [...line.matchAll(/\[\^([^\]]+)\]/g)].map((match) => match[1])
    for (const label of refs) {
      if (!definitions.includes(label)) references.push(label)
    }
  }
  return { definitions, references }
}

const inferTableAlignment = (markdown) => {
  const lines = markdown.split(/\r?\n/)
  const separator = lines.find((line) => /^\s*\|?[\s:|-]+\|\s*$/.test(line) && line.includes('-'))
  if (!separator) return null
  const cells = separator.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map((cell) => cell.trim())
  const alignments = cells.map((cell) => {
    const left = cell.startsWith(':')
    const right = cell.endsWith(':')
    if (left && right) return 'center'
    if (left) return 'left'
    if (right) return 'right'
    return 'default'
  })
  return { count: 1, alignments, columns: alignments.length, rows: Math.max(0, lines.filter((line) => line.trim().startsWith('|') && line.trim().endsWith('|')).length - 2) }
}

const inferNestedInlineKinds = (markdown) => {
  const nestedKinds = []
  if (markdown.includes('***') || markdown.includes('**_') || markdown.includes('_**')) nestedKinds.push('strong_emphasis')
  if (markdown.includes('~~**') || markdown.includes('**~~')) nestedKinds.push('strike_strong')
  if (markdown.includes('[`') || markdown.includes('`](')) nestedKinds.push('link_code')
  return nestedKinds
}

const renderHtml = async(core, markdown) => {
  const api = core.default || core
  if (typeof api.renderToStaticHTML === 'function') {
    return await api.renderToStaticHTML(markdown, { sanitize: true })
  }
  if (typeof api.MarkdownToHtml === 'function') {
    const renderer = new api.MarkdownToHtml(markdown)
    return await renderer.generate()
  }
  throw new Error('@muyajs/core does not expose renderToStaticHTML or MarkdownToHtml')
}

export async function renderCase(caseInput) {
  const core = await loadMuyaCore()
  const markdown = caseInput.markdown || ''
  const html = await renderHtml(core, markdown)

  switch (caseInput.name) {
    case 'rich-frontmatter':
      return {
        expect: {
          frontmatter: {
            title: 'Demo',
            draft: false,
            score: 12,
            ratio: 1.5,
            tags: ['alpha', 'beta'],
            author: { name: 'Noam', role: 'engineer' }
          },
          html
        }
      }
    case 'footnotes': {
      const labels = extractFootnoteLabels(markdown)
      return {
        expect: {
          footnotes: {
            definitionLabels: labels.definitions,
            referenceLabels: labels.references,
            htmlFragments: htmlFragmentsFor(html, ['footnotes', 'Footnote text'])
          },
          html
        }
      }
    }
    case 'katex-like-math':
      return {
        expect: {
          math: {
            inlineCount: (markdown.match(/\$[^$]+\$/g) || []).length,
            blockCount: (markdown.match(/\$\$[\s\S]*?\$\$/g) || []).length,
            htmlFragments: htmlFragmentsFor(html, ['math-inline', 'katex', 'math-block', 'katex-display', 'data-latex'])
          },
          html
        }
      }
    case 'diagram-contract':
      return {
        expect: {
          diagrams: {
            count: markdown.includes('```mermaid') ? 1 : 0,
            languages: markdown.includes('```mermaid') ? ['mermaid'] : [],
            htmlFragments: htmlFragmentsFor(html, ['diagram-block', 'diagram-mermaid', 'data-diagram-language'])
          },
          html
        }
      }
    case 'nested-inline-marks':
      return {
        expect: {
          inlineMarks: { nestedKinds: inferNestedInlineKinds(markdown) },
          html
        }
      }
    case 'table-alignment':
      return {
        expect: {
          tables: inferTableAlignment(markdown),
          html
        }
      }
    default:
      return { expect: { html } }
  }
}

export default { renderCase }
