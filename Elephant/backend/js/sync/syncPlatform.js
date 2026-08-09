export const MOBILE_PLATFORMS = Object.freeze(['android', 'ios'])
export const DESKTOP_RCLONE_PLATFORMS = Object.freeze(['darwin', 'linux', 'win32', 'freebsd', 'openbsd'])

export const normalizeRuntimePlatform = (platform = globalThis.process?.platform || '') => {
  const value = String(platform || '').trim().toLowerCase()
  if (value === 'macos' || value === 'mac') return 'darwin'
  if (value === 'windows') return 'win32'
  return value
}

export const getSyncPlatformCapabilities = (platform = globalThis.process?.platform || '') => {
  const normalized = normalizeRuntimePlatform(platform)
  const mobile = MOBILE_PLATFORMS.includes(normalized)
  const desktopRclone = DESKTOP_RCLONE_PLATFORMS.includes(normalized)
  return {
    platform: normalized,
    desktopRclone,
    mobile,
    mobileRcloneBinary: false,
    mobileSyncRequiresBackend: mobile,
    supported: desktopRclone || mobile,
    recommendedTransport: mobile ? 'remote-backend' : 'rclone-binary'
  }
}
