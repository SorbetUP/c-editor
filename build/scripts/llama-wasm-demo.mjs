#!/usr/bin/env node

import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { createRequire } from 'node:module'
import { File } from 'node:buffer'
import { createWasmLlamaRuntime } from '../Elephant/shared/ai/wasmLlamaRuntime.js'

const parseArgs = (argv = []) => {
  const result = { _: [] }
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index]
    if (!token.startsWith('--')) {
      result._.push(token)
      continue
    }
    const key = token.slice(2)
    const next = argv[index + 1]
    if (next && !next.startsWith('--')) {
      result[key] = next
      index += 1
    } else {
      result[key] = true
    }
  }
  return result
}

const resolveWllamaConfigPaths = ({ wasmPath = '', workerPath = '' } = {}) => {
  if (wasmPath || workerPath) {
    return {
      ...(wasmPath ? { default: wasmPath } : {}),
      ...(workerPath ? { worker: workerPath } : {})
    }
  }

  const require = createRequire(import.meta.url)
  const packageRoot = path.dirname(require.resolve('@wllama/wllama'))
  return {
    default: path.join(packageRoot, 'esm/wasm/wllama.wasm'),
    worker: path.join(packageRoot, 'compat/wasm/wllama.js')
  }
}

const main = async() => {
  const args = parseArgs(process.argv.slice(2))
  const modelPath = String(args.model || args.m || '').trim()
  const prompt = String(args.prompt || args.p || 'Say: local inference OK.').trim()
  const backend = String(args.backend || 'auto').trim()

  if (!modelPath) {
    console.error('Usage: node build/scripts/llama-wasm-demo.mjs --model /path/to/model.gguf [--prompt "Hello"] [--backend auto|cpu|gpu]')
    process.exitCode = 1
    return
  }

  const require = createRequire(import.meta.url)
  const packageRoot = path.dirname(require.resolve('@wllama/wllama'))
  const configPaths = resolveWllamaConfigPaths({
    wasmPath: path.join(packageRoot, 'esm/wasm/wllama.wasm'),
    workerPath: path.join(packageRoot, 'compat/wasm/wllama.js')
  })

  const runtime = createWasmLlamaRuntime({ configPaths })
  const file = new File([await fs.readFile(modelPath)], path.basename(modelPath), {
    type: 'application/octet-stream'
  })
  const session = await runtime.loadModel({
    files: [file],
    backend,
    modelLabel: path.basename(modelPath)
  })
  const response = await session.chat({
    messages: [{ role: 'user', content: prompt }],
    maxTokens: Number(args.max_tokens || 32),
    temperature: Number(args.temperature || 0)
  })

  console.log(response.text || '(empty response)')
}

main().catch((error) => {
  console.error(error instanceof Error ? error.stack || error.message : String(error || 'Unknown error'))
  process.exitCode = 1
})
