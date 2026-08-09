#!/usr/bin/env node

const { execSync } = require('child_process')
const path = require('path')

const root = path.join(__dirname, '..', '..')

execSync('node build/scripts/minify-locales.mjs', {
  cwd: root,
  stdio: 'inherit',
  env: process.env
})
