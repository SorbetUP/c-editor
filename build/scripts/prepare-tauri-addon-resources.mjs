#!/usr/bin/env node

import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs'
import { basename, join, resolve } from 'node:path'

const root = resolve(import.meta.dirname, '../..')
const sourceRoot = join(root, 'addons')
const destinationRoot = join(root, 'Elephant', 'backend', 'tauri', 'resources', 'official-addons')

if (!existsSync(join(sourceRoot, 'catalog.json')) || !existsSync(join(sourceRoot, 'official'))) {
  throw new Error('[addons] cannot prepare Tauri resources: addons/catalog.json or addons/official is missing')
}

rmSync(destinationRoot, { recursive: true, force: true })
mkdirSync(destinationRoot, { recursive: true })
cpSync(join(sourceRoot, 'catalog.json'), join(destinationRoot, 'catalog.json'))
cpSync(join(sourceRoot, 'official'), join(destinationRoot, 'official'), {
  recursive: true,
  filter(source) {
    return !['target', 'node_modules', 'releases'].includes(basename(source))
  }
})

console.log(`[addons] prepared Tauri official addon resources at ${destinationRoot}`)
