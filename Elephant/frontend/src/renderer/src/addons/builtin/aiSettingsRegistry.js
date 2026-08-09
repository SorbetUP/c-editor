export const AI_SETTINGS_PAGE_IDS = Object.freeze({
  providers: 'providers',
  chat: 'chat',
  search: 'search',
  ocr: 'ocr'
})

export const AI_SETTINGS_PAGES = Object.freeze([
  Object.freeze({
    id: AI_SETTINGS_PAGE_IDS.providers,
    label: 'Providers',
    icon: 'server',
    slot: '',
    routePage: '',
    addonId: 'elephant.ai',
    order: 60
  }),
  Object.freeze({
    id: AI_SETTINGS_PAGE_IDS.chat,
    label: 'Chat',
    icon: 'message-circle',
    slot: 'ai.chat',
    routePage: 'chat',
    addonId: 'elephant.ai-chat',
    order: 61
  }),
  Object.freeze({
    id: AI_SETTINGS_PAGE_IDS.search,
    label: 'Search',
    icon: 'search',
    slot: 'ai.search',
    routePage: 'embedding',
    addonId: 'elephant.ai-search',
    order: 62
  }),
  Object.freeze({
    id: AI_SETTINGS_PAGE_IDS.ocr,
    label: 'OCR',
    icon: 'scan-text',
    slot: 'ai.ocr',
    routePage: 'ocr',
    addonId: 'elephant.ai-ocr',
    order: 63
  })
])

export const AI_SETTINGS_PAGE_BY_ID = Object.freeze(Object.fromEntries(
  AI_SETTINGS_PAGES.map((page) => [page.id, page])
))

export const AI_PARENT_ADDON_ID = AI_SETTINGS_PAGE_BY_ID.providers.addonId
export const AI_CHILD_ADDON_IDS = Object.freeze([
  ...AI_SETTINGS_PAGES.filter((page) => page.slot).map((page) => page.addonId),
  'elephant.wiki',
  'elephant.graph'
])

export const visibleAiSettingsPages = (settingsContributions = []) => {
  const activeSlots = new Set(settingsContributions
    .filter((entry) => entry?.contribution?.section === 'ai')
    .map((entry) => entry?.contribution?.slot)
    .filter(Boolean))

  return AI_SETTINGS_PAGES.filter((page) => !page.slot || activeSlots.has(page.slot))
}
