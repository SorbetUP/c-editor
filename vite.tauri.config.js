import { createReadStream, existsSync, mkdirSync, statSync, cpSync } from 'fs'
import { resolve, dirname, join, normalize, extname } from 'path'
import { fileURLToPath } from 'url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import svgLoader from 'vite-svg-loader'
import postcssPresetEnv from 'postcss-preset-env'
import packageJson from './package.json' with { type: 'json' }

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
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

export default defineConfig({
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
    __MARKTEXT_VERSION_STRING__: JSON.stringify(`v${packageJson.version}`)
  },
  resolve: {
    alias: {
      ...npmPackageAliases,
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
})
