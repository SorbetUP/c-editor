const KNOWN_EXISTING_PATHS = new Set()
const HIDDEN_VAULT_ASSET_RE = /(?:^|\/)\.assets\//i
const IMAGE_ASSET_RE = /\.(?:png|jpe?g|gif|webp|svg|avif|bmp|ico)(?:[?#].*)?$/i
const EXCALIDRAW_SCENE_RE = /(?:^|\/)\.assets\/excalidraw-[^/]+\.excalidraw(?:[?#].*)?$/i
const TEXT_FILE_RE = /\.(?:md|markdown|txt|json|jsonl|ya?ml|toml|csv|tsv|html?|css|s[ac]ss|less|jsx?|mjs|cjs|tsx?|vue|xml|svg|excalidraw)(?:[?#].*)?$/i
const ANDROID_PRIVATE_PATH_RE = /^\/data\/(?:data|user\/\d+)\/([^/]+)(\/.*)?$/

let cachedActiveVaultRoot = ''
let activeVaultLookup = null
let androidSyncTimer = null
const ANDROID_ADVANCED_DIRTY_KEY = 'elephantnote:mobile-advanced-vault-dirty-v1'

const normalizeSlashes = (value = '') => String(value || '').replace(/\\/g, '/')
const canonicalizeMacPrivatePath = (value = '') => normalizeSlashes(value).replace(/^\/private(?=\/var\/)/, '')

const fileUrlToPath = (value = '') => {
  const text = String(value || '')
  if (!/^file:/i.test(text)) return text
  try {
    const url = new URL(text)
    let pathname = decodeURIComponent(url.pathname)
    if (/^\/[a-z]:\//i.test(pathname)) pathname = pathname.slice(1)
    if (url.hostname && url.hostname !== 'localhost') return `//${url.hostname}${pathname}`
    return pathname
  } catch {
    return text.replace(/^file:\/\//i, '')
  }
}

const normalizeFilePath = (value = '') => {
  const text = fileUrlToPath(value)
  const pathApi = globalThis.window?.path
  const normalized = normalizeSlashes(text)
  if (!normalized) return normalized
  if (pathApi?.normalize) return normalizeSlashes(pathApi.normalize(normalized))
  const absolute = normalized.startsWith('/') || /^[a-z]:\//i.test(normalized)
  const prefix = absolute ? (normalized.startsWith('/') ? '/' : normalized.slice(0, 3)) : ''
  const body = absolute && prefix ? normalized.slice(prefix.length) : normalized
  const parts = []
  for (const part of body.split('/')) {
    if (!part || part === '.') continue
    if (part === '..') {
      if (parts.length) parts.pop()
      continue
    }
    parts.push(part)
  }
  return `${prefix}${prefix && !prefix.endsWith('/') ? '/' : ''}${parts.join('/')}` || (absolute ? '/' : '.')
}

const parseAndroidPrivatePath = (pathname = '') => {
  const normalized = normalizeFilePath(pathname)
  const match = normalized.match(ANDROID_PRIVATE_PATH_RE)
  if (!match) return null
  return {
    normalized,
    packageName: match[1],
    suffix: match[2] || '',
    packageRoot: normalized.slice(0, normalized.length - (match[2] || '').length)
  }
}

export const canonicalizeAndroidPrivateVaultPath = (pathname = '', vaultRoot = cachedActiveVaultRoot) => {
  const normalized = normalizeFilePath(pathname)
  const candidate = parseAndroidPrivatePath(normalized)
  const activeRoot = parseAndroidPrivatePath(vaultRoot)
  if (!candidate || !activeRoot || candidate.packageName !== activeRoot.packageName) return normalized
  return normalizeFilePath(`${activeRoot.packageRoot}${candidate.suffix}`)
}

const invoke = (target, command, payload = {}) => {
  const caller = target?.__TAURI__?.core?.invoke
  if (typeof caller !== 'function') throw new Error(`Tauri command API is unavailable for ${command}`)
  return caller(command, payload)
}

const refreshActiveVaultRoot = async (target = globalThis) => {
  if (activeVaultLookup) return activeVaultLookup
  const caller = target?.__TAURI__?.core?.invoke
  if (typeof caller !== 'function') return cachedActiveVaultRoot
  activeVaultLookup = caller('tauri_vaults_get')
    .then((payload) => {
      const root = normalizeFilePath(payload?.activeVault?.path || '')
      if (root) cachedActiveVaultRoot = root
      return cachedActiveVaultRoot
    })
    .catch(() => cachedActiveVaultRoot)
    .finally(() => { activeVaultLookup = null })
  return activeVaultLookup
}

const pathInsideRoot = (pathname, root) => {
  const normalizedPath = canonicalizeAndroidPrivateVaultPath(pathname, root)
  const normalizedRoot = canonicalizeAndroidPrivateVaultPath(root, root)
  return Boolean(normalizedRoot && (
    normalizedPath === normalizedRoot || normalizedPath.startsWith(`${normalizedRoot}/`)
  ))
}

const scheduleAndroidTreeSync = (target) => {
  try {
    target.localStorage?.setItem(ANDROID_ADVANCED_DIRTY_KEY, '1')
  } catch {
    // The native shadow remains intact even if WebView storage is constrained.
  }
  target.dispatchEvent?.(new CustomEvent('elephantnote:vault-mutated'))
  target.clearTimeout?.(androidSyncTimer)
  androidSyncTimer = target.setTimeout?.(() => {
    invoke(target, 'tauri_android_vault_sync')
      .then(() => {
        try {
          target.localStorage?.removeItem(ANDROID_ADVANCED_DIRTY_KEY)
        } catch {
          // Ignore constrained WebView storage.
        }
      })
      .catch((error) => {
        if (!/unavailable|not configured/i.test(String(error?.message || error))) {
          console.warn('[tauri:file-utils] Android tree synchronization failed', error)
        }
      })
  }, 180)
}

const bytesToBase64 = async (value) => {
  let bytes
  if (typeof value === 'string') bytes = new TextEncoder().encode(value)
  else if (value instanceof Blob) bytes = new Uint8Array(await value.arrayBuffer())
  else if (value instanceof ArrayBuffer) bytes = new Uint8Array(value)
  else if (ArrayBuffer.isView(value)) bytes = new Uint8Array(value.buffer, value.byteOffset, value.byteLength)
  else if (value?.type === 'Buffer' && Array.isArray(value.data)) bytes = Uint8Array.from(value.data)
  else bytes = new TextEncoder().encode(String(value ?? ''))
  let binary = ''
  for (let offset = 0; offset < bytes.length; offset += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000))
  }
  return btoa(binary)
}

const base64ToBytes = (encoded) => {
  const binary = atob(String(encoded || ''))
  const bytes = new Uint8Array(binary.length)
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index)
  return bytes
}

const readEncoding = (option) => typeof option === 'string' ? option : (option?.encoding || '')
const formatReadResult = (bytes, option, pathname = '') => {
  const encoding = readEncoding(option)
  if (encoding || TEXT_FILE_RE.test(String(pathname || ''))) {
    return new TextDecoder(encoding === 'utf8' ? 'utf-8' : (encoding || 'utf-8')).decode(bytes)
  }
  if (globalThis.Buffer?.from) return globalThis.Buffer.from(bytes)
  return bytes
}

const rememberPath = (pathname, isDirectory = false) => {
  const normalized = canonicalizeAndroidPrivateVaultPath(pathname)
  if (!normalized) return normalized
  KNOWN_EXISTING_PATHS.add(normalized)
  if (!isDirectory) {
    const parent = globalThis.window?.path?.dirname?.(normalized)
    if (parent) KNOWN_EXISTING_PATHS.add(canonicalizeAndroidPrivateVaultPath(parent))
  }
  return normalized
}

const shouldTrustHiddenAssetPath = (pathname = '') => {
  const normalized = canonicalizeAndroidPrivateVaultPath(pathname)
  return HIDDEN_VAULT_ASSET_RE.test(normalized) &&
    (IMAGE_ASSET_RE.test(normalized) || EXCALIDRAW_SCENE_RE.test(normalized))
}

const wrapVaultAwareRead = (target, fileUtils) => {
  const original = fileUtils.readFile
  if (typeof original !== 'function') return
  fileUtils.readFile = async(pathname, options, ...rest) => {
    const root = await refreshActiveVaultRoot(target)
    const resolved = canonicalizeAndroidPrivateVaultPath(pathname, root)
    if (!pathInsideRoot(resolved, root)) return original.call(fileUtils, resolved, options, ...rest)
    const result = await invoke(target, 'tauri_vault_read_binary', { pathname: resolved })
    return formatReadResult(base64ToBytes(result.dataBase64), options, resolved)
  }
}

const wrapVaultAwareWrite = (target, fileUtils, name) => {
  const original = fileUtils[name]
  if (typeof original !== 'function') return
  fileUtils[name] = async(pathname, content, ...rest) => {
    const root = await refreshActiveVaultRoot(target)
    const resolved = canonicalizeAndroidPrivateVaultPath(pathname, root)
    if (!pathInsideRoot(resolved, root)) return original.call(fileUtils, resolved, content, ...rest)
    const result = await invoke(target, 'tauri_vault_write_binary', {
      pathname: resolved,
      dataBase64: await bytesToBase64(content)
    })
    rememberPath(resolved, false)
    scheduleAndroidTreeSync(target)
    return result
  }
}

const wrapVaultAwareEnsureDir = (target, fileUtils) => {
  const original = fileUtils.ensureDir
  if (typeof original !== 'function') return
  fileUtils.ensureDir = async(pathname, ...rest) => {
    const root = await refreshActiveVaultRoot(target)
    const resolved = canonicalizeAndroidPrivateVaultPath(pathname, root)
    if (!pathInsideRoot(resolved, root)) return original.call(fileUtils, resolved, ...rest)
    const result = await invoke(target, 'tauri_vault_ensure_dir', { pathname: resolved })
    rememberPath(resolved, true)
    scheduleAndroidTreeSync(target)
    return result
  }
}

const wrapVaultAwareRemove = (target, fileUtils, name) => {
  const original = fileUtils[name]
  if (typeof original !== 'function') return
  fileUtils[name] = async(pathname, ...rest) => {
    const root = await refreshActiveVaultRoot(target)
    const resolved = canonicalizeAndroidPrivateVaultPath(pathname, root)
    if (!pathInsideRoot(resolved, root)) return original.call(fileUtils, resolved, ...rest)
    const result = await invoke(target, 'tauri_vault_remove_path', { pathname: resolved })
    scheduleAndroidTreeSync(target)
    return result
  }
}

const wrapVaultAwareCopy = (target, fileUtils, name) => {
  const original = fileUtils[name]
  if (typeof original !== 'function') return
  fileUtils[name] = async(source, destination, ...rest) => {
    const root = await refreshActiveVaultRoot(target)
    const resolvedSource = canonicalizeAndroidPrivateVaultPath(source, root)
    const resolvedDestination = canonicalizeAndroidPrivateVaultPath(destination, root)
    if (!pathInsideRoot(resolvedDestination, root)) {
      return original.call(fileUtils, resolvedSource, resolvedDestination, ...rest)
    }
    const bytes = await fileUtils.readFile(resolvedSource)
    await fileUtils.writeFile(resolvedDestination, bytes)
    rememberPath(resolvedDestination, false)
    return resolvedDestination
  }
}

const wrapVaultAwareMove = (target, fileUtils) => {
  const original = fileUtils.move
  if (typeof original !== 'function') return
  fileUtils.move = async(source, destination, ...rest) => {
    const root = await refreshActiveVaultRoot(target)
    const resolvedSource = canonicalizeAndroidPrivateVaultPath(source, root)
    const resolvedDestination = canonicalizeAndroidPrivateVaultPath(destination, root)
    if (pathInsideRoot(resolvedSource, root) && pathInsideRoot(resolvedDestination, root)) {
      const result = await invoke(target, 'tauri_vault_rename_path', {
        source: resolvedSource,
        destination: resolvedDestination
      })
      rememberPath(resolvedDestination, false)
      scheduleAndroidTreeSync(target)
      return result
    }
    return original.call(fileUtils, resolvedSource, resolvedDestination, ...rest)
  }
}

const wrapSyncPathMethod = (fileUtils, name, pathIndexes = [0], mapper = null) => {
  const original = fileUtils?.[name]
  if (typeof original !== 'function') return
  fileUtils[name] = (...args) => {
    const nextArgs = [...args]
    for (const index of pathIndexes) {
      if (typeof nextArgs[index] === 'string') nextArgs[index] = canonicalizeAndroidPrivateVaultPath(nextArgs[index])
    }
    return mapper ? mapper(original, nextArgs) : original.apply(fileUtils, nextArgs)
  }
}

export const resolveTauriVaultPath = async (target = globalThis, pathname = '') => {
  const root = await refreshActiveVaultRoot(target)
  return canonicalizeAndroidPrivateVaultPath(pathname, root)
}

export const installTauriFileUtilsPathGuards = (target = globalThis) => {
  const fileUtils = target?.fileUtils
  if (!target?.__TAURI__ || !fileUtils || target.__ELEPHANT_FILE_UTILS_PATH_GUARDS__) return false
  target.__ELEPHANT_FILE_UTILS_PATH_GUARDS__ = true

  void refreshActiveVaultRoot(target)
  wrapVaultAwareRead(target, fileUtils)
  wrapVaultAwareWrite(target, fileUtils, 'outputFile')
  wrapVaultAwareWrite(target, fileUtils, 'writeFile')
  wrapVaultAwareEnsureDir(target, fileUtils)
  wrapVaultAwareRemove(target, fileUtils, 'remove')
  wrapVaultAwareRemove(target, fileUtils, 'removeFile')
  wrapVaultAwareCopy(target, fileUtils, 'copy')
  wrapVaultAwareCopy(target, fileUtils, 'copyFile')
  wrapVaultAwareMove(target, fileUtils)

  wrapSyncPathMethod(fileUtils, 'pathExistsSync', [0], (original, args) => {
    const pathname = args[0]
    return original.apply(fileUtils, args) ||
      KNOWN_EXISTING_PATHS.has(canonicalizeAndroidPrivateVaultPath(pathname)) ||
      shouldTrustHiddenAssetPath(pathname)
  })
  wrapSyncPathMethod(fileUtils, 'isSamePathSync', [0, 1], (_original, args) =>
    canonicalizeMacPrivatePath(args[0]) === canonicalizeMacPrivatePath(args[1]))
  wrapSyncPathMethod(fileUtils, 'isChildOfDirectory', [0, 1])
  wrapSyncPathMethod(fileUtils, 'isFile', [0], (original, args) =>
    original.apply(fileUtils, args) ||
    KNOWN_EXISTING_PATHS.has(canonicalizeAndroidPrivateVaultPath(args[0])))
  wrapSyncPathMethod(fileUtils, 'isDirectory', [0])

  console.info('[tauri:file-utils] Rust-owned vault filesystem bridge installed')
  return true
}
