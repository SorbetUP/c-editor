import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import assert from 'node:assert/strict'

const root = path.resolve(import.meta.dirname, '../../..')
const read = (relative) => readFile(path.join(root, relative), 'utf8')

test('acceptance build exposes the official embedded WebDriver plugin without production registration', async () => {
  const cargo = await read('Elephant/backend/tauri/Cargo.toml')
  const lib = await read('Elephant/backend/tauri/src/lib_min.rs')
  const productionConfig = await read('Elephant/backend/tauri/tauri.conf.json')
  const acceptanceConfig = await read('Elephant/backend/tauri/tauri.wdio.conf.json')
  const wdioConfig = await read('tools/tauri-wdio-embedded/wdio.conf.mjs')
  const runner = await read('tools/tauri-wdio-embedded/tests/differential.e2e.mjs')
  const packageJson = await read('tools/tauri-wdio-embedded/package.json')

  assert.match(cargo, /acceptance-wdio\s*=\s*\[\s*"dep:tauri-plugin-wdio-webdriver"\s*\]/)
  assert.match(cargo, /tauri-plugin-wdio-webdriver\s*=\s*\{[^}]*version\s*=\s*"1"[^}]*optional\s*=\s*true/)
  assert.match(lib, /cfg\(feature\s*=\s*"acceptance-wdio"\)[\s\S]*tauri_plugin_wdio_webdriver::init\(\)/)
  assert.match(productionConfig, /"capabilities"\s*:\s*\[\s*"default"\s*\]/)
  assert.match(acceptanceConfig, /acceptance-wdio/)
  assert.match(acceptanceConfig, /wdio-webdriver:default/)
  assert.match(wdioConfig, /driverProvider:\s*['"]embedded['"]/) 
  assert.match(wdioConfig, /@wdio\/tauri-service/)
  assert.match(packageJson, /"@wdio\/tauri-service":\s*"1\.3\.0"/)
  assert.match(runner, /assert\.equal\(actions\.length, 14/)
  assert.match(runner, /for \(const chord of action\.keysBeforeText \?\? \[\]\)/)
  assert.match(runner, /await editor\.addValue\(action\.text\)/)
  assert.doesNotMatch(runner, /browser\.(execute|executeAsync|$$eval|evaluate)\b/)
  assert.doesNotMatch(runner, /mock|fixture injection|injected success/i)
})
