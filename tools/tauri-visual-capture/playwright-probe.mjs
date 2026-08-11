import { createRequire } from 'node:module'

export const probePlaywrightTauriAttach = async () => {
  const report = {
    target: 'existing-macos-wkwebview',
    method: 'playwright.webkit.connectOverCDP',
    status: 'unknown',
    playwrightVersion: null,
    detail: null
  }
  try {
    const require = createRequire(import.meta.url)
    const playwright = require('playwright')
    report.playwrightVersion = require('playwright/package.json').version
    await playwright.webkit.connectOverCDP('http://127.0.0.1:1')
    report.status = 'unexpected-success'
    report.detail = 'Playwright unexpectedly accepted a WebKit CDP connection; this must be investigated before use.'
  } catch (error) {
    const message = error?.message || String(error)
    report.status = /only supported in Chromium/i.test(message) ? 'unsupported' : 'probe-error'
    report.detail = message.split('\n')[0]
  }
  return report
}
if (import.meta.url === `file://${process.argv[1]}`) {
  const report = await probePlaywrightTauriAttach()
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`)
  if (report.status === 'probe-error' || report.status === 'unexpected-success') process.exitCode = 1
}
