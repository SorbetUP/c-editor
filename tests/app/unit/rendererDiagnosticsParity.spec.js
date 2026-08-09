import { describe, expect, it, beforeEach, vi } from 'vitest'
import { JSDOM } from 'jsdom'

import { getDiagnosticLogs, installRendererDiagnostics, pushDiagnosticLog, showDiagnosticOverlay } from '../../../Elephant/frontend/src/renderer/src/platform/rendererDiagnostics.js'

describe('renderer diagnostics parity', () => {
  beforeEach(() => {
    const dom = new JSDOM('<body></body>', { url: 'http://localhost/' })
    globalThis.window = dom.window
    globalThis.document = dom.window.document
    globalThis.localStorage = dom.window.localStorage
  })

  it('stores logs in a bounded renderer-visible buffer', () => {
    pushDiagnosticLog('info', 'hello', { a: 1 })
    const logs = getDiagnosticLogs()
    expect(logs.at(-1).message).toBe('hello')
    expect(logs.at(-1).details).toEqual({ a: 1 })
  })

  it('serializes errors and shows diagnostic overlays', () => {
    const error = new Error('boom')
    pushDiagnosticLog('error', 'failed', error)
    showDiagnosticOverlay('Fatal', error)
    expect(getDiagnosticLogs().at(-1).details.message).toBe('boom')
    expect(document.getElementById('elephant-diagnostic-overlay')?.textContent).toContain('boom')
  })

  it('installs Vue handlers without throwing', () => {
    const app = { config: {} }
    installRendererDiagnostics(app)
    expect(typeof app.config.errorHandler).toBe('function')
    expect(typeof app.config.warnHandler).toBe('function')
    app.config.warnHandler('warn message', null, 'trace')
    expect(getDiagnosticLogs().some((entry) => entry.message.includes('vue warning'))).toBe(true)
  })

  it('forwards logs to Tauri terminal when invoke is available', async() => {
    const invoke = vi.fn(() => Promise.resolve(true))
    window.__TAURI__ = { core: { invoke } }
    pushDiagnosticLog('info', 'terminal-log', { ok: true })
    await Promise.resolve()
    expect(invoke).toHaveBeenCalledWith('tauri_debug_log', expect.objectContaining({
      level: 'info',
      message: 'terminal-log'
    }))
  })
})
