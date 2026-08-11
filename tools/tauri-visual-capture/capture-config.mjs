import { createServer } from 'node:net'
import { mkdirSync, writeFileSync } from 'node:fs'
import path from 'node:path'

import { DEFAULT_SHARED_SCENARIO } from './scenario.mjs'

export const root = path.resolve(import.meta.dirname, '../..')
const defaultOutput = path.join(root, 'test-results', 'tauri-visual-capture', new Date().toISOString().replaceAll(/[:.]/g, '-'))

const cliValue = (names) => {
  for (const name of names) {
    const index = process.argv.indexOf(name)
    if (index !== -1 && process.argv[index + 1]) return process.argv[index + 1]
  }
  return null
}

export const outputRoot = path.resolve(process.env.DIFFERENTIAL_OUTPUT_DIR || process.env.ELEPHANT_TAURI_CAPTURE_OUTPUT || cliValue(['--output']) || defaultOutput)
export const scenarioPath = path.resolve(process.env.DIFFERENTIAL_SCENARIO_PATH || process.env.ELEPHANT_TAURI_CAPTURE_SCENARIO || cliValue(['--shared-scenario', '--scenario']) || DEFAULT_SHARED_SCENARIO)
export const frameRoot = path.join(outputRoot, 'frames')
export const requestRoot = path.join(outputRoot, 'physical-requests')
export const delay = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds))
export const writeJson = (filename, value) => writeFileSync(filename, `${JSON.stringify(value, null, 2)}\n`, 'utf8')

export const reserveLocalPort = () => new Promise((resolve, reject) => {
  const server = createServer()
  server.once('error', reject)
  server.listen(0, '127.0.0.1', () => {
    const port = server.address().port
    server.close((error) => error ? reject(error) : resolve(port))
  })
})

export const prepareOutput = () => {
  mkdirSync(frameRoot, { recursive: true })
  mkdirSync(requestRoot, { recursive: true })
}

export const createRunLogger = () => {
  const entries = []
  let processOutput = ''
  return {
    log: (entry) => entries.push({ at: new Date().toISOString(), ...entry }),
    appendProcessOutput: (stream, text) => { processOutput += `[${stream}] ${text}` },
    write: () => {
      writeJson(path.join(outputRoot, 'run-log.json'), entries)
      writeFileSync(path.join(outputRoot, 'tauri-process.log'), processOutput, 'utf8')
    }
  }
}
