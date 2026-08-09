import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip

const corpus = [
  ['URL autolink', '<https://example.com/path?q=1>'],
  ['email autolink', '<alpha@example.com>'],
  ['inline HTML', 'before <span data-x="1">alpha</span> after'],
  ['HTML block', '<div>\nalpha\n</div>'],
  ['YAML front matter', '---\ntitle: Alpha\ntags:\n  - one\n---\n\nbody'],
  ['TOML front matter', '+++\ntitle = "Alpha"\n+++\n\nbody'],
  ['inline math', 'Euler: $e^{i\\pi}+1=0$.'],
  ['display math', '$$\ne^{i\\pi}+1=0\n$$'],
  ['GitLab display math', '```math\ne^{i\\pi}+1=0\n```'],
  ['emoji shortcode', 'status :rocket: done'],
  ['superscript', 'x^2^ + y^2^'],
  ['subscript', 'H~2~O'],
  ['highlight extension', 'before ==highlight== after'],
  ['reference link', '[alpha][docs]\n\n[docs]: https://example.com "Docs"'],
  ['reference image', '![diagram][asset]\n\n[asset]: /tmp/a.png "Asset"'],
  ['footnote syntax', 'alpha[^note]\n\n[^note]: explanation'],
  ['escaped punctuation', '\\*literal\\* and \\[brackets\\]'],
  ['mixed extension paragraph', ':sparkles: H~2~O costs $x^2$ in <kbd>HTML</kbd>']
]

describeBundled('Muya Markdown corpus differential round trips', () => {
  let jsEditor = null

  beforeAll(initializeRustWasm)

  afterEach(() => {
    jsEditor?.destroy?.()
    jsEditor = null
    document.body.innerHTML = ''
  })

  for (const [name, initial] of corpus) {
    it(name, async () => {
      const result = await runDifferentialTrace({
        initial,
        runJs: () => {},
        runRust: () => {}
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
    })
  }
})
