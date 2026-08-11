import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, writeFileSync } from 'node:fs'
import path from 'node:path'
import { tmpdir } from 'node:os'

import { sha256File } from './manifest.mjs'
import { materializeSharedFixture } from './scenario.mjs'

export const snapshotFiles = (directory, relative = '') => {
  const files = []
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const entryRelative = path.posix.join(relative, entry.name)
    const absolute = path.join(directory, entry.name)
    if (entry.isDirectory()) files.push(...snapshotFiles(absolute, entryRelative))
    else if (entry.isFile()) files.push({ path: entryRelative, sha256: sha256File(absolute) })
  }
  return files.sort((left, right) => left.path.localeCompare(right.path))
}

export const createFixture = async (scenario, { writeJson }) => {
  const suppliedRoot = process.env.DIFFERENTIAL_FIXTURE_ROOT
  const fixtureRoot = suppliedRoot ? path.resolve(suppliedRoot) : mkdtempSync(path.join(tmpdir(), 'elephant-tauri-visual-capture-'))
  const roots = scenario.fixture.roots
  for (const relativeRoot of Object.values(roots)) mkdirSync(path.join(fixtureRoot, relativeRoot), { recursive: true })
  const fixture = await materializeSharedFixture(scenario, fixtureRoot)
  const configRoot = path.join(fixtureRoot, roots.config)
  const userDataRoot = path.join(fixtureRoot, roots.userData)
  const vaultRoot = path.join(fixtureRoot, roots.vault)
  writeJson(path.join(configRoot, 'tauri-vaults.json'), {
    schemaVersion: 1,
    vaults: [{ id: 'e2e-vault', name: 'E2E Vault', path: vaultRoot, icon: 'vault', lastOpenedAt: '2026-06-22T10:00:00.000Z', enabled: true }],
    activeVaultId: 'e2e-vault'
  })
  return { fixtureRoot, vaultRoot, configRoot, userDataRoot, fixture, supplied: Boolean(suppliedRoot), cleanup: !suppliedRoot }
}

const canonicalPath = (value) => {
  if (!value || typeof value !== 'string') return null
  try {
    return realpathSync(value)
  } catch {
    return path.resolve(value)
  }
}

export const proveProfileIsolation = async (client, fixture) => {
  const state = await client.command('readState')
  const expectedVault = canonicalPath(fixture.vaultRoot)
  const activeVault = canonicalPath(state.activeVault)
  const configPath = path.join(fixture.configRoot, 'tauri-vaults.json')
  const config = JSON.parse(readFileSync(configPath, 'utf8'))
  const activeConfig = config.vaults?.find((vault) => vault.id === config.activeVaultId)
  const configuredVault = canonicalPath(activeConfig?.path)
  const userDataMarker = path.join(fixture.userDataRoot, 'screenshot')
  const observations = {
    activeVault: state.activeVault,
    expectedVault: fixture.vaultRoot,
    configuredVault: activeConfig?.path || null,
    configPath,
    configExists: existsSync(configPath),
    userDataRoot: fixture.userDataRoot,
    userDataMarker,
    userDataMarkerExists: existsSync(userDataMarker)
  }
  if (activeVault !== expectedVault) throw new Error(`acceptance fixture is not active: expected ${expectedVault}, observed ${activeVault || '<none>'}`)
  if (configuredVault !== expectedVault) throw new Error(`acceptance config does not point to the fixture vault: expected ${expectedVault}, observed ${configuredVault || '<none>'}`)
  if (!observations.configExists) throw new Error(`acceptance config was not observed at ${configPath}`)
  if (!observations.userDataMarkerExists) throw new Error(`acceptance user-data path was not observed at ${userDataMarker}`)
  return { status: 'proven', homeChanged: false, acceptanceContract: ['ELEPHANT_ACCEPTANCE_TAURI_PORT', 'ELEPHANT_ACCEPTANCE_PROFILE_DIR'], observed: observations }
}
