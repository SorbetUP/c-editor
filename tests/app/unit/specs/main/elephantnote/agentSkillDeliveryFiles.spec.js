import fs from 'node:fs'
import path from 'node:path'

const root = process.cwd()
const exists = (relativePath) => fs.existsSync(path.join(root, relativePath))
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('agent delivery skill files', () => {
  it('keeps delivery and connector support files present', () => {
    for (const file of [
      'agent/skill/truthful-delivery/SKILL.md',
      'agent/skill/real-implementation/SKILL.md',
      'agent/skill/test-integrity/SKILL.md',
      'agent/skill/completion-audit/SKILL.md',
      'agent/skill/connector-retry-discipline/SKILL.md',
      'agent/skill/apex/steps/step-06-integrity.md',
      'agent/skill/apex/steps/INDEX.md'
    ]) {
      expect(exists(file)).toBe(true)
      expect(read(file)).toContain('# ')
    }
  })
})
