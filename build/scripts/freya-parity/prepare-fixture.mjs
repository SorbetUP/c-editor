#!/usr/bin/env node
import { rm, mkdir } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { loadScenario, materializeFixture } from '../../../tools/freya-differential/lib/scenario.mjs'

const args = process.argv.slice(2)
const get = (name) => {
  const index = args.indexOf(name)
  if (index < 0 || !args[index + 1]) throw new Error(`Missing ${name}`)
  return path.resolve(args[index + 1])
}
const scenarioPath = get('--scenario')
const root = get('--root')
await rm(root, { recursive: true, force: true })
await mkdir(root, { recursive: true })
const scenario = await loadScenario(scenarioPath)
const fixture = await materializeFixture(scenario, root)
process.stdout.write(`${JSON.stringify({ scenarioId: scenario.id, fixtureId: fixture.id, root, files: fixture.files }, null, 2)}\n`)
