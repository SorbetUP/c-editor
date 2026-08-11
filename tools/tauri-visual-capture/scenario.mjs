import path from 'node:path'

import { loadSharedScenario, materializeSharedFixture, DEFAULT_SHARED_SCENARIO } from './shared-scenario.mjs'

export { DEFAULT_SHARED_SCENARIO, loadSharedScenario, materializeSharedFixture }

export const captureScenario = {
  id: 'elephant-tauri-freya-differential',
  version: 1,
  geometry: { x: 80, y: 60, width: 1280, height: 840 },
  source: path.relative(process.cwd(), DEFAULT_SHARED_SCENARIO)
}
