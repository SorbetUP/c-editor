import { defineConfig } from 'vitest/config'
import baseConfig from './vitest.config.js'

export default defineConfig({
  ...baseConfig,
  test: {
    ...baseConfig.test,
    include: ['tests/app/unit/specs/main/elephantnote/*contract.spec.js'],
    coverage: {
      provider: 'v8',
      reportsDirectory: 'build/coverage-critical',
      reporter: ['text', 'json-summary', 'html'],
      include: [
        'Elephant/frontend/app/utils/noteCardView.js',
        'Elephant/frontend/src/renderer/src/platform/rendererPathFacade.js',
        'build/scripts/test-integrity-core.mjs'
      ],
      thresholds: {
        lines: 95,
        functions: 95,
        branches: 90,
        statements: 95
      }
    }
  }
})
