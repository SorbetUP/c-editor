import { cpSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const scriptDirectory = resolve(fileURLToPath(new URL('.', import.meta.url)))
const repositoryRoot = resolve(scriptDirectory, '../..')
const viteEntry = resolve(repositoryRoot, 'Elephant/node_modules/vite/bin/vite.js')
const configPath = resolve(repositoryRoot, 'vite.muya.config.mjs')
const rustBuildScript = resolve(repositoryRoot, 'build/scripts/build-muya-wasm.mjs')
const generatedDirectory = resolve(repositoryRoot, 'build/out/muya-native')
const avaloniaAssetsDirectory = resolve(
  repositoryRoot,
  'Elephant/avalonia/src/ElephantNote.Avalonia/Assets/Muya'
)

const rustResult = spawnSync(process.execPath, [rustBuildScript], {
  cwd: repositoryRoot,
  env: { ...process.env, NODE_PATH: resolve(repositoryRoot, 'Elephant/node_modules') },
  stdio: 'inherit'
})

if (rustResult.error) throw rustResult.error
if (rustResult.status !== 0) process.exit(rustResult.status ?? 1)

const result = spawnSync(process.execPath, [viteEntry, 'build', '--config', configPath], {
  cwd: repositoryRoot,
  env: { ...process.env, NODE_PATH: resolve(repositoryRoot, 'Elephant/node_modules') },
  stdio: 'inherit'
})

if (result.error) throw result.error
if (result.status !== 0) process.exit(result.status ?? 1)

const nativeIndexPath = resolve(generatedDirectory, 'index.html')
const nativeIndex = readFileSync(nativeIndexPath, 'utf8')
const entryMatch = nativeIndex.match(/<script\s+type="module"[^>]*src="([^"]+)"[^>]*><\/script>/)
if (!entryMatch) throw new Error('Muya native build did not produce a module entry script.')

const entryPath = resolve(generatedDirectory, entryMatch[1])
const inlineEntry = readFileSync(entryPath, 'utf8')
  // The entry is inlined into index.html, so import.meta.url becomes the
  // document URL instead of the original assets/ entry URL. Keep Vite's
  // generated chunks under assets/ while making every relative chunk lookup
  // resolve from the inlined document.
  .replaceAll('"./', '"./assets/')
  .replaceAll("'./", "'./assets/")
  // The WASM binary keeps its generated import namespace; it is not a file
  // lookup relative to the inlined document.
  .replaceAll('"./assets/muya_wasm_bg.js": import0', '"./muya_wasm_bg.js": import0')
  .replace(/<\/script/gi, '<\\\\/script')
const inlineIndex = nativeIndex.replace(
  entryMatch[0],
  () => `<script type="module">\n${inlineEntry}\n</script>`
)
writeFileSync(nativeIndexPath, inlineIndex)

mkdirSync(avaloniaAssetsDirectory, { recursive: true })
cpSync(generatedDirectory, avaloniaAssetsDirectory, { recursive: true, force: true })
