import bus from '@/bus'
import notice from '@/services/notification'
import { moveImageToFolder, uploadImage } from '@/util/fileSystem'
import { normalizeInsertedImageSource } from '@/util/imageSource'

export const createEditorImageAction = ({
  getCurrentFile,
  getProjectTree,
  preferencesStore,
  isSourceCode = () => false
}) => async (image, id, alt = '') => {
  const { filename = '', pathname: currentPathname } = getCurrentFile?.() || {}
  const preferences = preferencesStore.$state
  const isTabSavedOnDisk = Boolean(currentPathname)
  let relativeBasePath = isTabSavedOnDisk ? window.path.dirname(currentPathname) : null
  const projectTree = getProjectTree?.()

  if (
    isTabSavedOnDisk &&
    preferences.imageRelativeDirectoryBase !== 'file' &&
    projectTree
  ) {
    const { pathname: rootPath } = projectTree
    if (rootPath && window.fileUtils.isChildOfDirectory(rootPath, currentPathname)) {
      relativeBasePath = rootPath
    }
  }

  const resolveImagePath = (imagePath = '') => {
    const replacement = isTabSavedOnDisk
      ? filename.replace(/\.[^/.]+$/, '')
      : ''
    return imagePath.replace(/\${filename}/g, replacement)
  }

  const globalFolder = resolveImagePath(preferences.imageFolderPath)
  const relativeDirectory = resolveImagePath(preferences.imageRelativeDirectoryName)
  const relativeFolder = relativeBasePath
    ? window.path.join(relativeBasePath, relativeDirectory)
    : null
  let destination = ''

  switch (preferences.imageInsertAction) {
    case 'upload':
      try {
        destination = await uploadImage(currentPathname, image, preferences)
      } catch (error) {
        notice.notify({
          title: 'Upload Image',
          type: 'warning',
          message: error
        })
        destination = await moveImageToFolder(currentPathname, image, globalFolder)
      }
      break
    case 'folder':
      destination = isTabSavedOnDisk && preferences.imagePreferRelativeDirectory
        ? await moveImageToFolder(
          null,
          image,
          relativeFolder,
          true,
          currentPathname
        )
        : await moveImageToFolder(currentPathname, image, globalFolder)
      break
    case 'path':
      if (typeof image === 'string') {
        destination = image
      } else if (isTabSavedOnDisk && preferences.imagePreferRelativeDirectory) {
        destination = await moveImageToFolder(
          null,
          image,
          relativeFolder,
          true,
          currentPathname
        )
      } else {
        destination = await moveImageToFolder(currentPathname, image, globalFolder)
      }
      break
    default:
      destination = typeof image === 'string' ? image : ''
  }

  if (id && isSourceCode()) {
    const baseDirectory = isTabSavedOnDisk
      ? window.path.dirname(currentPathname)
      : window.DIRNAME
    const normalized = normalizeInsertedImageSource(destination, baseDirectory)
    bus.emit('image-action', {
      id,
      result: normalized || destination,
      alt
    })
  }

  return destination
}
