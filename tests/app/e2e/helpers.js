const fs = require('fs-extra')
const os = require('os')
const path = require('path')
const { _electron } = require('playwright')

const projectRoot = path.resolve(__dirname, '../../..')
const electronMain = path.join(__dirname, 'electron-main.js')

const getDateAsFilename = () => {
  const date = new Date()
  return '' + date.getFullYear() + (date.getMonth() + 1) + date.getDay()
}

const getTempPath = () => {
  const name =
    'marktext-e2etest-' + getDateAsFilename() + '-' + Math.random().toString(36).slice(2, 8)
  return path.join(os.tmpdir(), name)
}

const electronPackageRoots = () => [
  path.join(projectRoot, 'node_modules', 'electron'),
  path.join(projectRoot, 'Elephant', 'node_modules', 'electron')
]

const getElectronPath = () => {
  const packageRoot = electronPackageRoots().find((candidate) => fs.existsSync(path.join(candidate, 'path.txt')))
  if (!packageRoot) {
    throw new Error(`Electron test binary is not installed. Checked: ${electronPackageRoots().join(', ')}`)
  }
  const relPath = fs.readFileSync(path.join(packageRoot, 'path.txt'), 'utf-8').trim()
  return path.join(packageRoot, 'dist', relPath)
}

const createSeededVaultFixture = async () => {
  const root = getTempPath()
  const userDataPath = path.join(root, 'user-data')
  const configRoot = path.join(root, 'elephant-config')
  const vaultRoot = path.join(root, 'vault')
  const metaRoot = path.join(vaultRoot, '.elephantnote')

  await fs.ensureDir(metaRoot)
  await fs.ensureDir(path.join(vaultRoot, 'Projects'))
  await fs.writeFile(
    path.join(vaultRoot, 'Alpha.md'),
    [
      '---',
      'title: "Alpha note"',
      'type: "note"',
      'tags: ["e2e", "alpha"]',
      '---',
      '',
      '# Alpha note',
      '',
      'Visible alpha body line.',
      ''
    ].join('\n'),
    'utf8'
  )
  await fs.writeFile(
    path.join(vaultRoot, 'Projects', 'Beta.md'),
    [
      '---',
      'title: "Beta project"',
      'type: "note"',
      'tags: ["e2e", "beta"]',
      '---',
      '',
      '# Beta project',
      '',
      'Visible beta body line.',
      ''
    ].join('\n'),
    'utf8'
  )
  await fs.writeJson(
    path.join(metaRoot, 'workspace.json'),
    {
      version: 1,
      vaultName: 'E2E Vault',
      sidebar: [
        { id: 'alpha-note', title: 'Alpha note', type: 'note', path: 'Alpha.md', collapsed: false },
        {
          id: 'projects-folder',
          title: 'Projects',
          type: 'folder',
          path: 'Projects',
          collapsed: false
        }
      ]
    },
    { spaces: 2 }
  )
  await fs.writeJson(
    path.join(metaRoot, 'index.json'),
    {
      version: 1,
      updatedAt: new Date('2026-06-22T10:00:00.000Z').toISOString(),
      entries: []
    },
    { spaces: 2 }
  )
  await fs.writeJson(
    path.join(metaRoot, 'calendar.json'),
    { version: 1, updatedAt: '', events: [] },
    { spaces: 2 }
  )
  await fs.writeJson(
    path.join(metaRoot, 'sources.json'),
    { version: 1, updatedAt: '', sources: [] },
    { spaces: 2 }
  )
  await fs.writeJson(
    path.join(metaRoot, 'wiki.json'),
    { version: 1, updatedAt: '', records: [] },
    { spaces: 2 }
  )
  await fs.ensureDir(configRoot)
  await fs.writeJson(
    path.join(configRoot, 'elephantnote.json'),
    {
      vaults: [{ id: 'e2e-vault', name: 'E2E Vault', path: vaultRoot, icon: 'vault' }],
      activeVaultId: 'e2e-vault'
    },
    { spaces: 2 }
  )

  return { root, userDataPath, configRoot, vaultRoot }
}

const attachPageDiagnostics = (page) => {
  if (page.__elephantDiagnosticsAttached) return
  page.__elephantDiagnosticsAttached = true
  page.on('console', (message) => {
    console.log(`[e2e-renderer:${message.type()}] ${message.text()}`)
  })
  page.on('pageerror', (error) => {
    console.error(`[e2e-renderer:pageerror] ${error?.stack || error?.message || String(error)}`)
  })
  page.on('requestfailed', (request) => {
    console.error(`[e2e-renderer:requestfailed] ${request.method()} ${request.url()} ${request.failure()?.errorText || ''}`)
  })
  page.on('crash', () => {
    console.error('[e2e-renderer:crash] renderer process crashed')
  })
}

const installVisibleErrorObserver = async (page) => {
  await page.evaluate(() => {
    if (window.__ELEPHANT_E2E_VISIBLE_ERROR_OBSERVER__) return
    window.__ELEPHANT_E2E_VISIBLE_ERROR_OBSERVER__ = true
    const reported = new Set()
    const inspect = () => {
      for (const element of document.querySelectorAll('.en-addons-feedback.error')) {
        const style = window.getComputedStyle(element)
        const rect = element.getBoundingClientRect()
        if (style.display === 'none' || style.visibility === 'hidden' || rect.width <= 0 || rect.height <= 0) continue
        const message = String(element.textContent || '').trim()
        if (!message || reported.has(message)) continue
        reported.add(message)
        console.error(`[e2e-visible-addon-error] ${message}`)
      }
    }
    const observer = new MutationObserver(inspect)
    observer.observe(document.documentElement, { childList: true, subtree: true, characterData: true, attributes: true })
    inspect()
  })
}

const launchElectron = async (userArgs, options = {}) => {
  userArgs = userArgs || []
  const executablePath = getElectronPath()
  const userDataPath = options.userDataPath || getTempPath()
  const baseArgs =
    process.platform === 'darwin' ? ['--use-mock-keychain', '--password-store=basic'] : []
  const args = baseArgs.concat([`--user-data-dir=${userDataPath}`, electronMain], userArgs)
  const app = await _electron.launch({
    executablePath,
    args,
    cwd: projectRoot,
    env: {
      ...process.env,
      ELECTRON_ENABLE_LOGGING: '1',
      PERF_TESTING: 'true',
      ELEPHANT_DISABLE_KEYCHAIN_PROMPT: 'true',
      ...(options.env || {})
    },
    timeout: 30000
  })
  const electronProcess = app.process()
  electronProcess.stdout?.on('data', (chunk) => process.stdout.write(`[e2e-electron:stdout] ${chunk}`))
  electronProcess.stderr?.on('data', (chunk) => process.stderr.write(`[e2e-electron:stderr] ${chunk}`))
  app.on('window', attachPageDiagnostics)
  const page = await app.firstWindow()
  attachPageDiagnostics(page)
  await page.waitForLoadState('domcontentloaded')
  await page.waitForTimeout(750)
  await installVisibleErrorObserver(page)
  const bootstrap = await page.evaluate(() => ({
    title: document.title,
    bodyLength: document.body?.innerText?.length || 0,
    hasTauri: Boolean(window.__TAURI__?.core?.invoke),
    hasInternals: Boolean(window.__TAURI_INTERNALS__?.invoke),
    hasElephantApi: Boolean(window.elephantnote?.api),
    html: document.body?.innerHTML?.slice(0, 500) || ''
  })).catch((error) => ({ evaluationError: error?.message || String(error) }))
  console.log(`[e2e-bootstrap] ${JSON.stringify(bootstrap)}`)
  return { app, page, userDataPath }
}

const launchElectronWithSeededVault = async (userArgs = []) => {
  const fixture = await createSeededVaultFixture()
  const launch = await launchElectron(userArgs, {
    userDataPath: fixture.userDataPath,
    env: {
      ELEPHANTNOTE_CONFIG_DIR: fixture.configRoot,
      ELEPHANT_E2E_VAULT_ROOT: fixture.vaultRoot
    }
  })
  return { ...launch, fixture }
}

module.exports = {
  getElectronPath,
  getTempPath,
  createSeededVaultFixture,
  launchElectron,
  launchElectronWithSeededVault
}
