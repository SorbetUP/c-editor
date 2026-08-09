import { existsSync } from 'node:fs'
import { defineConfig } from 'vitest/config'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'
import vue from '@vitejs/plugin-vue'
import packageJson from './package.json' with { type: 'json' }

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const apiContractsRuntime = resolve(__dirname, 'Elephant/shared/apiContractsRuntime.js')
const muyaWasmGenerated = resolve(
  __dirname,
  'Elephant/frontend/src/muya/lib/rust/generated/muya_wasm.js'
)
const muyaWasmBundle = existsSync(muyaWasmGenerated)
  ? muyaWasmGenerated
  : resolve(__dirname, 'Elephant/frontend/src/muya/lib/rust/disabledWasm.js')
const npmPackageAliases = Object.fromEntries(
  Object.keys({
    ...(packageJson.dependencies || {}),
    ...(packageJson.devDependencies || {})
  })
    .filter((name) => !['vite', 'prismjs'].includes(name))
    .map((name) => [name, resolve(__dirname, 'Elephant/node_modules', name)])
)

export default defineConfig({
  test: {
    environment: 'jsdom',
    setupFiles: ['tests/app/unit/setup/webgl.js'],
    include: ['tests/app/unit/**/*.spec.js', 'tests/elephant/unit/**/*.spec.js'],
    globals: true,
    coverage: {
      provider: 'v8',
      reportsDirectory: 'build/coverage',
      include: [
        'Elephant/backend/js/sync/RcloneManager.js',
        'Elephant/backend/js/sync/rcloneArgs.js',
        'Elephant/backend/js/sync/rcloneVaultEngine.js'
      ],
      thresholds: {
        lines: 90,
        functions: 90,
        branches: 90,
        statements: 90
      }
    }
  },
  plugins: [vue()],
  resolve: {
    alias: {
      ...npmPackageAliases,
      'muya-rust-wasm-bundle': muyaWasmBundle,
      'electron-log/renderer': resolve(__dirname, 'tests/app/unit/stubs/electronLog.js'),
      'electron-log': resolve(__dirname, 'tests/app/unit/stubs/electronLog.js'),
      'elephant-front': resolve(__dirname, 'Elephant/frontend/app'),
      'elephant-shared': resolve(__dirname, 'Elephant/shared'),
      'common/elephantnote/apiContracts': apiContractsRuntime,
      'common/elephantnote': resolve(__dirname, 'Elephant/shared'),
      '@/elephantnote': resolve(__dirname, 'Elephant/frontend/app'),
      'main_renderer/elephantnote': resolve(__dirname, 'Elephant/backend/js'),
      '@': resolve(__dirname, 'Elephant/frontend/src/renderer/src'),
      common: resolve(__dirname, 'Elephant/frontend/src/common'),
      muya: resolve(__dirname, 'Elephant/frontend/src/muya')
    },
    extensions: ['.mjs', '.js', '.json', '.vue']
  }
})
