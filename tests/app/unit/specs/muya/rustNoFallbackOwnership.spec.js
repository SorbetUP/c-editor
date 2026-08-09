import { readFileSync, readdirSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const root = resolve('Elephant/frontend/src/muya/lib/rust')
const businessRoots = ['inputController', 'bridge.js', 'runtime.js', 'protocol.js']

const sourceFiles = (entry) => {
  const path = resolve(root, entry)
  if (!entry.endsWith('.js')) {
    return readdirSync(path, { withFileTypes: true }).flatMap((item) =>
      item.isDirectory()
        ? []
        : item.name.endsWith('.js')
          ? [resolve(path, item.name)]
          : []
    )
  }
  return [path]
}

const forbidden = [
  [/\bfallback\s*\(/, 'silent JavaScript fallback'],
  [/document\.execCommand\s*\(/, 'browser editing command'],
  [/\.innerHTML\s*=/, 'direct HTML mutation'],
  [/\.textContent\s*=/, 'direct text mutation'],
  [/insertAdjacentHTML\s*\(/, 'direct HTML insertion'],
  [/\.insertNode\s*\(/, 'direct range mutation']
]

describe('Muya Rust business-path ownership', () => {
  for (const file of businessRoots.flatMap(sourceFiles)) {
    it(`${file.slice(root.length + 1)} has no silent JS editing fallback`, () => {
      const source = readFileSync(file, 'utf8')
      for (const [pattern, label] of forbidden) {
        expect(source, `${label} found in ${file}`).not.toMatch(pattern)
      }
    })
  }

  it('owns every supported beforeinput mutation through a Rust command path', () => {
    const controller = readFileSync(resolve(root, 'inputController/index.js'), 'utf8')
    const commands = readFileSync(resolve(root, 'inputController/commands.js'), 'utf8')
    expect(commands).toContain("case 'insertText'")
    expect(commands).toContain("case 'insertParagraph'")
    expect(commands).toContain("case 'deleteContentBackward'")
    expect(controller).toContain("event.inputType === 'deleteContentForward'")
    expect(commands).toContain("case 'historyUndo'")
    expect(commands).toContain("case 'historyRedo'")
  })
})
