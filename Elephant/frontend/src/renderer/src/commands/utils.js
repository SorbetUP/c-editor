/// Check whether the package is updatable at runtime.
export const isUpdatable = () => {
  // TODO: t('commands.utils.todoUpdateCheck')

  const fileUtils = globalThis.window?.fileUtils
  const pathApi = globalThis.window?.path
  const resourcesPath = globalThis.process?.resourcesPath
  const platform = globalThis.process?.platform
  const env = globalThis.process?.env || {}

  if (typeof fileUtils?.isFile !== 'function' || typeof pathApi?.join !== 'function' || !resourcesPath) {
    return false
  }

  const resFile = fileUtils.isFile(pathApi.join(resourcesPath, 'app-update.yml'))
  if (!resFile) {
    // t('commands.utils.noUpdateResourceFile')
    return false
  } else if (env.APPIMAGE) {
    // We are running as AppImage.
    return true
  } else if (
    platform === 'win32' &&
    fileUtils.isFile(pathApi.join(resourcesPath, 'md.ico'))
  ) {
    // Windows is a little but tricky. The update resource file is always available and
// There is no reliable runtime check for the packaged target type here.
    // As workaround we check whether "md.ico" exists that is only included in the setup.
    return true
  }

  // Otherwise assume that we cannot perform an auto update (standalone binary, archives,
  // packed for package manager).
  return false
}
