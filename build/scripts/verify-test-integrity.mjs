import fs from 'node:fs'
import { execFileSync } from 'node:child_process'
import { findForbiddenTestAdditions, validateChangedTestSource } from './test-integrity-core.mjs'

const runGit = (args) => execFileSync('git', args, { encoding: 'utf8' }).trim()
const requestedBase = process.env.TEST_INTEGRITY_BASE || 'origin/develop_next'

const resolveBase = () => {
  const candidates = [requestedBase, 'origin/develop_next', 'develop_next', 'origin/develop', 'develop', 'HEAD^']
  for (const candidate of candidates) {
    if (!candidate) continue
    try {
      runGit(['rev-parse', '--verify', candidate])
      return candidate
    } catch {}
  }
  throw new Error('Unable to resolve a base revision for test-integrity verification')
}

const base = resolveBase()
const range = `${base}...HEAD`
const diff = runGit(['diff', '--unified=0', range, '--', 'tests'])
const changedFiles = runGit(['diff', '--name-only', '--diff-filter=ACMR', range, '--', 'tests'])
  .split(/\r?\n/)
  .filter(Boolean)

const failures = findForbiddenTestAdditions(diff).map(
  (failure) => `added line ${failure.line}: ${failure.rule}: ${failure.source}`
)

for (const filename of changedFiles) {
  if (!fs.existsSync(filename)) continue
  failures.push(...validateChangedTestSource(filename, fs.readFileSync(filename, 'utf8')))
}

if (failures.length) {
  console.error(`[test-integrity] rejected ${failures.length} invalid test change(s) against ${base}`)
  for (const failure of failures) console.error(`- ${failure}`)
  process.exit(1)
}

console.log(`[test-integrity] ok base=${base} changedTestFiles=${changedFiles.length} forbiddenAdditions=0`)
