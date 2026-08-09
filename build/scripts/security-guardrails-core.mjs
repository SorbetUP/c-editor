import fs from 'node:fs'
import path from 'node:path'

export const DEFAULT_BASELINE_PATH = 'agent/security/guardrails/guardrails-baseline.json'

const BLOCKER = 'blocker'
const WARNING = 'warning'

const IGNORED_SCAN_DIRECTORIES = new Set([
  '.git',
  '.next',
  '.pnpm-store',
  '.tauri',
  'build/coverage',
  'dist',
  'node_modules',
  'out',
  'release',
  'target'
])

const SAFE_ENV_FILE_NAMES = new Set([
  '.env.example',
  '.env.sample',
  '.env.template',
  '.env.defaults'
])

const readText = (filePath) => fs.readFileSync(filePath, 'utf8')

const toRepoPath = (root, absolutePath) => path.relative(root, absolutePath).split(path.sep).join('/')

const readJson = (filePath) => JSON.parse(readText(filePath))

const exists = (filePath) => fs.existsSync(filePath)

const listFiles = (directory, predicate = () => true) => {
  if (!exists(directory)) return []
  const pending = [directory]
  const files = []

  while (pending.length) {
    const current = pending.pop()
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const absolutePath = path.join(current, entry.name)
      if (entry.isDirectory()) {
        if (!IGNORED_SCAN_DIRECTORIES.has(entry.name)) pending.push(absolutePath)
      } else if (entry.isFile() && predicate(absolutePath)) {
        files.push(absolutePath)
      }
    }
  }

  return files.sort()
}

const normalizePermissionValue = (value) => String(value ?? '').trim().replaceAll('\\', '/')

const makeFinding = ({ severity = BLOCKER, id, file, value, message }) => ({
  severity,
  id,
  file,
  value: String(value ?? ''),
  message
})

export const makeFindingKey = (finding) => [finding.id, finding.file, String(finding.value ?? '')].join('\u0000')

export const loadSecurityBaseline = (root = process.cwd(), baselinePath = DEFAULT_BASELINE_PATH) => {
  const absolutePath = path.join(root, baselinePath)
  if (!exists(absolutePath)) return { acceptedFindings: [] }

  const baseline = readJson(absolutePath)
  return {
    ...baseline,
    acceptedFindings: Array.isArray(baseline.acceptedFindings) ? baseline.acceptedFindings : []
  }
}

export const splitFindingsByBaseline = (findings, baseline = { acceptedFindings: [] }) => {
  const acceptedKeys = new Set((baseline.acceptedFindings || []).map(makeFindingKey))
  const acceptedFindings = []
  const newFindings = []

  for (const finding of findings) {
    if (acceptedKeys.has(makeFindingKey(finding))) acceptedFindings.push(finding)
    else newFindings.push(finding)
  }

  return { acceptedFindings, newFindings }
}

const getScopePath = (entry) => {
  if (typeof entry === 'string') return entry
  if (entry && typeof entry === 'object' && typeof entry.path === 'string') return entry.path
  return null
}

const isBroadFsScope = (scopePath) => {
  const value = normalizePermissionValue(scopePath)
  if (!value) return false

  if (['/', '/**', '/**/*', '*', '**', '**/*'].includes(value)) return true

  const broadVariables = [
    '$HOME',
    '$DOCUMENT',
    '$DOWNLOAD',
    '$DESKTOP',
    '$APPDATA',
    '$LOCALDATA',
    '$CONFIG',
    '$CACHE'
  ]

  return broadVariables.some((variableName) => (
    value === variableName ||
    value === `${variableName}/**` ||
    value === `${variableName}/**/*`
  ))
}

const cspDirectiveValue = (csp, directive) => {
  if (!csp) return ''
  if (typeof csp === 'object' && typeof csp[directive] === 'string') return csp[directive]
  if (typeof csp !== 'string') return ''

  const match = csp
    .split(';')
    .map((item) => item.trim())
    .find((item) => item.startsWith(`${directive} `))

  return match ? match.slice(directive.length).trim() : ''
}

const scanTauriConfig = (root, findings) => {
  const configFile = path.join(root, 'Elephant/backend/tauri', 'tauri.conf.json')
  if (!exists(configFile)) return

  const file = toRepoPath(root, configFile)
  let config
  try {
    config = readJson(configFile)
  } catch (error) {
    findings.push(makeFinding({
      id: 'TAURI_CONFIG_INVALID_JSON',
      file,
      value: error.message,
      message: 'Tauri config JSON must be parseable so production security settings can be audited.'
    }))
    return
  }

  if (config?.app?.withGlobalTauri === true) {
    findings.push(makeFinding({
      id: 'TAURI_GLOBAL_API_ENABLED',
      file,
      value: 'withGlobalTauri: true',
      message: 'Production Tauri builds must not expose the global window.__TAURI__ bridge; use explicit imports from @tauri-apps/api instead.'
    }))
  }

  const csp = config?.app?.security?.csp
  const scriptSrc = cspDirectiveValue(csp, 'script-src')
  const connectSrc = cspDirectiveValue(csp, 'connect-src')

  for (const unsafeToken of ["'unsafe-inline'", "'unsafe-eval'"]) {
    if (scriptSrc.includes(unsafeToken)) {
      findings.push(makeFinding({
        id: 'TAURI_CSP_UNSAFE_SCRIPT_SOURCE',
        file,
        value: `script-src ${unsafeToken}`,
        message: 'Production script-src must not allow unsafe inline scripts or eval. Keep WASM exceptions separate and documented when required.'
      }))
    }
  }

  for (const broadLocalhost of ['http://localhost:*', 'http://127.0.0.1:*']) {
    if (connectSrc.includes(broadLocalhost)) {
      findings.push(makeFinding({
        severity: WARNING,
        id: 'TAURI_CSP_BROAD_LOCALHOST_CONNECT',
        file,
        value: `connect-src ${broadLocalhost}`,
        message: 'Prefer exact local runtime ports instead of wildcard localhost connect-src entries in production.'
      }))
    }
  }
}

const scanTauriCapabilities = (root, findings) => {
  const capabilityDirectory = path.join(root, 'Elephant/backend/tauri', 'capabilities')
  const capabilityFiles = listFiles(capabilityDirectory, (filePath) => filePath.endsWith('.json'))

  for (const capabilityFile of capabilityFiles) {
    const file = toRepoPath(root, capabilityFile)
    let capability
    try {
      capability = readJson(capabilityFile)
    } catch (error) {
      findings.push(makeFinding({
        id: 'TAURI_CAPABILITY_INVALID_JSON',
        file,
        value: error.message,
        message: 'Tauri capability JSON must be parseable so permissions can be audited.'
      }))
      continue
    }

    const permissions = Array.isArray(capability.permissions) ? capability.permissions : []
    let hasFsScope = false
    let hasWriteLikeFsPermission = false

    for (const permission of permissions) {
      if (typeof permission === 'string') {
        if (permission === 'fs:default') {
          findings.push(makeFinding({
            id: 'TAURI_FS_DEFAULT_PERMISSION',
            file,
            value: permission,
            message: 'Avoid fs:default; list only the exact filesystem operations the app needs.'
          }))
        }

        if (/^shell:/.test(permission)) {
          findings.push(makeFinding({
            id: 'TAURI_SHELL_PERMISSION',
            file,
            value: permission,
            message: 'Shell permissions are high risk and must not be added without a dedicated threat model.'
          }))
        }

        if (/^process:/.test(permission)) {
          findings.push(makeFinding({
            id: 'TAURI_PROCESS_PERMISSION',
            file,
            value: permission,
            message: 'Process permissions can affect app lifecycle or command execution and must be reviewed explicitly.'
          }))
        }

        if (/^fs:allow-(write-file|mkdir|remove|rename|copy-file)$/.test(permission)) {
          hasWriteLikeFsPermission = true
        }
      } else if (permission && typeof permission === 'object') {
        if (permission.identifier === 'fs:scope') {
          hasFsScope = true
          const allowEntries = Array.isArray(permission.allow) ? permission.allow : []
          for (const entry of allowEntries) {
            const scopePath = getScopePath(entry)
            if (isBroadFsScope(scopePath)) {
              findings.push(makeFinding({
                id: 'TAURI_FS_SCOPE_BROAD',
                file,
                value: normalizePermissionValue(scopePath),
                message: 'Filesystem scopes should be vault-specific or app-data-specific, not broad user directories.'
              }))
            }
          }
        }
      }
    }

    if (hasWriteLikeFsPermission && !hasFsScope) {
      findings.push(makeFinding({
        id: 'TAURI_FS_WRITE_WITHOUT_SCOPE',
        file,
        value: 'write-like fs permission without fs:scope',
        message: 'Write-like filesystem permissions must be constrained by an fs:scope block.'
      }))
    }
  }
}

const scanElectronHardening = (root, findings) => {
  const configFile = path.join(root, 'src', 'main', 'config.js')
  if (exists(configFile)) {
    const content = readText(configFile)
    const file = toRepoPath(root, configFile)
    const checks = [
      {
        id: 'ELECTRON_CONTEXT_ISOLATION_DISABLED',
        pattern: /contextIsolation\s*:\s*false/,
        value: 'contextIsolation: false',
        message: 'Electron renderer windows must keep contextIsolation enabled.'
      },
      {
        id: 'ELECTRON_NODE_INTEGRATION_ENABLED',
        pattern: /nodeIntegration\s*:\s*true/,
        value: 'nodeIntegration: true',
        message: 'Electron renderer windows must not enable Node.js integration.'
      },
      {
        id: 'ELECTRON_WEB_SECURITY_DISABLED',
        pattern: /webSecurity\s*:\s*false/,
        value: 'webSecurity: false',
        message: 'Electron renderer windows must keep Chromium web security enabled.'
      }
    ]

    for (const check of checks) {
      if (check.pattern.test(content)) {
        findings.push(makeFinding({
          id: check.id,
          file,
          value: check.value,
          message: check.message
        }))
      }
    }
  }

  const sourceFiles = listFiles(path.join(root, 'src'), (filePath) => /\.(mjs|js|ts|tsx|vue)$/.test(filePath))
  for (const absolutePath of sourceFiles) {
    const content = readText(absolutePath)
    const file = toRepoPath(root, absolutePath)
    if (/@electron\/remote/.test(content)) {
      findings.push(makeFinding({
        severity: WARNING,
        id: 'ELECTRON_REMOTE_COMPATIBILITY_USAGE',
        file,
        value: '@electron/remote',
        message: '@electron/remote is compatibility attack surface and should be removed after migrating renderer calls to explicit IPC.'
      }))
    }
    if (/from\s+['"]fs-extra['"]/.test(content) && /fileUtils/.test(content)) {
      findings.push(makeFinding({
        severity: WARNING,
        id: 'ELECTRON_PRELOAD_EXPOSES_FILE_UTILS',
        file,
        value: 'fs-extra fileUtils bridge',
        message: 'Do not expose generic filesystem helpers to the renderer; expose audited vault-specific IPC only.'
      }))
    }
  }
}

const scanGitHubWorkflows = (root, findings) => {
  const workflowDirectory = path.join(root, '.github', 'workflows')
  const workflowFiles = listFiles(workflowDirectory, (filePath) => /\.ya?ml$/.test(filePath))

  for (const workflowFile of workflowFiles) {
    const file = toRepoPath(root, workflowFile)
    const content = readText(workflowFile)

    if (/^\s*pull_request_target\s*:/m.test(content)) {
      findings.push(makeFinding({
        id: 'GITHUB_ACTIONS_PULL_REQUEST_TARGET',
        file,
        value: 'pull_request_target',
        message: 'pull_request_target can expose secrets to untrusted PR code and must not be used here.'
      }))
    }

    if (/^\s*permissions\s*:\s*write-all\s*$/m.test(content)) {
      findings.push(makeFinding({
        id: 'GITHUB_ACTIONS_WRITE_ALL',
        file,
        value: 'permissions: write-all',
        message: 'Workflow permissions must be least-privilege, not write-all.'
      }))
    }

    const broadWritePermission = content.match(/^\s*(contents|actions|checks|deployments|id-token|issues|packages|pages|pull-requests|repository-projects|statuses)\s*:\s*write\s*$/m)
    if (broadWritePermission) {
      findings.push(makeFinding({
        id: 'GITHUB_ACTIONS_WRITE_PERMISSION',
        file,
        value: broadWritePermission[0].trim(),
        message: 'Write permissions in CI must be justified by a dedicated release or reporting workflow.'
      }))
    }
  }
}

const scanPackageScripts = (root, findings) => {
  const packageFile = path.join(root, 'package.json')
  if (!exists(packageFile)) return

  const packageJson = readJson(packageFile)
  const scripts = packageJson.scripts || {}
  const dangerousPipePattern = /(curl|wget)\b[^|\n]*\|\s*(sh|bash|zsh|pwsh|powershell)\b/
  const processEnvPattern = /NODE_TLS_REJECT_UNAUTHORIZED\s*=\s*0/

  for (const [scriptName, command] of Object.entries(scripts)) {
    if (dangerousPipePattern.test(command)) {
      findings.push(makeFinding({
        id: 'PACKAGE_SCRIPT_REMOTE_PIPE_TO_SHELL',
        file: 'package.json',
        value: `${scriptName}: ${command}`,
        message: 'Package scripts must not pipe remote content directly into a shell.'
      }))
    }

    if (processEnvPattern.test(command)) {
      findings.push(makeFinding({
        id: 'PACKAGE_SCRIPT_DISABLE_TLS_VALIDATION',
        file: 'package.json',
        value: `${scriptName}: ${command}`,
        message: 'Package scripts must not disable TLS certificate validation.'
      }))
    }
  }
}

const isUnsafeEnvFileName = (fileName) => {
  if (SAFE_ENV_FILE_NAMES.has(fileName)) return false
  if (!fileName.startsWith('.env')) return false
  return fileName === '.env' || fileName.startsWith('.env.') || fileName.startsWith('.env-') || fileName.startsWith('.env_')
}

const isPrivateKeyFileName = (fileName) => /^(id_rsa|id_dsa|id_ecdsa|id_ed25519|.*\.(pem|key|p12|pfx))$/i.test(fileName)

const SECRET_PATTERNS = [
  {
    id: 'SECRET_PRIVATE_KEY_CONTENT',
    pattern: /-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----/,
    message: 'Private key material must never be committed. Use a secret manager or local untracked file.'
  },
  {
    id: 'SECRET_GITHUB_TOKEN',
    pattern: /\bgh[pousr]_[A-Za-z0-9_]{20,}\b/,
    message: 'GitHub tokens must not be committed.'
  },
  {
    id: 'SECRET_OPENAI_COMPATIBLE_TOKEN',
    pattern: /\bsk-[A-Za-z0-9]{32,}\b/,
    message: 'OpenAI-compatible API keys must not be committed.'
  }
]

const shouldScanFileContent = (filePath) => {
  const extension = path.extname(filePath).toLowerCase()
  const textExtensions = new Set([
    '',
    '.c',
    '.conf',
    '.css',
    '.env',
    '.go',
    '.html',
    '.js',
    '.json',
    '.jsx',
    '.md',
    '.mjs',
    '.py',
    '.rs',
    '.sh',
    '.toml',
    '.ts',
    '.tsx',
    '.txt',
    '.vue',
    '.yaml',
    '.yml'
  ])
  return textExtensions.has(extension)
}

const scanCommittedSecrets = (root, findings) => {
  const files = listFiles(root)

  for (const absolutePath of files) {
    const file = toRepoPath(root, absolutePath)
    const fileName = path.basename(file)

    if (isUnsafeEnvFileName(fileName)) {
      findings.push(makeFinding({
        id: 'SECRET_ENV_FILE_COMMITTED',
        file,
        value: fileName,
        message: 'Real .env files must not be committed. Commit .env.example instead.'
      }))
    }

    if (isPrivateKeyFileName(fileName)) {
      findings.push(makeFinding({
        id: 'SECRET_PRIVATE_KEY_FILE',
        file,
        value: fileName,
        message: 'Private key or certificate bundle files must not be committed.'
      }))
    }

    if (!shouldScanFileContent(absolutePath)) continue

    let content = ''
    try {
      content = readText(absolutePath)
    } catch {
      continue
    }

    for (const secretPattern of SECRET_PATTERNS) {
      const match = content.match(secretPattern.pattern)
      if (!match) continue
      findings.push(makeFinding({
        id: secretPattern.id,
        file,
        value: match[0].slice(0, 24),
        message: secretPattern.message
      }))
    }
  }
}

export const collectSecurityFindings = (root = process.cwd()) => {
  const findings = []
  scanTauriConfig(root, findings)
  scanTauriCapabilities(root, findings)
  scanElectronHardening(root, findings)
  scanGitHubWorkflows(root, findings)
  scanPackageScripts(root, findings)
  scanCommittedSecrets(root, findings)
  return findings.sort((left, right) => makeFindingKey(left).localeCompare(makeFindingKey(right)))
}

export const summarizeFindings = ({ acceptedFindings, newFindings }) => {
  const blockerCount = newFindings.filter((finding) => finding.severity === BLOCKER).length
  const warningCount = newFindings.filter((finding) => finding.severity === WARNING).length

  return {
    blockerCount,
    warningCount,
    acceptedCount: acceptedFindings.length,
    hasBlockingFindings: blockerCount > 0
  }
}
