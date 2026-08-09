/**
 * Common Markdown image contract for core and image addons.
 *
 * The rendered artifact is always a normal Markdown image. Addons may place
 * an optional companion next to it, but the core must never require that
 * companion to render the image.
 */
export const IMAGE_ASSET_EXTENSIONS = Object.freeze([
  'png', 'jpg', 'jpeg', 'gif', 'webp', 'avif', 'bmp', 'ico', 'svg'
])

const IMAGE_DESTINATION_RE = /^([^\s]+)(?:\s+["']([^"']*)["'])?$/

export const parseMarkdownImageDestination = (destination = '') => {
  const value = String(destination || '').trim()
  const match = value.match(IMAGE_DESTINATION_RE)
  return {
    source: match?.[1] || value,
    title: match?.[2] || ''
  }
}

export const isStandardMarkdownImagePath = (value = '') => {
  const source = parseMarkdownImageDestination(value).source
  const pathname = source.split(/[?#]/, 1)[0].replace(/\\/g, '/')
  const extension = pathname.split('.').pop()?.toLowerCase() || ''
  return IMAGE_ASSET_EXTENSIONS.includes(extension)
}

export const markdownImage = (alt = '', source = '', title = '') => {
  const safeAlt = String(alt || '').replace(/[\r\n\]]/g, ' ').trim()
  const safeSource = String(source || '').replace(/[\r\n)]/g, '').trim()
  if (!safeSource) return ''
  const suffix = String(title || '').trim()
  return `![${safeAlt}](${safeSource}${suffix ? ` "${suffix.replace(/[\r\n"]|"/g, ' ')}"` : ''})`
}
