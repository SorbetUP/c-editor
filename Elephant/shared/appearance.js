/* eslint-disable @stylistic/object-property-newline */
export const ELEPHANTNOTE_THEME_STORAGE_KEY = 'elephantnote:theme'
export const ELEPHANTNOTE_DEFAULT_THEME = 'light'

const rgba = (rgb, alpha) => `rgba(${rgb.join(', ')}, ${alpha})`

const createThemeTokens = ({
  bg,
  surface,
  sidebar,
  soft,
  softStrong,
  border,
  borderStrong,
  text,
  textRgb,
  muted,
  subtle,
  primary,
  primaryRgb,
  danger,
  shadow,
  editorBg = bg,
  codeBg = surface,
  floatShadow = shadow
}) => ({
  '--en-bg': bg,
  '--en-surface': surface,
  '--en-sidebar-bg': sidebar,
  '--en-soft': soft,
  '--en-soft-strong': softStrong,
  '--en-border': border,
  '--en-border-strong': borderStrong,
  '--en-text': text,
  '--en-muted': muted,
  '--en-subtle': subtle,
  '--en-primary': primary,
  '--en-danger': danger,
  '--en-card-shadow': shadow,
  '--themeColor': primary,
  '--selectionColor': rgba(primaryRgb, 0.2),
  '--deleteColor': danger,
  '--itemBgColor': soft,
  '--floatBgColor': surface,
  '--floatHoverColor': softStrong,
  '--floatBorderColor': borderStrong,
  '--floatShadow': floatShadow,
  '--editorBgColor': editorBg,
  '--editorColor': rgba(textRgb, 0.88),
  '--editorColor80': rgba(textRgb, 0.8),
  '--editorColor60': rgba(textRgb, 0.62),
  '--editorColor50': rgba(textRgb, 0.52),
  '--editorColor40': rgba(textRgb, 0.42),
  '--editorColor30': rgba(textRgb, 0.32),
  '--editorColor10': rgba(textRgb, 0.12),
  '--editorColor04': rgba(textRgb, 0.05),
  '--iconColor': muted,
  '--codeBlockBgColor': codeBg
})

export const ELEPHANTNOTE_THEME_FAMILIES = [
  {
    id: 'default',
    name: 'Elephant',
    description: 'Neutral notes workspace with crisp blue accents.',
    swatches: ['#2563eb', '#ffffff', '#0f141d'],
    light: 'light',
    dark: 'dark'
  },
  {
    id: 'apple',
    name: 'Apple',
    description: 'Frosted system surfaces, graphite text and iOS blue.',
    swatches: ['#007aff', '#f5f5f7', '#1d1d1f'],
    light: 'apple-light',
    dark: 'apple-dark'
  },
  {
    id: 'graphite',
    name: 'Graphite',
    description: 'Quiet monochrome chrome for dense writing sessions.',
    swatches: ['#6b7280', '#f4f4f5', '#111113'],
    light: 'graphite-light',
    dark: 'graphite-dark'
  },
  {
    id: 'nord',
    name: 'Nord',
    description: 'Cold blue surfaces with cyan focus states.',
    swatches: ['#5e81ac', '#eceff4', '#2e3440'],
    light: 'nord-light',
    dark: 'nord-dark'
  },
  {
    id: 'solar',
    name: 'Solar',
    description: 'Warm reading palette with amber selection and code blocks.',
    swatches: ['#d97706', '#fff7ed', '#1c1917'],
    light: 'solar-light',
    dark: 'solar-dark'
  },
  {
    id: 'forest',
    name: 'Forest',
    description: 'Green editorial palette with soft natural contrast.',
    swatches: ['#15803d', '#f4f7f2', '#101813'],
    light: 'forest-light',
    dark: 'forest-dark'
  },
  {
    id: 'beige',
    name: 'Beige',
    description: 'Paper-like ivory surfaces with warm clay accents.',
    swatches: ['#a45b3f', '#fffaf1', '#211c18', '#e8d7c2'],
    light: 'beige-light',
    dark: 'beige-dark'
  },
  {
    id: 'pastel',
    name: 'Pastel',
    description: 'Soft lavender, peach and mint for a calm workspace.',
    swatches: ['#8b5cf6', '#fbf8ff', '#211b35', '#bdebd4'],
    light: 'pastel-light',
    dark: 'pastel-dark'
  },
  {
    id: 'gamer-violet',
    name: 'Gamer Violet',
    description: 'Deep violet surfaces with neon purple focus states.',
    swatches: ['#a855f7', '#f7f2ff', '#0c0716', '#22d3ee'],
    light: 'gamer-violet-light',
    dark: 'gamer-violet-dark'
  }
]

export const ELEPHANTNOTE_THEME_TOKENS = {
  dark: createThemeTokens({
    bg: '#0f141d', surface: '#141a24', sidebar: '#101722', soft: '#1b2432', softStrong: '#202b3b',
    border: '#283244', borderStrong: '#3a465a', text: '#eef3fb', textRgb: [238, 243, 251],
    muted: '#98a3b6', subtle: '#7f8aa0', primary: '#5ea1ff', primaryRgb: [94, 161, 255],
    danger: '#ff6b7a', shadow: '0 18px 44px rgba(0, 0, 0, 0.28)'
  }),
  light: createThemeTokens({
    bg: '#f7f9fc', surface: '#ffffff', sidebar: '#edf2f7', soft: '#e9eff7', softStrong: '#dfe7f1',
    border: '#c5cfdd', borderStrong: '#aebacd', text: '#101828', textRgb: [16, 24, 40],
    muted: '#475467', subtle: '#667085', primary: '#2563eb', primaryRgb: [37, 99, 235],
    danger: '#dc2626', shadow: '0 30px 90px rgba(15, 23, 42, 0.16)'
  }),
  'apple-dark': createThemeTokens({
    bg: '#1d1d1f', surface: '#2c2c2e', sidebar: '#19191b', soft: '#3a3a3c', softStrong: '#48484a',
    border: '#3f3f46', borderStrong: '#636366', text: '#f5f5f7', textRgb: [245, 245, 247],
    muted: '#a1a1aa', subtle: '#8e8e93', primary: '#0a84ff', primaryRgb: [10, 132, 255],
    danger: '#ff453a', shadow: '0 24px 60px rgba(0, 0, 0, 0.34)', codeBg: '#242426'
  }),
  'apple-light': createThemeTokens({
    bg: '#f5f5f7', surface: '#ffffff', sidebar: '#eeeeef', soft: '#e8e8ed', softStrong: '#d8d8df',
    border: '#d1d1d6', borderStrong: '#aeaeb2', text: '#1d1d1f', textRgb: [29, 29, 31],
    muted: '#515154', subtle: '#6e6e73', primary: '#007aff', primaryRgb: [0, 122, 255],
    danger: '#ff3b30', shadow: '0 28px 80px rgba(60, 60, 67, 0.16)', codeBg: '#f0f0f3'
  }),
  'graphite-dark': createThemeTokens({
    bg: '#111113', surface: '#18181b', sidebar: '#0d0d0f', soft: '#242428', softStrong: '#303036',
    border: '#2f2f35', borderStrong: '#52525b', text: '#f4f4f5', textRgb: [244, 244, 245],
    muted: '#a1a1aa', subtle: '#71717a', primary: '#a1a1aa', primaryRgb: [161, 161, 170],
    danger: '#f87171', shadow: '0 20px 54px rgba(0, 0, 0, 0.36)', codeBg: '#202024'
  }),
  'graphite-light': createThemeTokens({
    bg: '#f4f4f5', surface: '#ffffff', sidebar: '#e4e4e7', soft: '#e7e7ea', softStrong: '#d4d4d8',
    border: '#c9c9cf', borderStrong: '#a1a1aa', text: '#18181b', textRgb: [24, 24, 27],
    muted: '#52525b', subtle: '#71717a', primary: '#52525b', primaryRgb: [82, 82, 91],
    danger: '#dc2626', shadow: '0 22px 70px rgba(39, 39, 42, 0.14)', codeBg: '#ededf0'
  }),
  'nord-dark': createThemeTokens({
    bg: '#2e3440', surface: '#3b4252', sidebar: '#252b35', soft: '#434c5e', softStrong: '#4c566a',
    border: '#4c566a', borderStrong: '#607089', text: '#eceff4', textRgb: [236, 239, 244],
    muted: '#d8dee9', subtle: '#aeb8c8', primary: '#88c0d0', primaryRgb: [136, 192, 208],
    danger: '#bf616a', shadow: '0 22px 58px rgba(16, 22, 31, 0.32)', codeBg: '#343b49'
  }),
  'nord-light': createThemeTokens({
    bg: '#eceff4', surface: '#ffffff', sidebar: '#e5e9f0', soft: '#dfe5ee', softStrong: '#d8dee9',
    border: '#c4ccda', borderStrong: '#9aa8bd', text: '#2e3440', textRgb: [46, 52, 64],
    muted: '#4c566a', subtle: '#667085', primary: '#5e81ac', primaryRgb: [94, 129, 172],
    danger: '#bf616a', shadow: '0 24px 72px rgba(46, 52, 64, 0.15)', codeBg: '#e5e9f0'
  }),
  'solar-dark': createThemeTokens({
    bg: '#1c1917', surface: '#292524', sidebar: '#161311', soft: '#332b26', softStrong: '#40352e',
    border: '#51443b', borderStrong: '#786556', text: '#fef3c7', textRgb: [254, 243, 199],
    muted: '#d6b98c', subtle: '#b99b70', primary: '#f59e0b', primaryRgb: [245, 158, 11],
    danger: '#fb7185', shadow: '0 22px 60px rgba(20, 12, 6, 0.38)', codeBg: '#241f1c'
  }),
  'solar-light': createThemeTokens({
    bg: '#fff7ed', surface: '#fffbf5', sidebar: '#ffedd5', soft: '#fedfbd', softStrong: '#fdba74',
    border: '#f3c99c', borderStrong: '#d69a63', text: '#3b2415', textRgb: [59, 36, 21],
    muted: '#7c4f2d', subtle: '#9a6b43', primary: '#d97706', primaryRgb: [217, 119, 6],
    danger: '#dc2626', shadow: '0 24px 72px rgba(124, 79, 45, 0.16)', codeBg: '#ffedd5'
  }),
  'forest-dark': createThemeTokens({
    bg: '#101813', surface: '#17231b', sidebar: '#0c130f', soft: '#203025', softStrong: '#2a3f31',
    border: '#31513c', borderStrong: '#4b755a', text: '#eef8ef', textRgb: [238, 248, 239],
    muted: '#a8c7ad', subtle: '#85a58c', primary: '#4ade80', primaryRgb: [74, 222, 128],
    danger: '#fb7185', shadow: '0 22px 62px rgba(4, 18, 10, 0.4)', codeBg: '#142019'
  }),
  'forest-light': createThemeTokens({
    bg: '#f4f7f2', surface: '#ffffff', sidebar: '#e7efe2', soft: '#dce8d5', softStrong: '#c8dabc',
    border: '#bdceb4', borderStrong: '#8cab7c', text: '#172016', textRgb: [23, 32, 22],
    muted: '#3f5f3a', subtle: '#5f7d55', primary: '#15803d', primaryRgb: [21, 128, 61],
    danger: '#dc2626', shadow: '0 24px 70px rgba(32, 70, 40, 0.15)', codeBg: '#e7efe2'
  }),
  'beige-light': createThemeTokens({
    bg: '#f5efe4', surface: '#fffaf1', sidebar: '#ebe1d2', soft: '#eadcca', softStrong: '#ddc8ad',
    border: '#d5c2aa', borderStrong: '#b89d7d', text: '#342820', textRgb: [52, 40, 32],
    muted: '#725d4d', subtle: '#8b7461', primary: '#a45b3f', primaryRgb: [164, 91, 63],
    danger: '#c2413b', shadow: '0 24px 72px rgba(84, 58, 38, 0.16)', editorBg: '#fbf4e9', codeBg: '#eee1d0'
  }),
  'beige-dark': createThemeTokens({
    bg: '#211c18', surface: '#2b2520', sidebar: '#191512', soft: '#382f28', softStrong: '#463a31',
    border: '#53463b', borderStrong: '#756252', text: '#f6eadc', textRgb: [246, 234, 220],
    muted: '#c5ad98', subtle: '#a88f7a', primary: '#d8a070', primaryRgb: [216, 160, 112],
    danger: '#f28b82', shadow: '0 22px 64px rgba(10, 6, 3, 0.42)', editorBg: '#241f1b', codeBg: '#302821'
  }),
  'pastel-light': createThemeTokens({
    bg: '#f8f5ff', surface: '#fffefe', sidebar: '#f0ebff', soft: '#eee8fb', softStrong: '#dfd5f4',
    border: '#d8cfea', borderStrong: '#b8a9d4', text: '#302943', textRgb: [48, 41, 67],
    muted: '#6f6484', subtle: '#8b7f9d', primary: '#8b5cf6', primaryRgb: [139, 92, 246],
    danger: '#e56b8f', shadow: '0 24px 76px rgba(93, 70, 135, 0.15)', editorBg: '#fcfaff', codeBg: '#f0eafd'
  }),
  'pastel-dark': createThemeTokens({
    bg: '#211b35', surface: '#2b2442', sidebar: '#181329', soft: '#393050', softStrong: '#493c63',
    border: '#51456b', borderStrong: '#74638f', text: '#f7f0ff', textRgb: [247, 240, 255],
    muted: '#cabee0', subtle: '#a99abb', primary: '#c4b5fd', primaryRgb: [196, 181, 253],
    danger: '#fb8eab', shadow: '0 22px 66px rgba(8, 4, 20, 0.42)', editorBg: '#241e39', codeBg: '#312848'
  }),
  'gamer-violet-light': createThemeTokens({
    bg: '#f7f2ff', surface: '#ffffff', sidebar: '#eee4ff', soft: '#e9ddff', softStrong: '#d7c2ff',
    border: '#ccb5ef', borderStrong: '#a986d4', text: '#26153b', textRgb: [38, 21, 59],
    muted: '#654f7f', subtle: '#806a98', primary: '#9333ea', primaryRgb: [147, 51, 234],
    danger: '#e11d48', shadow: '0 26px 82px rgba(89, 28, 135, 0.2)', editorBg: '#fbf8ff', codeBg: '#ede1ff'
  }),
  'gamer-violet-dark': createThemeTokens({
    bg: '#0c0716', surface: '#160d25', sidebar: '#090510', soft: '#211234', softStrong: '#301748',
    border: '#3a1e52', borderStrong: '#66358b', text: '#f8efff', textRgb: [248, 239, 255],
    muted: '#c4a7d8', subtle: '#9879ad', primary: '#a855f7', primaryRgb: [168, 85, 247],
    danger: '#fb7185', shadow: '0 0 42px rgba(168, 85, 247, 0.18), 0 28px 90px rgba(0, 0, 0, 0.52)',
    editorBg: '#10091c', codeBg: '#1c102c', floatShadow: '0 0 34px rgba(168, 85, 247, 0.24), 0 18px 48px rgba(0, 0, 0, 0.52)'
  })
}

export const ELEPHANTNOTE_THEME_IDS = Object.keys(ELEPHANTNOTE_THEME_TOKENS)

const THEME_META_BY_ID = ELEPHANTNOTE_THEME_FAMILIES.reduce((acc, family) => {
  acc[family.light] = { family, mode: 'light' }
  acc[family.dark] = { family, mode: 'dark' }
  return acc
}, {})

export const normalizeThemeId = (theme) =>
  ELEPHANTNOTE_THEME_IDS.includes(theme) ? theme : ELEPHANTNOTE_DEFAULT_THEME

export const getThemeMode = (theme) => THEME_META_BY_ID[normalizeThemeId(theme)]?.mode || 'light'

export const getThemeFamily = (theme) =>
  THEME_META_BY_ID[normalizeThemeId(theme)]?.family || ELEPHANTNOTE_THEME_FAMILIES[0]

export const getThemeLabel = (theme) => {
  const normalized = normalizeThemeId(theme)
  const meta = THEME_META_BY_ID[normalized]
  if (!meta) return 'Elephant Light'
  return `${meta.family.name} ${meta.mode === 'dark' ? 'Dark' : 'Light'}`
}

export const getThemeVariant = (familyId, mode) => {
  const family =
    ELEPHANTNOTE_THEME_FAMILIES.find((item) => item.id === familyId) ||
    ELEPHANTNOTE_THEME_FAMILIES[0]
  return mode === 'dark' ? family.dark : family.light
}

export const getOppositeThemeVariant = (theme) => {
  const normalized = normalizeThemeId(theme)
  const meta = THEME_META_BY_ID[normalized]
  if (!meta) return ELEPHANTNOTE_DEFAULT_THEME
  return meta.mode === 'dark' ? meta.family.light : meta.family.dark
}

export const getThemeTokens = (theme) => ({
  ...ELEPHANTNOTE_THEME_TOKENS[normalizeThemeId(theme)]
})

export const VAULT_ICON_OPTIONS = [
  { name: 'home', label: 'Home', lucide: 'Home' },
  { name: 'file-text', label: 'Files', lucide: 'FileText', aliases: ['book'] },
  { name: 'database', label: 'Database', lucide: 'Database' },
  { name: 'graduation-cap', label: 'Learning', lucide: 'GraduationCap' },
  { name: 'landmark', label: 'Archive', lucide: 'Landmark' },
  { name: 'rocket', label: 'Project', lucide: 'Rocket' },
  { name: 'star', label: 'Favorite', lucide: 'Star' },
  { name: 'terminal', label: 'Code', lucide: 'Terminal' },
  { name: 'workflow', label: 'Workflow' }
]

export const normalizeVaultIcon = (icon = '') => {
  const normalized = String(icon || '').trim()
  const directMatch = VAULT_ICON_OPTIONS.find((option) => option.name === normalized)
  if (directMatch) return directMatch.name
  const aliasMatch = VAULT_ICON_OPTIONS.find((option) => option.aliases?.includes(normalized))
  return aliasMatch?.name || ''
}

export const isVaultIconName = (icon = '') => normalizeVaultIcon(icon) === icon
