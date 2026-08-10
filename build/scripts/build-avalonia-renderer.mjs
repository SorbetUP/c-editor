import { cpSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'

const repositoryRoot = resolve(new URL('..', import.meta.url).pathname, '..')
const rendererSource = resolve(repositoryRoot, 'build/out/renderer')
const rendererTarget = resolve(
  repositoryRoot,
  'Elephant/avalonia/src/ElephantNote.Avalonia/Assets/Renderer'
)
const bridgeSource = resolve(repositoryRoot, 'build/scripts/avalonia-renderer-bridge.js')

if (!existsSync(rendererSource)) {
  throw new Error(`Tauri renderer output is missing: ${rendererSource}`)
}

mkdirSync(rendererTarget, { recursive: true })
cpSync(rendererSource, rendererTarget, { recursive: true, force: true })
cpSync(bridgeSource, join(rendererTarget, 'avalonia-renderer-bridge.js'))

const indexPath = join(rendererTarget, 'index.html')
const index = readFileSync(indexPath, 'utf8')
const marker = '<script type="module"'
if (!index.includes('avalonia-renderer-bridge.js')) {
  if (!index.includes(marker)) throw new Error(`Renderer entry has no module script: ${indexPath}`)
  writeFileSync(
    indexPath,
    index.replace(marker, '<script src="./avalonia-renderer-bridge.js"></script>\n    <script type="module"')
  )
}

process.stdout.write(`[avalonia-renderer] copied ${rendererSource} -> ${rendererTarget}\n`)
