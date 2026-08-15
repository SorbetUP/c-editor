import fs from 'node:fs'
import path from 'node:path'

const root = process.cwd()

describe('Freya application launcher', () => {
  it('uses the native Freya runtime for the normal start and dev commands', () => {
    const packageJson = JSON.parse(
      fs.readFileSync(path.join(root, 'package.json'), 'utf8')
    )

    expect(packageJson.scripts.start).toBe('pnpm freya:dev')
    expect(packageJson.scripts.dev).toBe('pnpm freya:dev')
    expect(packageJson.scripts['freya:dev']).toContain(
      'cargo run --manifest-path Elephant/freya/Cargo.toml'
    )
    expect(packageJson.scripts.start).not.toContain('tauri')
    expect(packageJson.scripts.dev).not.toContain('tauri')
  })
})
