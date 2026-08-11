import { appendFile, mkdtemp, readFile, rm, stat } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { bindRun } from '../scenario.mjs'

test('binding creates an absent outputRoot before progress logging', async () => {
  const parent = await mkdtemp(path.join(tmpdir(), 'source-playwright-output-root-'))
  const outputRoot = path.join(parent, 'nested', 'output')
  const fixtureRoot = path.join(parent, 'fixture')
  const environment = {
    DIFFERENTIAL_SCENARIO_PATH: path.resolve('migration/freya/differential-scenarios.json'),
    DIFFERENTIAL_OUTPUT_DIR: outputRoot,
    DIFFERENTIAL_FIXTURE_ROOT: fixtureRoot,
    DIFFERENTIAL_EXPECTED_VIEWPORT_JSON: JSON.stringify({ width: 1280, height: 840, scaleFactor: 1, deviceScaleFactor: 1, fullPage: false, colorScheme: 'light', locale: 'en-US' })
  }
  const previous = Object.fromEntries(Object.keys(environment).map((name) => [name, process.env[name]]))
  try {
    Object.assign(process.env, environment)
    await assert.rejects(stat(outputRoot), /ENOENT/)
    const run = await bindRun()
    await appendFile(path.join(run.outputRoot, 'progress.log'), 'progress\n')
    assert.equal(await readFile(path.join(outputRoot, 'progress.log'), 'utf8'), 'progress\n')
  } finally {
    for (const [name, value] of Object.entries(previous)) {
      if (value === undefined) delete process.env[name]
      else process.env[name] = value
    }
    await rm(parent, { recursive: true, force: true })
  }
})
