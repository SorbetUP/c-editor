import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const { launchElectron } = require('../../tests/app/e2e/helpers.js')

export async function launchSourceRenderer (run) {
  const launched = await launchElectron([], {
    userDataPath: run.userDataRoot,
    env: {
      ELEPHANTNOTE_CONFIG_DIR: run.configRoot,
      ELEPHANT_E2E_VAULT_ROOT: run.vaultRoot,
      ELEPHANTNOTE_MUYA_RUNTIME: 'rust',
      ELEPHANT_E2E_HIDE_WINDOW: '1',
      ELECTRON_ENABLE_LOGGING: '0'
    }
  })
  const { app, page } = launched
  await page.setViewportSize({ width: run.viewport.width, height: run.viewport.height })
  if (page.viewportSize()?.width !== run.viewport.width || page.viewportSize()?.height !== run.viewport.height) {
    throw new Error('Electron Playwright page viewport did not match the shared scenario')
  }
  await page.waitForSelector('.en-library-grid', { state: 'visible', timeout: 30000 })
  return launched
}
