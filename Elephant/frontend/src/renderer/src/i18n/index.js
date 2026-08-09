import { createI18n } from 'vue-i18n'
import bus from '../bus'
import { isPortableRuntime, readPortablePreference } from '../platform/preferenceStorage'
import {
  APP_DEFAULT_LOCALE,
  APP_LANGUAGE_STORAGE_KEY,
  getAppMessages,
  getSupportedLanguageOptions,
  isRtlLocale,
  mergeMessages,
  normalizeAppLocale,
  resolveStoredLocale
} from 'elephant-front/i18n/appMessages'

const createFallbackEnglishTranslations = () => ({
  about: {
    copyright: 'Copyright',
    copyrightContributors: 'Contributors'
  },
  quickInsert: {
    basicBlock: 'Basic blocks',
    paragraph: { title: 'Paragraph', subtitle: 'Plain text paragraph' },
    horizontalLine: { title: 'Horizontal line', subtitle: 'Add a visual divider' },
    frontMatter: { title: 'Front matter', subtitle: 'YAML metadata block' },
    header: 'Headings',
    header1: { title: 'Heading 1', subtitle: '# Heading' },
    header2: { title: 'Heading 2', subtitle: '## Heading' },
    header3: { title: 'Heading 3', subtitle: '### Heading' },
    header4: { title: 'Heading 4', subtitle: '#### Heading' },
    header5: { title: 'Heading 5', subtitle: '##### Heading' },
    header6: { title: 'Heading 6', subtitle: '###### Heading' },
    advancedBlock: 'Advanced blocks',
    tableBlock: { title: 'Table', subtitle: 'Insert a markdown table' },
    mathFormula: { title: 'Math formula', subtitle: 'Insert a LaTeX math block' },
    htmlBlock: { title: 'HTML block', subtitle: 'Insert raw HTML' },
    codeBlock: { title: 'Code block', subtitle: 'Insert fenced code' },
    quoteBlock: { title: 'Quote block', subtitle: 'Insert a block quote' },
    listBlock: 'Lists',
    orderedList: { title: 'Numbered list', subtitle: 'Create an ordered list' },
    bulletList: { title: 'Bullet list', subtitle: 'Create an unordered list' },
    todoList: { title: 'Task list', subtitle: 'Create a checklist' },
    diagram: 'Diagrams',
    vegaChart: { title: 'Vega-Lite chart', subtitle: 'Insert a Vega-Lite chart block' },
    flowChart: { title: 'Flowchart', subtitle: 'Insert a flowchart block' },
    sequenceChart: { title: 'Sequence diagram', subtitle: 'Insert a sequence diagram block' },
    plantUMLChart: { title: 'PlantUML diagram', subtitle: 'Insert a PlantUML block' },
    mermaid: { title: 'Mermaid diagram', subtitle: 'Insert a Mermaid block' }
  },
  editor: {
    image: {
      toolbar: {
        edit: 'Edit image',
        inline: 'Inline image',
        alignLeft: 'Align left',
        alignCenter: 'Align center',
        alignRight: 'Align right',
        delete: 'Delete image'
      },
      selector: {
        tab: { select: 'Select image', embedLink: 'Embed link' },
        select: { chooseButton: 'Choose image', tip: 'Choose an image from your device.' },
        inputs: { alt: 'Alt text', src: 'Image path or URL', title: 'Title' },
        embedButton: 'Embed image',
        hint: {
          prefix: 'Need alt text or title?',
          full: 'Show more fields',
          simple: 'Show fewer fields'
        }
      }
    },
    'highlight-start': 'Highlight start',
    'highlight-end': 'Highlight end',
    'input-footnote-definition': 'Input footnote definition',
    'input-yaml-front-matter': 'Input YAML front matter',
    'input-language-identifier': 'Input language identifier',
    'input-mathematical-formula': 'Input mathematical formula',
    copyContent: 'Copy code',
    fence: 'Code fence',
    indent: 'Indent',
    'front-matter-delimiter': 'Front matter delimiter',
    'math-delimiter': 'Math delimiter',
    'mermaid-start': 'Mermaid start',
    'flowchart-start': 'Flowchart start',
    'sequence-start': 'Sequence start',
    'plantuml-start': 'PlantUML start',
    'vega-lite-start': 'Vega-Lite start',
    'click-to-add-image': 'Click to add image',
    'load-image-failed': 'Failed to load image'
  },
  frontMenu: {
    duplicate: 'Duplicate',
    turnInto: 'Turn into',
    newParagraph: 'New paragraph',
    delete: 'Delete'
  },
  edit: {
    undo: 'Undo',
    redo: 'Redo',
    cut: 'Cut',
    copy: 'Copy',
    paste: 'Paste',
    selectAll: 'Select all'
  },
  commands: {
    file: {
      changeEncoding: 'Change encoding',
      quickOpen: 'Quick open',
      changeLineEnding: 'Change line ending',
      trailingNewline: 'Trailing newline'
    },
    spellchecker: { switchLanguage: 'Switch spellchecker language' }
  },
  commandPalette: {
    placeholders: {
      selectLanguage: 'Select language',
      selectOption: 'Select an option',
      searchFileToOpen: 'Search a file to open'
    }
  },
  search: {
    searchPlaceholder: 'Search notes...',
    caseSensitive: 'Case sensitive',
    wholeWord: 'Whole word',
    useRegex: 'Use regular expression'
  }
})

const loadLocaleMessages = (locale) => {
  const normalized = normalizeAppLocale(locale)
  if (normalized === APP_DEFAULT_LOCALE) return {}
  try {
    return globalThis.window?.i18nUtils?.loadTranslations?.(normalized) || {}
  } catch (error) {
    console.warn(`⚠️ Failed to load ${normalized} translations, using fallback messages`, error)
    return {}
  }
}

const englishFallback = mergeMessages(createFallbackEnglishTranslations(), getAppMessages('en'))
const initialPreference = globalThis.localStorage?.getItem(APP_LANGUAGE_STORAGE_KEY) || 'system'
const initialLocale = resolveStoredLocale(initialPreference)
const initialMessages = initialLocale === APP_DEFAULT_LOCALE
  ? englishFallback
  : mergeMessages(
      englishFallback,
      mergeMessages(loadLocaleMessages(initialLocale), getAppMessages(initialLocale))
    )
const loadedLocales = new Set(['en', initialLocale])

const i18n = createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: APP_DEFAULT_LOCALE,
  messages: {
    en: englishFallback,
    [initialLocale]: initialMessages
  },
  modifiers: { '@': () => '@' },
  pluralRules: {},
  messageCompiler: {
    compile: (message) => {
      if (typeof message === 'string' && message.includes('|')) return () => message
      return null
    }
  }
})

const applyDocumentLocale = (locale) => {
  if (typeof document === 'undefined') return
  document.documentElement.lang = locale
  document.documentElement.dir = isRtlLocale(locale) ? 'rtl' : 'ltr'
  document.body?.classList.toggle('en-rtl', isRtlLocale(locale))
}

export const t = (key, ...args) => {
  try {
    if (!i18n?.global) return key
    if (typeof i18n.global.te === 'function' && !i18n.global.te(key)) return key
    return i18n.global.t(key, ...args)
  } catch (error) {
    console.error('❌ Translation function error:', error)
    return key
  }
}

export const setLanguage = (requestedLocale, { persist = true } = {}) => {
  const preference = requestedLocale || 'system'
  const locale = resolveStoredLocale(preference)
  if (!loadedLocales.has(locale)) {
    const localeMessages = mergeMessages(
      englishFallback,
      mergeMessages(loadLocaleMessages(locale), getAppMessages(locale))
    )
    i18n.global.setLocaleMessage(locale, localeMessages)
    loadedLocales.add(locale)
  }
  i18n.global.locale.value = locale
  applyDocumentLocale(locale)
  if (persist) globalThis.localStorage?.setItem(APP_LANGUAGE_STORAGE_KEY, preference)
  globalThis.dispatchEvent?.(new CustomEvent('elephantnote:language-changed', {
    detail: { locale, preference }
  }))
  return locale
}

export const getCurrentLanguage = () => i18n.global.locale.value
export const getLanguagePreference = () => globalThis.localStorage?.getItem(APP_LANGUAGE_STORAGE_KEY) || 'system'
export const getLanguageOptions = () => getSupportedLanguageOptions(getCurrentLanguage())

export { i18n }
export default i18n

applyDocumentLocale(initialLocale)

if (window.tauri && window.tauri.ipcRenderer) {
  if (isPortableRuntime()) {
    const language = readPortablePreference('language')
    if (language) {
      setLanguage(language)
      bus.emit('language-changed', getCurrentLanguage())
    }
  }
  window.tauri.ipcRenderer.on('language-changed', (_event, newLocale) => {
    setLanguage(newLocale)
    bus.emit('language-changed', getCurrentLanguage())
  })

  window.tauri.ipcRenderer.send('mt::get-current-language')
  window.tauri.ipcRenderer.on('mt::current-language', (_event, language) => {
    if (!getLanguagePreference() || getLanguagePreference() === 'system') {
      setLanguage(language, { persist: false })
      bus.emit('language-changed', getCurrentLanguage())
    }
  })
}
