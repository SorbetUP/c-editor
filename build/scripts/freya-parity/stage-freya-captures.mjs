#!/usr/bin/env node
import { copyFile, mkdir, readdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'

const args = process.argv.slice(2)
const value = (flag) => {
  const index = args.indexOf(flag)
  if (index < 0 || !args[index + 1]) throw new Error(`Missing ${flag}`)
  return path.resolve(args[index + 1])
}
const source = value('--source')
const output = value('--output')
const config = JSON.parse(await readFile(value('--config'), 'utf8'))
const staged = []
for (const checkpoint of config.checkpoints) {
  const framesDir = path.join(source, 'checkpoints', checkpoint.id, 'frames')
  const frames = (await readdir(framesDir)).filter((name) => name.endsWith('.png')).sort()
  if (!frames.length) throw new Error(`INFRA_ERROR: no Freya PNG frames for ${checkpoint.id}`)
  const selected = frames.at(-1)
  const targetDir = path.join(output, checkpoint.id)
  await mkdir(targetDir, { recursive: true })
  const target = path.join(targetDir, 'static.png')
  await copyFile(path.join(framesDir, selected), target)
  staged.push({ checkpoint: checkpoint.id, sourceFrame: selected, path: path.relative(output, target) })
}
await writeFile(path.join(output, 'staging.json'), `${JSON.stringify({ runtime: 'freya', staged }, null, 2)}\n`)
console.log(`[freya-parity] staged ${staged.length} Freya checkpoints`)
