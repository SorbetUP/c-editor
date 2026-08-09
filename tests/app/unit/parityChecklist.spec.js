import { describe, expect, it } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'

describe('Electron Tauri parity checklist', () => {
  it('keeps all critical parity sections documented', () => {
    const file = path.join(process.cwd(), 'agent/tauri-parity/elephant_tauri/parity/electron_tauri_parity.md')
    const content = fs.readFileSync(file, 'utf8')
    for (const section of ['Startup', 'Vaults', 'Library cards', 'Editor', 'Muya runtime', 'Models', 'Views', 'Error handling']) {
      expect(content).toContain(`## ${section}`)
    }
  })
})
