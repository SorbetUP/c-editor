#!/usr/bin/env node

import { readFile, writeFile } from 'node:fs/promises'
import { basename, resolve } from 'node:path'
import { analyzeVueFile } from './ast.mjs'

function usage() {
  console.error('Usage: node tools/vue-to-freya/cli.mjs <Component.vue> [--json <report.json>] [--rust <component.rs>]')
  process.exitCode = 2
}

const args = process.argv.slice(2)
const sourcePath = args.find((arg) => !arg.startsWith('--'))
if (!sourcePath) usage()
if (!sourcePath) process.exit()

const jsonIndex = args.indexOf('--json')
const rustIndex = args.indexOf('--rust')
const jsonPath = jsonIndex >= 0 ? args[jsonIndex + 1] : null
const rustPath = rustIndex >= 0 ? args[rustIndex + 1] : null
if (jsonIndex >= 0 && !jsonPath) usage()
if (rustIndex >= 0 && !rustPath) usage()

const absoluteSource = resolve(sourcePath)
const source = await readFile(absoluteSource, 'utf8')
const report = analyzeVueFile(basename(absoluteSource), source)

if (jsonPath) await writeFile(resolve(jsonPath), `${JSON.stringify(report, null, 2)}\n`)
if (rustPath) await writeFile(resolve(rustPath), `${report.skeleton}\n`)

if (!jsonPath && !rustPath) {
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`)
} else {
  process.stdout.write(JSON.stringify({
    source: report.source,
    target: report.target,
    unsupported: report.unsupported.length,
    json: jsonPath,
    rust: rustPath
  }) + '\n')
}
