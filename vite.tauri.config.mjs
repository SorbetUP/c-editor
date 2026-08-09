import {
  cpSync,
  createReadStream,
  existsSync,
  mkdirSync,
  readFileSync,
  statSync
} from 'fs'
import { resolve, dirname, join, normalize, extname } from 'path'
import { fileURLToPath, pathToFileURL } from 'url'
import packageJson from './package.json' with { type: 'json' }

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const exportedPackageEntry = (value) => {
  if (typeof value === 'string') return value
  if (Array.isArray(value)) {
    return value.map(exportedPackageEntry).find(Boolean) || ''
  }
  if (!value || typeof value !== 'object') return ''
  for (const condition of ['import', 'module', 'node', 'default', 'require']) {
    const entry = exportedPackageEntry(value[condition])
    if (entry) return entry
  }
  return Object.values(value).map(exportedPackageEntry).find(Boolean) || ''
}

const importElephantPackage = async (name) => {
  const packageDir = resolve(__dirname, 'Elephant/node_modules', name)
  const manifestPath = join(packageDir, 'package.json')
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'))
  const packageExport = manifest.exports?.['.'] ?? manifest.exports
  const entry = exportedPackageEntry(packageExport) || manifest.module || manifest.main || 'index.js'
  const module = await import(pathToFileURL(resolve(packageDir, entry)).href)
  return module.default || module
}

const [vue, svgLoader, postcssPresetEnv] = await Promise.all([
  importElephantPackage('@vitejs/plugin-vue'),
  importElephantPackage('vite-svg-loader'),
  importElephantPackage('postcss-preset-env')
])

const excalidrawDistDir = resolve(__dirname, 'Elephant/node_modules/@excalidraw/excalidraw/dist')
const npmPackageAliases = Object.fromEntries(
  Object.keys({
    ...(packageJson.dependencies || {}),
    ...(packageJson.devDependencies || {})
  })
    .filter((name) => !['vite', 'prismjs'].includes(name))
    .map((name) => [name, resolve(__dirname, 'Elephant/node_modules', name)])
)
const excalidrawAssetFolders = ['excalidraw-assets', 'excalidraw-assets-dev']
const muyaWasmGenerated = resolve(
  __dirname,
  'Elephant/frontend/src/muya/lib/rust/generated/muya_wasm.js'
)
if (!existsSync(muyaWasmGenerated)) {
  throw new Error('Muya Rust WASM bundle is missing. Run `pnpm muya:wasm:build`.')
}

const isAndroidBuild = process.env.ELEPHANTNOTE_ANDROID_BUILD === '1'
const mobileNodeShim = resolve(
  __dirname,
  'Elephant/frontend/src/renderer/src/platform/mobileNodeBuiltinsShim.js'
)
const mobileNodeAliases = isAndroidBuild
  ? Object.fromEntries(
      [
        'fs/promises',
        'node:fs/promises',
        'child_process',
        'node:child_process',
        'crypto',
        'node:crypto',
        'fs',
        'node:fs',
        'os',
        'node:os',
        'zlib',
        'node:zlib'
      ].map((name) => [name, mobileNodeShim])
    )
  : {}

const contentTypes = {
  '.css': 'text/css; charset=utf-8',
  '.gif': 'image/gif',
  '.jpeg': 'image/jpeg',
  '.jpg': 'image/jpeg',
  '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
  '.wasm': 'application/wasm',
  '.webp': 'image/webp'
}

const decodeRequestPath = (value = '') => {
  try {
    return decodeURIComponent(value)
  } catch {
    return value
  }
}

const resolveExcalidrawAssetRequest = (requestUrl = '') => {
  const [pathname] = String(requestUrl || '').split('?')
  const decodedPathname = decodeRequestPath(pathname || '')
  if (!decodedPathname.startsWith('/excalidraw-assets/')) return ''
  const relativePath = decodedPathname.replace(/^\/excalidraw-assets\//, '')
  const assetPath = resolve(excalidrawDistDir, relativePath)
  const normalizedRoot = normalize(`${excalidrawDistDir}/`)
  if (!normalize(assetPath).startsWith(normalizedRoot)) return ''
  return assetPath
}

const excalidrawAssetsPlugin = () => ({
  name: 'elephantnote-excalidraw-assets',
  configureServer(server) {
    server.middlewares.use((request, response, next) => {
      const assetPath = resolveExcalidrawAssetRequest(request.url || '')
      if (!assetPath) return next()
      try {
        if (!existsSync(assetPath) || !statSync(assetPath).isFile()) return next()
        response.setHeader(
          'Content-Type',
          contentTypes[extname(assetPath).toLowerCase()] || 'application/octet-stream'
        )
        response.setHeader('Cache-Control', 'no-store')
        return createReadStream(assetPath).pipe(response)
      } catch (error) {
        return next(error)
      }
    })
  },
  writeBundle() {
    const targetRoot = resolve(__dirname, 'build/out/renderer/excalidraw-assets')
    mkdirSync(targetRoot, { recursive: true })
    for (const folderName of excalidrawAssetFolders) {
      const source = join(excalidrawDistDir, folderName)
      if (existsSync(source)) {
        cpSync(source, join(targetRoot, folderName), { recursive: true, force: true })
      }
    }
  }
})

export default {
  root: resolve(__dirname, 'Elephant/frontend/src/renderer'),
  base: './',
  build: {
    outDir: resolve(__dirname, 'build/out/renderer'),
    emptyOutDir: true,
    assetsInclude: ['**/*.md']
  },
  define: {
    'process.env.IS_PREACT': JSON.stringify('false'),
    'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV || 'development'),
    __ELEPHANT_MUYA_WASM_BUNDLED__: JSON.stringify(true),
    __ELEPHANTNOTE_ANDROID_BUILD__: JSON.stringify(isAndroidBuild),
    __MARKTEXT_VERSION_STRING__: JSON.stringify(`v${packageJson.version}`)
  },
  resolve: {
    alias: {
      ...npmPackageAliases,
      ...mobileNodeAliases,
      'muya-rust-wasm-bundle': muyaWasmGenerated,
      path: resolve(__dirname, 'Elephant/frontend/src/renderer/src/platform/nodePathShim.js'),
      'node:path': resolve(
        __dirname,
        'Elephant/frontend/src/renderer/src/platform/nodePathShim.js'
      ),
      'elephant-front': resolve(__dirname, 'Elephant/frontend/app'),
      'elephant-shared': resolve(__dirname, 'Elephant/shared'),
      'common/elephantnote': resolve(__dirname, 'Elephant/shared'),
      '@/elephantnote': resolve(__dirname, 'Elephant/frontend/app'),
      '@': resolve(__dirname, 'Elephant/frontend/src/renderer/src'),
      common: resolve(__dirname, 'Elephant/frontend/src/common'),
      muya: resolve(__dirname, 'Elephant/frontend/src/muya')
    },
    extensions: ['.mjs', '.js', '.json', '.vue']
  },
  plugins: [vue(), svgLoader(), excalidrawAssetsPlugin()],
  css: {
    postcss: {
      plugins: [
        postcssPresetEnv({
          stage: 0,
          features: { 'nesting-rules': true }
        })
      ]
    }
  }
}
