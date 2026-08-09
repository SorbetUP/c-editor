#!/usr/bin/env node

import fs from 'node:fs'
import {
  collectSecurityFindings,
  loadSecurityBaseline,
  splitFindingsByBaseline,
  summarizeFindings
} from './security-guardrails-core.mjs'

const root = process.cwd()
const baseline = loadSecurityBaseline(root)
const findings = collectSecurityFindings(root)
const result = splitFindingsByBaseline(findings, baseline)
const summary = summarizeFindings(result)

fs.writeFileSync('security-guardrails-report.json', `${JSON.stringify({
  summary,
  acceptedFindings: result.acceptedFindings,
  newFindings: result.newFindings
}, null, 2)}\n`)

const printFinding = (finding) => {
  console.error(`- [${finding.severity}] ${finding.id} in ${finding.file}`)
  console.error(`  value: ${finding.value}`)
  console.error(`  ${finding.message}`)
}

if (result.acceptedFindings.length) {
  console.warn(`Security guardrails: ${result.acceptedFindings.length} known finding(s) are accepted by the baseline and must be reduced over time.`)
  for (const finding of result.acceptedFindings) {
    console.warn(`- [baseline] ${finding.id} in ${finding.file}: ${finding.value}`)
  }
}

if (result.newFindings.length) {
  console.error(`Security guardrails: ${summary.blockerCount} blocker(s), ${summary.warningCount} warning(s) not accepted by the baseline.`)
  for (const finding of result.newFindings) printFinding(finding)
}

if (summary.hasBlockingFindings) {
  console.error('Security guardrails failed. Fix the issue or add a reviewed baseline entry with a reason if it is intentional temporary debt.')
  process.exit(1)
}

console.log('Security guardrails passed.')
