// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  installExecutableCodeBlocks,
  resetExecutableCodeNativeRuntimeForTests
} from '../../../../../../Elephant/frontend/src/renderer/src/platform/executableCodeBlocks'

const wait = (milliseconds = 0) => new Promise((resolve) => setTimeout(resolve, milliseconds))
const settle = async() => {
  await wait(0)
  await wait(30)
  await wait(0)
}

const blockMarkup = (source, index = 0, language = 'python') => `
  <pre id="code-block-${index}" class="ag-paragraph ag-fence-code language-${language}">
    <button class="en-code-native-run" data-block-key="code-block-${index}" type="button"><span class="en-code-native-run-icon"></span></button>
    <a class="ag-code-copy" contenteditable="false">copy</a>
    <span id="language-${index}" class="ag-language-input">${language}</span>
    <code class="language-${language}">${source}</code>
    <elephant-code-output class="en-code-native-output" data-block-key="code-block-${index}" contenteditable="false"></elephant-code-output>
  </pre>
`

const installEditor = (sources = ['print("hello")']) => {
  document.body.innerHTML = `<main class="muya-container en-editor-host" contenteditable="true">
    ${sources.map((source, index) => blockMarkup(source, index)).join('')}
  </main>`
}

describe('native executable code runtime', () => {
  let invoke
  let clipboardWrite

  beforeEach(() => {
    resetExecutableCodeNativeRuntimeForTests(window)
    document.body.innerHTML = ''
    clipboardWrite = vi.fn(async() => {})
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText: clipboardWrite }
    })
    invoke = vi.fn(async(command, payload) => {
      if (command === 'tauri_programs_list_with_custom') {
        return { executionEnabled: true, outputLineLimit: 200, environments: [], customEnvironments: [] }
      }
      if (command === 'tauri_programs_set_with_custom') return payload.environments
      if (command === 'tauri_programs_run_with_custom' && payload.stop) return { stopped: true }
      if (command === 'tauri_programs_run_with_custom') {
        return {
          success: true,
          language: payload.id,
          environment: payload.id,
          stdout: 'hello\n',
          stderr: '',
          exitCode: 0,
          durationMs: 4,
          interrupted: false,
          timedOut: false,
          truncated: false,
          outputLineLimit: 200
        }
      }
      throw new Error(`Unexpected command: ${command}`)
    })
    window.__TAURI__ = { core: { invoke } }
  })

  afterEach(() => {
    resetExecutableCodeNativeRuntimeForTests(window)
    delete window.__TAURI__
    document.body.innerHTML = ''
  })

  const start = async(sources) => {
    installEditor(sources)
    const runtime = installExecutableCodeBlocks(window)
    await settle()
    return runtime
  }

  it('uses the native pre as the single code and output component', async() => {
    const runtime = await start()
    const pre = document.querySelector('pre.ag-fence-code')
    const run = pre.querySelector(':scope > .en-code-native-run')
    const language = pre.querySelector(':scope > .ag-language-input')
    const code = pre.querySelector(':scope > code')
    const output = pre.querySelector(':scope > elephant-code-output')

    expect(runtime.version).toBe('native-v1')
    expect(run.parentElement).toBe(pre)
    expect(language.parentElement).toBe(pre)
    expect(code.parentElement).toBe(pre)
    expect(output.parentElement).toBe(pre)
    expect(document.querySelector('.en-code-v6-toolbar')).toBeNull()
    expect(document.body.querySelector(':scope > elephant-code-output')).toBeNull()
  })

  it('runs the language and source owned by the native Muya block', async() => {
    const runtime = await start()
    document.querySelector('.en-code-native-run').click()
    await settle()

    expect(invoke).toHaveBeenCalledWith('tauri_programs_run_with_custom', expect.objectContaining({
      id: 'python',
      command: 'print("hello")',
      stop: false
    }))
    const output = document.querySelector('elephant-code-output')
    expect(output.hidden).toBe(false)
    expect(output.shadowRoot.textContent).toContain('hello')
    expect(runtime.states.size).toBe(1)
  })

  it('reads a native Muya language change without any bridge event', async() => {
    await start()
    const pre = document.querySelector('pre')
    const language = pre.querySelector('.ag-language-input')
    language.textContent = 'javascript'
    pre.classList.remove('language-python')
    pre.classList.add('language-javascript')
    document.querySelector('.en-code-native-run').click()
    await settle()

    expect(invoke).toHaveBeenCalledWith('tauri_programs_run_with_custom', expect.objectContaining({
      id: 'javascript'
    }))
  })

  it('turns Run into Stop and sends cancellation for the same execution', async() => {
    let resolveRun
    invoke.mockImplementation(async(command, payload) => {
      if (command === 'tauri_programs_run_with_custom' && payload.stop) return { stopped: true }
      if (command === 'tauri_programs_run_with_custom') {
        return new Promise((resolve) => { resolveRun = resolve })
      }
      return {}
    })
    await start(['while True:\n  pass'])
    const button = document.querySelector('.en-code-native-run')
    button.click()
    await settle()
    expect(button.classList.contains('is-running')).toBe(true)
    expect(button.getAttribute('aria-label')).toBe('Stop code execution')

    button.click()
    await settle()
    expect(invoke).toHaveBeenCalledWith('tauri_programs_run_with_custom', expect.objectContaining({ stop: true }))

    resolveRun({ success: false, interrupted: true, stdout: '', stderr: '', exitCode: null })
    await settle()
  })

  it('restores session output when Muya rerenders the same block key', async() => {
    const runtime = await start()
    document.querySelector('.en-code-native-run').click()
    await settle()
    const state = [...runtime.states.values()][0]
    const oldPre = document.querySelector('pre')
    oldPre.outerHTML = blockMarkup('print("hello")', 0)
    await settle()

    expect(runtime.states.size).toBe(1)
    expect([...runtime.states.values()][0]).toBe(state)
    expect(document.querySelector('elephant-code-output').shadowRoot.textContent).toContain('hello')
  })

  it('keeps one independent state per native code block', async() => {
    const runtime = await start(['print("one")', 'print("two")', 'print("three")'])
    expect(runtime.states.size).toBe(3)
    expect(document.querySelectorAll('pre.ag-fence-code')).toHaveLength(3)
    expect(document.querySelectorAll('.en-code-native-run')).toHaveLength(3)
    expect(document.querySelectorAll('elephant-code-output')).toHaveLength(3)
  })

  it('prunes disconnected blocks without a mutation scan loop', async() => {
    const runtime = await start()
    document.querySelector('pre').remove()
    await wait(1100)
    expect(runtime.states.size).toBe(0)
    expect(document.querySelector('.en-code-native-run')).toBeNull()
    expect(document.querySelector('elephant-code-output')).toBeNull()
  })

  it('does no work when the page or output is scrolled', async() => {
    const runtime = await start()
    const statesBefore = runtime.states.size
    window.dispatchEvent(new Event('scroll'))
    document.dispatchEvent(new Event('scroll'))
    document.querySelector('elephant-code-output').dispatchEvent(new Event('scroll'))
    await settle()
    expect(runtime.states.size).toBe(statesBefore)
  })
})
