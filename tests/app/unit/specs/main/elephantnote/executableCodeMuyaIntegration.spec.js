// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import Muya from '../../../../../../Elephant/frontend/src/muya/lib'
import { search as searchLanguages } from '../../../../../../Elephant/frontend/src/muya/lib/prism'
import {
  renderExecutableOutput,
  renderExecutableRunButton
} from '../../../../../../Elephant/frontend/src/muya/lib/parser/render/renderBlock/renderExecutableCodeRuntime'
import {
  installExecutableCodeBlocks,
  resetExecutableCodeNativeRuntimeForTests
} from '../../../../../../Elephant/frontend/src/renderer/src/platform/executableCodeBlocks'

const wait = (milliseconds = 0) => new Promise((resolve) => setTimeout(resolve, milliseconds))
const settle = async() => {
  await wait(0)
  await wait(80)
  await wait(0)
}

const INITIAL_MARKDOWN = '```python\nprint("hello")\n```'
const executableCodeExtension = {
  id: 'elephant.code-execution.fenced-code-runtime',
  decorateContainer({ block, children }) {
    if (block?.type !== 'pre' || block?.functionType !== 'fencecode') return children
    return [renderExecutableRunButton(block), ...children, renderExecutableOutput(block)]
  }
}

describe('native executable code integration with real Muya', () => {
  let muya
  let invoke

  beforeEach(() => {
    resetExecutableCodeNativeRuntimeForTests(window)
    document.body.innerHTML = '<div class="en-editor-host muya-container"></div>'
    invoke = vi.fn(async(command, payload) => {
      if (command === 'tauri_programs_list_with_custom') {
        return { executionEnabled: true, outputLineLimit: 200, environments: [], customEnvironments: [] }
      }
      if (command === 'tauri_programs_run_with_custom' && payload.stop) return { stopped: true }
      if (command === 'tauri_programs_run_with_custom') {
        return {
          success: true,
          language: payload.id,
          environment: payload.id,
          stdout: 'hello\n',
          stderr: '',
          exitCode: 0,
          durationMs: 3,
          interrupted: false,
          timedOut: false,
          truncated: false,
          outputLineLimit: 200
        }
      }
      return {}
    })
    window.__TAURI__ = { core: { invoke } }
    globalThis.__ELEPHANT_ADDONS__ = {
      getContributions(area) {
        return area === 'editor.extensions'
          ? [{ addonId: 'elephant.code-execution', contribution: executableCodeExtension }]
          : []
      }
    }
  })

  afterEach(() => {
    resetExecutableCodeNativeRuntimeForTests(window)
    muya?.destroy?.()
    muya = null
    delete window.__TAURI__
    delete globalThis.__ELEPHANT_ADDONS__
    document.body.innerHTML = ''
  })

  const createMuya = async() => {
    const runtime = installExecutableCodeBlocks(window)
    const origin = document.querySelector('.en-editor-host')
    muya = new Muya(origin, {
      markdown: INITIAL_MARKDOWN,
      t: (key) => key
    })
    await settle()
    return { muya, runtime }
  }

  const setNativeLanguage = async(language) => {
    const native = document.querySelector('.ag-language-input')
    const languageBlock = muya.contentState.getBlock(native.id)
    muya.contentState.updateCodeLanguage(languageBlock, language)
    muya.dispatchChange()
    await settle()
  }

  it('renders addon controls without modifying one Markdown character', async() => {
    const created = await createMuya()
    const baseline = muya.getMarkdown()
    const change = vi.fn()
    muya.on('change', change)

    expect(created.runtime.version).toBe('native-v1')
    expect(document.querySelector('pre.ag-fence-code > .en-code-native-run')).not.toBeNull()
    expect(document.querySelector('pre.ag-fence-code > elephant-code-output')).not.toBeNull()
    expect(document.querySelector('.en-code-v6-toolbar')).toBeNull()
    expect(muya.getMarkdown()).toBe(baseline)
    expect(muya.getMarkdown()).toContain('```python')
    expect(muya.getMarkdown()).toContain('print("hello")')
    expect(change).not.toHaveBeenCalled()
  })

  it('does not save partial language identifiers while the user is typing', async() => {
    await createMuya()
    const baseline = muya.getMarkdown()
    const change = vi.fn()
    muya.on('change', change)

    const native = document.querySelector('.ag-language-input')
    native.textContent = 'py'
    native.dispatchEvent(new Event('input', { bubbles: true }))
    await settle()

    expect(muya.getMarkdown()).toBe(baseline)
    expect(change).not.toHaveBeenCalled()

    native.dispatchEvent(new FocusEvent('focusout', { bubbles: true }))
    await settle()

    expect(change).toHaveBeenCalledTimes(1)
    expect(muya.getMarkdown()).toContain('```py')
    expect(muya.getMarkdown()).toContain('print("hello")')
  })

  it('returns relevant and bounded language suggestions', () => {
    const matches = searchLanguages('py').map((item) => item.name)
    expect(matches).toContain('python')
    expect(matches).not.toContain('c')
    expect(matches).not.toContain('d')
    expect(matches).not.toContain('j')
    expect(matches).not.toContain('q')
    expect(matches.length).toBeLessThanOrEqual(8)
  })

  it('uses Muya native language state and runs the selected language', async() => {
    await createMuya()

    const change = vi.fn()
    const input = vi.fn()
    muya.on('change', change)
    muya.container.addEventListener('input', input)

    await setNativeLanguage('javascript')
    document.querySelector('.en-code-native-run').click()
    await settle()

    expect(input).not.toHaveBeenCalled()
    expect(change).toHaveBeenCalledTimes(1)
    expect(muya.getMarkdown()).toContain('```javascript')
    expect(muya.getMarkdown()).toContain('print("hello")')
    expect(muya.getMarkdown()).not.toContain('```python')
    expect(invoke).toHaveBeenCalledWith('tauri_programs_run_with_custom', expect.objectContaining({
      id: 'javascript',
      command: 'print("hello")',
      stop: false
    }))
    const output = document.querySelector('elephant-code-output')
    expect(output.hidden).toBe(false)
    expect(output.shadowRoot.textContent).toContain('hello')
  })

  it('survives repeated native Muya language changes without recursion', async() => {
    await createMuya()

    const change = vi.fn()
    muya.on('change', change)

    for (let index = 0; index < 20; index += 1) {
      await setNativeLanguage(index % 2 === 0 ? 'javascript' : 'python')
    }

    expect(change).toHaveBeenCalledTimes(20)
    expect(muya.getMarkdown()).toContain('```python')
    expect(muya.getMarkdown()).toContain('print("hello")')
    expect(document.querySelectorAll('.en-code-native-run')).toHaveLength(1)
    expect(document.querySelectorAll('elephant-code-output')).toHaveLength(1)
  })
})
