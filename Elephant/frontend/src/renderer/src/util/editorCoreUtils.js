import path from 'path'

const IMAGE_EXT_REG = /\.(?:jpeg|jpg|png|gif|svg|webp)(?=\?|$)/i
const URL_REG = /^http(s)?:\/\/([a-z0-9\-._~]+\.[a-z]{2,}|[0-9.]+|localhost|\[[a-f0-9.:]+\])(:[0-9]{1,5})?\/[\S]+/i
const DATA_URL_REG = /^data:image\/[\w+-]+(;[\w-]+=[\w-]+|;base64)*,[a-zA-Z0-9+/]+={0,2}$/

export const wordCount = (markdown = '') => {
  const value = String(markdown)
  const paragraph = value.split(/\n{2,}/).filter((line) => line).length
  const removedChinese = value.replace(/[\u4e00-\u9fa5]/g, '')
  const tokens = removedChinese.split(/[\s\n]+/).filter((token) => token)
  const chineseWordLength = value.length - removedChinese.length
  return {
    word: chineseWordLength + tokens.length,
    paragraph,
    character: tokens.reduce((total, token) => total + token.length, 0) + chineseWordLength,
    all: value.length
  }
}

export const getImageInfo = (source, baseUrl = window.DIRNAME) => {
  const src = String(source || '')
  const imageExtension = IMAGE_EXT_REG.test(src)
  const isUrl = URL_REG.test(src) || (imageExtension && /^file:\/\/.+/.test(src))

  if (imageExtension) {
    const isAbsoluteLocal = /^(?:\/|\\|[a-zA-Z]:\\|[a-zA-Z]:\/).+/.test(src)
    if (isUrl || (!isAbsoluteLocal && !baseUrl)) {
      if (!isUrl && !baseUrl) console.warn('"baseUrl" is not defined!')
      return { isUnknownType: false, src }
    }
    return {
      isUnknownType: false,
      src: 'file://' + path.resolve(baseUrl, src)
    }
  }

  if (isUrl) return { isUnknownType: true, src }
  if (DATA_URL_REG.test(src)) return { isUnknownType: false, src }
  return { isUnknownType: false, src: '' }
}

export const escapeHTML = (value = '') => String(value).replace(
  /[&<>'"]/g,
  (character) => ({
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    "'": '&#39;',
    '"': '&quot;'
  })[character] || character
)

export const unescapeHTML = (value = '') => String(value).replace(
  /(?:&amp;|&lt;|&gt;|&quot;|&#39;)/g,
  (entity) => ({
    '&amp;': '&',
    '&lt;': '<',
    '&gt;': '>',
    '&#39;': "'",
    '&quot;': '"'
  })[entity] || entity
)

const downcode = (value) => String(value || '')
  .normalize('NFKD')
  .replace(/[\u0300-\u036f]/g, '')

export class HeadingSlugger {
  constructor() {
    this.seen = Object.create(null)
  }

  slug(value) {
    let slug = downcode(value)
      .replace(/<[!/a-z].*?>/ig, '')
      .replace(/[$%&|<>'"]/g, '')
      .toLowerCase()
      .trim()
      .replace(/[\u2000-\u206F\u2E00-\u2E7F\\'!"#$%&()*+,./:;<=>?@[\]^`{|}~]/g, '')
      .replace(/\s/g, '-')

    if (!slug) slug = 'heading'
    if (Object.prototype.hasOwnProperty.call(this.seen, slug)) {
      const original = slug
      do {
        this.seen[original] += 1
        slug = `${original}-${this.seen[original]}`
      } while (Object.prototype.hasOwnProperty.call(this.seen, slug))
    }
    this.seen[slug] = 0
    return slug
  }
}
