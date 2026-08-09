#!/usr/bin/env node

import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const root = resolve(import.meta.dirname, '../..')
const failures = []

const requiredFiles = new Map([
  ['AGENTS.md', [
    'Never rewrite a working feature by hand',
    'Moving a core feature into an add-on is an extraction, not a redesign.',
    'Add-on extraction and parity contract',
    'AI and runtime engines, including Marvin',
    'Ponytail-style engineering constraints',
    'Red-before-green requirement',
    'Real application validation',
    'test:desktop:acceptance:packaged',
    'PROVEN',
    'NOT PROVEN',
    'BLOCKED'
  ]],
  ['agent/skill/elephant-change-safety/SKILL.md', [
    'elephant-change-safety',
    'integrate, do not imitate',
    'prove add-on lifecycle',
    'prove AI/runtime behavior',
    'red/green and sabotage check',
    'real application validation',
    'Required delivery record'
  ]],
  ['agent/skill/truthful-delivery/SKILL.md', [
    'Do not say a feature works without evidence.',
    'Do not call UI/runtime verified from static reading alone.'
  ]],
  ['agent/skill/test-integrity/SKILL.md', [
    'A test is meaningful only if it would fail when the claimed behavior is absent or broken.',
    'mocks the full implementation path'
  ]],
  ['agent/skill/real-implementation/SKILL.md', [
    'the UI action or API entrypoint calls the intended real path',
    'Returning a hardcoded success value.'
  ]],
  ['agent/skill/apex/steps/step-06-integrity.md', [
    'elephant-change-safety',
    'Known-good branch or commit provenance',
    'real Tauri or platform runtime evidence'
  ]],
  ['.github/copilot-instructions.md', [
    'read and obey `/AGENTS.md`',
    'integrate code from known-good branches/commits',
    'packaged acceptance scenario'
  ]],
  ['.github/pull_request_template.md', [
    'Source provenance',
    'Add-on lifecycle',
    'Regression evidence',
    'Real application evidence',
    'Test integrity'
  ]]
])

for (const [relativePath, markers] of requiredFiles) {
  let content
  try {
    content = readFileSync(resolve(root, relativePath), 'utf8')
  } catch (error) {
    failures.push(`${relativePath}: missing or unreadable (${error.message})`)
    continue
  }

  if (!content.trim()) {
    failures.push(`${relativePath}: empty`)
    continue
  }

  for (const marker of markers) {
    if (!content.includes(marker)) {
      failures.push(`${relativePath}: missing mandatory marker ${JSON.stringify(marker)}`)
    }
  }
}

const agents = (() => {
  try {
    return readFileSync(resolve(root, 'AGENTS.md'), 'utf8')
  } catch {
    return ''
  }
})()

const forbiddenWeakeningPatterns = [
  /unit tests alone prove (?:the )?(?:application|feature|runtime) works/i,
  /status endpoint alone proves/i,
  /it is acceptable to recreate a working branch/i,
  /tests may be weakened/i,
  /errors may be silently ignored/i
]

for (const pattern of forbiddenWeakeningPatterns) {
  if (pattern.test(agents)) {
    failures.push(`AGENTS.md: contains policy weakening pattern ${pattern}`)
  }
}

if (failures.length > 0) {
  console.error('[agent-governance] FAILED')
  for (const failure of failures) console.error(`[agent-governance] ${failure}`)
  process.exit(1)
}

console.log(`[agent-governance] OK files=${requiredFiles.size}`)
console.log('[agent-governance] Rules require source provenance, UI preservation, add-on lifecycle proof, real runtime execution, red-before-green tests, logs and truthful claims.')
