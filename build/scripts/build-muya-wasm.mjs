import { existsSync, mkdirSync, rmSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const manifest = resolve(root, 'Elephant/crates/muya-wasm/Cargo.toml')
const output = resolve(root, 'Elephant/frontend/src/muya/lib/rust/generated')

const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: 'utf8',
    stdio: options.capture ? 'pipe' : 'inherit',
    env: process.env
  })
  if (result.error) throw result.error
  if (result.status !== 0) {
    const detail = options.capture ? `\n${result.stderr || result.stdout || ''}` : ''
    throw new Error(`${command} exited with status ${result.status}.${detail}`)
  }
  return result.stdout || ''
}

const cargoMetadata = () =>
  JSON.parse(
    run('cargo', ['metadata', '--manifest-path', manifest, '--format-version', '1'], {
      capture: true
    })
  )

const dependencyVersion = (metadata) => {
  const dependency = metadata.packages.find((entry) => entry.name === 'wasm-bindgen')
  if (!dependency) throw new Error('Unable to resolve the wasm-bindgen dependency version.')
  return dependency.version
}

const verifyCli = (expected) => {
  const result = spawnSync('wasm-bindgen', ['--version'], { encoding: 'utf8' })
  const actual = result.status === 0 ? String(result.stdout).trim().split(/\s+/).at(-1) : ''
  if (actual === expected) return
  throw new Error(
    `wasm-bindgen-cli ${expected} is required. Install it with: ` +
      `cargo install wasm-bindgen-cli --version ${expected} --locked`
  )
}

const cleanReleasePackages = () => {
  run('cargo', [
    'clean',
    '--manifest-path',
    manifest,
    '--target',
    'wasm32-unknown-unknown',
    '--release',
    '--package',
    'muya-core',
    '--package',
    'muya-wasm'
  ])
}

const main = () => {
  const metadata = cargoMetadata()
  const version = dependencyVersion(metadata)
  const wasm = resolve(
    metadata.target_directory,
    'wasm32-unknown-unknown/release/muya_wasm.wasm'
  )
  verifyCli(version)
  cleanReleasePackages()
  run('cargo', [
    'build',
    '--manifest-path',
    manifest,
    '--target',
    'wasm32-unknown-unknown',
    '--release'
  ])
  if (!existsSync(wasm)) throw new Error(`Muya WASM output was not created at ${wasm}.`)

  rmSync(output, { recursive: true, force: true })
  mkdirSync(output, { recursive: true })
  run('wasm-bindgen', [
    wasm,
    '--target',
    'web',
    '--out-dir',
    output,
    '--out-name',
    'muya_wasm',
    '--no-typescript'
  ])
  console.log(`[Muya Rust] generated browser package with wasm-bindgen ${version}`)
}

main()
