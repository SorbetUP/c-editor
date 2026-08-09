import GeneralIcon from '@/assets/icons/pref_general.svg'
import EditorIcon from '@/assets/icons/pref_editor.svg'
import MarkdownIcon from '@/assets/icons/pref_markdown.svg'
import ThemeIcon from '@/assets/icons/pref_theme.svg'
import ImageIcon from '@/assets/icons/pref_image.svg'
import SpellIcon from '@/assets/icons/pref_spellcheck.svg'
import KeyBindingIcon from '@/assets/icons/pref_key_binding.svg'
import { Connection } from '@element-plus/icons-vue'

import preferences from '../../../../common/preferences/schema.json'
import { t } from '../../i18n'

export const getCategory = () => [
  {
    name: t('preferences.categories.general'),
    label: 'general',
    icon: GeneralIcon,
    path: '/preference/general'
  },
  {
    name: 'Sync',
    label: 'rclone',
    icon: Connection,
    path: '/preference/rclone',
    featured: true
  },
  {
    name: t('preferences.categories.editor'),
    label: 'editor',
    icon: EditorIcon,
    path: '/preference/editor'
  },
  {
    name: t('preferences.categories.markdown'),
    label: 'markdown',
    icon: MarkdownIcon,
    path: '/preference/markdown'
  },
  {
    name: t('preferences.categories.spelling'),
    label: 'spelling',
    icon: SpellIcon,
    path: '/preference/spelling'
  },
  {
    name: t('preferences.categories.theme'),
    label: 'theme',
    icon: ThemeIcon,
    path: '/preference/theme'
  },
  {
    name: t('preferences.categories.image'),
    label: 'image',
    icon: ImageIcon,
    path: '/preference/image'
  },
  {
    name: t('preferences.categories.keybindings'),
    label: 'keybindings',
    icon: KeyBindingIcon,
    path: '/preference/keybindings'
  }
]

export const getTranslatedSearchContent = () => {
  const result = []
  Object.keys(preferences).forEach((k) => {
    const { description, enum: emums } = preferences[k]

    if (description.endsWith('--internal')) return

    let [category] = description.split('--')
    let mappedCategory = category.toLowerCase()
    if (category === 'General') mappedCategory = 'general'
    else if (category === 'Editor') mappedCategory = 'editor'
    else if (category === 'Markdown') mappedCategory = 'markdown'
    else if (category === 'Theme') mappedCategory = 'theme'
    else if (category === 'Image') mappedCategory = 'image'
    else if (category === 'View') mappedCategory = 'view'
    else if (category === 'Searcher') mappedCategory = 'searcher'
    else if (category === 'Watcher') mappedCategory = 'watcher'
    else if (category === 'Spelling') mappedCategory = 'spelling'
    else if (category === 'Custom CSS') mappedCategory = 'custom css'
    else mappedCategory = category.toLowerCase().replace(/\s+/g, '-')

    let routeCategory = mappedCategory
    const validRoutes = [
      'general',
      'rclone',
      'editor',
      'markdown',
      'spelling',
      'theme',
      'image',
      'keybindings'
    ]
    if (!validRoutes.includes(routeCategory)) routeCategory = 'general'

    const categoryKey = `preferences.search.categories.${mappedCategory}`
    const itemKey = `preferences.search.items.${k}`

    let translatedCategory = category
    const englishCategory = category
    try {
      translatedCategory = t(categoryKey)
    } catch (e) {
      console.warn(`   ⚠️ 搜索分类翻译失败: ${e.message}`)
      try {
        translatedCategory = t(`preferences.categories.${mappedCategory}`)
      } catch (e2) {
        console.warn(`   ❌ 搜索分类fallback也失败: ${e2.message}`)
        translatedCategory = category
      }
    }

    let translatedPreference = description.split('--')[1] || description
    const englishPreference = description.split('--')[1] || description
    try {
      translatedPreference = t(itemKey)
    } catch (e) {
      console.warn(`   ⚠️ 搜索项目翻译失败: ${e.message}`)
      try {
        translatedPreference = t(`preferences.items.${k}`)
      } catch (e2) {
        console.warn(`   ❌ 搜索项目fallback也失败: ${e2.message}`)
        translatedPreference = description.split('--')[1] || description
      }
    }

    result.push({
      key: k,
      category: translatedCategory,
      categoryEn: englishCategory,
      preference: translatedPreference,
      preferenceEn: englishPreference,
      routeCategory,
      description,
      enum: emums
    })
  })
  return result
}

export const setupLanguageChangeListener = () => {
  const handleLanguageChange = () => {
    if (window.__VUE_I18N__) {
      try {
        const g =
          typeof window.__VUE_I18N__.global === 'function'
            ? window.__VUE_I18N__.global()
            : window.__VUE_I18N__.global
        const currentLanguage = g && g.locale ? g.locale.value || g.locale : 'en'
        window.dispatchEvent(
          new CustomEvent('languageChanged', {
            detail: { language: currentLanguage }
          })
        )
      } catch (e) {
        console.warn('⚠️ 无法获取更新后的语言设置:', e)
      }
    }
  }

  if (window.__VUE_I18N__) {
    try {
      const i18n = window.__VUE_I18N__
      const g = typeof i18n.global === 'function' ? i18n.global() : i18n.global
      if (g && g.locale && g.locale.value !== undefined) {
        // Locale changes are detected by the polling fallback below.
      }
    } catch (e) {
      console.warn('⚠️ 设置语言变化监听器失败:', e)
    }
  }

  setInterval(() => {
    try {
      if (window.__VUE_I18N__) {
        const g =
          typeof window.__VUE_I18N__.global === 'function'
            ? window.__VUE_I18N__.global()
            : window.__VUE_I18N__.global
        const currentLanguage = g && g.locale ? g.locale.value || g.locale : 'en'
        if (currentLanguage !== getTranslatedSearchContent.lastLanguage) {
          getTranslatedSearchContent.lastLanguage = currentLanguage
          handleLanguageChange()
        }
      }
    } catch {
      // Ignore errors and continue checking.
    }
  }, 1000)

  try {
    if (window.__VUE_I18N__) {
      const g =
        typeof window.__VUE_I18N__.global === 'function'
          ? window.__VUE_I18N__.global()
          : window.__VUE_I18N__.global
      getTranslatedSearchContent.lastLanguage = g && g.locale ? g.locale.value || g.locale : 'en'
    }
  } catch {
    getTranslatedSearchContent.lastLanguage = 'en'
  }
}

setupLanguageChangeListener()

export const refreshSearchContent = () => {
  if (getTranslatedSearchContent.lastLanguage) {
    delete getTranslatedSearchContent.lastLanguage
  }

  window.dispatchEvent(
    new CustomEvent('languageChanged', {
      detail: { language: 'force-refresh' }
    })
  )

  return getTranslatedSearchContent()
}

function createDebugPopup() {
  const existingPopup = document.getElementById('debugPopup')
  if (existingPopup) {
    document.body.removeChild(existingPopup)
  }

  const popup = document.createElement('div')
  popup.id = 'debugPopup'
  popup.style.cssText = `
    position: fixed;
    top: 50px;
    right: 20px;
    width: 400px;
    height: 300px;
    background: white;
    border: 2px solid #333;
    z-index: 9999;
    padding: 20px;
    overflow: auto;
  `
  popup.textContent = 'Debug popup'
  document.body.appendChild(popup)
}

export { createDebugPopup }
