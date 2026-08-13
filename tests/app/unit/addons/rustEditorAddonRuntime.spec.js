import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it, vi } from 'vitest'

import { createRustEditorRuntimeBinding } from '../../../../Elephant/frontend/src/renderer/src/muya/editorRuntimeResource'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('Rust editor addon runtime', () => {
  it('publishes semantic blocks and document change events', () => {
    const attributes = new Map([
      ['data-elephant-editor-node', '7'],
      ['data-elephant-editor-layer', 'block'],
      ['data-elephant-editor-kind', 'code_block'],
      ['data-language', 'python']
    ])
    const code = {
      getAttribute: (name) => attributes.get(name) ?? null
    }
    const container = {
      querySelectorAll: vi.fn(() => [code])
    }

    const bridge = {
      revision: 3,
      selection: null,
      snapshot: vi.fn(() => ({ revision: 3 })),
      dispatch: vi.fn()
    }
    const binding = createRustEditorRuntimeBinding({
      runtime: { bridge, domContainer: container },
      getMarkdown: () => '```python\nprint(1)\n```'
    })
    const listener = vi.fn()
    binding.resource.watch(listener)

    expect(binding.resource).toMatchObject({
      apiVersion: 1,
      owner: 'elephant.core.editor',
      engine: 'rust'
    })
    expect(binding.resource.queryBlocks({ kind: 'code_block' })[0]).toMatchObject({
      nodeId: 7,
      kind: 'code_block',
      language: 'python'
    })

    binding.notify({ reason: 'document-change' })
    expect(listener).toHaveBeenLastCalledWith(expect.objectContaining({
      engine: 'rust',
      revision: 3,
      reason: 'document-change'
    }))
  })

  it('removes production dependency on Muya DOM selectors', () => {
    const codeExecution = read('addons/official/code-execution/main.js')
    const trusted = read('Elephant/frontend/src/renderer/src/addons/trustedAddonRuntime.js')
    const renderer = read('Elephant/frontend/src/muya/lib/rust/domRenderer/elements.js')
    const runtime = read('Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditor.vue')

    expect(codeExecution).not.toMatch(/muya-code-block|data-function-type=.fencecode|ag-code-block/)
    expect(codeExecution).toContain("queryBlocks?.({ kind: 'code_block' })")
    expect(trusted).not.toContain("host?.get('muya')")
    expect(trusted).toContain("host?.get('editor.runtime')")
    expect(renderer).toContain("data-elephant-editor-kind")
    expect(runtime).toContain('createAddonRuntimeContext({')
    expect(runtime).toContain("'editor.runtime': editorRuntimeBinding.resource")
    expect(runtime).not.toContain('ag-fence-code')
    expect(codeExecution).toContain("runtime?.engine === 'rust'")
  })
})
