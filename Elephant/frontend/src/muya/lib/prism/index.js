import Prism from 'prismjs'
import { filter } from 'fuzzaldrin'
import initLoadLanguage, { loadedLanguages, transformAliasToOrigin } from './loadLanguage'
import { languages } from 'prismjs/components.js'

const prism = Prism
window.Prism = Prism

import('prismjs/plugins/keep-markup/prism-keep-markup')

const langs = []

for (const name of Object.keys(languages)) {
  const lang = languages[name]
  langs.push({
    name,
    ...lang
  })
  if (lang.alias) {
    if (typeof lang.alias === 'string') {
      langs.push({
        name: lang.alias,
        ...lang
      })
    } else if (Array.isArray(lang.alias)) {
      langs.push(
        ...lang.alias.map((a) => ({
          name: a,
          ...lang
        }))
      )
    }
  }
}

const loadLanguage = initLoadLanguage(Prism)
const preferredLanguages = [
  'python',
  'javascript',
  'typescript',
  'bash',
  'json',
  'yaml',
  'rust',
  'markdown'
]

const uniqueLanguages = (items) => {
  const names = new Set()
  return items.filter((item) => {
    const name = String(item?.name || '').toLowerCase()
    if (!name || names.has(name) || name === 'meta') return false
    names.add(name)
    return true
  })
}

const searchableText = (item) => `${item.name || ''} ${item.title || ''}`.toLowerCase()

const search = (text = '') => {
  const query = String(text).trim().toLowerCase()
  if (!query) {
    return preferredLanguages
      .map((name) => langs.find((item) => item.name === name))
      .filter(Boolean)
  }

  const direct = uniqueLanguages(langs.filter((item) => {
    const name = String(item.name || '').toLowerCase()
    const title = String(item.title || '').toLowerCase()
    return name === query || name.startsWith(query) || title.startsWith(query)
  })).sort((left, right) => {
    const leftName = String(left.name).toLowerCase()
    const rightName = String(right.name).toLowerCase()
    const leftScore = leftName === query ? 0 : leftName.startsWith(query) ? 1 : 2
    const rightScore = rightName === query ? 0 : rightName.startsWith(query) ? 1 : 2
    return leftScore - rightScore || leftName.localeCompare(rightName)
  })

  // Do not mix strong prefix matches with fuzzaldrin's very loose one-letter
  // aliases. This is what previously made "py" suggest c, d, j and q.
  if (direct.length) return direct.slice(0, 8)

  return uniqueLanguages(filter(langs, query, { key: 'name' })
    .filter((item) => searchableText(item).includes(query) || query.length >= 3))
    .slice(0, 8)
}

// pre load latex and yaml and html for `math block` \ `front matter` and `html block`
loadLanguage('latex')
loadLanguage('yaml')

export { search, loadLanguage, loadedLanguages, transformAliasToOrigin }

export default prism
