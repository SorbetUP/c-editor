/* eslint-disable @stylistic/object-property-newline */
import ISO6391 from 'iso-639-1'
import { additionalAppLocales } from './additionalAppLocales'

export const APP_LANGUAGE_STORAGE_KEY = 'elephantnote:tauri:language'
export const APP_DEFAULT_LOCALE = 'en'

const english = {
  common: {
    cancel: 'Cancel',
    close: 'Close',
    save: 'Save',
    saving: 'Saving…',
    search: 'Search',
    settings: 'Settings',
    remove: 'Remove',
    add: 'Add',
    words: 'words',
    characters: 'characters',
    localFirst: 'Local-first'
  },
  navigation: {
    appearance: 'Appearance',
    editor: 'Editor',
    vaults: 'Vaults',
    sync: 'Sync',
    ai: 'AI',
    sites: 'Sites',
    import: 'Import'
  },
  settings: {
    searchPlaceholder: 'Search all settings',
    searchEmptyTitle: 'No setting found',
    searchEmptyDescription: 'Try another word, feature name or control.',
    results: '{count} result | {count} results',
    language: 'Language',
    languageDescription: 'Use any ISO language supported by Elephant or an installed translation pack.',
    systemLanguage: 'System language',
    colorMode: 'Color mode',
    colorModeDescription: 'Use the light or dark variant of the selected theme.',
    light: 'Light',
    dark: 'Dark',
    theme: 'Theme',
    themeDescription: 'Choose the visual family used throughout Elephant.',
    sidebarWidth: 'Sidebar width',
    sidebarWidthDescription: 'Resize the main navigation rail.'
  },
  note: {
    untitled: 'Untitled',
    titleLabel: 'Note title',
    addTag: 'Add tag',
    pin: 'Pin note',
    unpin: 'Unpin note',
    close: 'Close note',
    graph: 'Graph',
    openGraph: 'Open graph',
    toggleTheme: 'Toggle theme',
    focusMode: 'Focus mode',
    findInNotes: 'Find in notes'
  },
  excalidraw: {
    title: 'Excalidraw',
    drawingName: 'Drawing name',
    drawingPlaceholder: 'drawing',
    save: 'Save drawing',
    cancel: 'Cancel drawing',
    saving: 'Saving drawing…',
    failedTitle: 'Excalidraw failed to open.',
    failedSave: 'The drawing could not be saved.',
    failedInitialize: 'The drawing canvas could not be initialized.',
    localBadge: 'Saved in this vault',
    hint: 'Draw freely. The editable scene and PNG preview are stored together.'
  },
  search: {
    open: 'Search notes',
    shortcutHint: 'Search notes with {shortcut}'
  }
}

const french = {
  common: { cancel: 'Annuler', close: 'Fermer', save: 'Enregistrer', saving: 'Enregistrement…', search: 'Rechercher', settings: 'Paramètres', remove: 'Retirer', add: 'Ajouter', words: 'mots', characters: 'caractères', localFirst: 'Local d’abord' },
  navigation: { appearance: 'Apparence', editor: 'Éditeur', vaults: 'Coffres', sync: 'Synchronisation', ai: 'IA', sites: 'Sites', import: 'Importer' },
  settings: { searchPlaceholder: 'Rechercher dans tous les paramètres', searchEmptyTitle: 'Aucun paramètre trouvé', searchEmptyDescription: 'Essayez un autre mot, une fonctionnalité ou un contrôle.', results: '{count} résultat | {count} résultats', language: 'Langue', languageDescription: 'Utilisez toute langue ISO prise en charge par Elephant ou par un pack de traduction installé.', systemLanguage: 'Langue du système', colorMode: 'Mode de couleur', colorModeDescription: 'Utiliser la variante claire ou sombre du thème sélectionné.', light: 'Clair', dark: 'Sombre', theme: 'Thème', themeDescription: 'Choisissez la famille visuelle utilisée dans Elephant.', sidebarWidth: 'Largeur de la barre latérale', sidebarWidthDescription: 'Redimensionner la navigation principale.' },
  note: { untitled: 'Sans titre', titleLabel: 'Titre de la note', addTag: 'Ajouter une étiquette', pin: 'Épingler la note', unpin: 'Désépingler la note', close: 'Fermer la note', graph: 'Graphe', openGraph: 'Ouvrir le graphe', toggleTheme: 'Changer de thème', focusMode: 'Mode concentration', findInNotes: 'Rechercher dans les notes' },
  excalidraw: { title: 'Excalidraw', drawingName: 'Nom du dessin', drawingPlaceholder: 'dessin', save: 'Enregistrer le dessin', cancel: 'Annuler le dessin', saving: 'Enregistrement du dessin…', failedTitle: 'Impossible d’ouvrir Excalidraw.', failedSave: 'Le dessin n’a pas pu être enregistré.', failedInitialize: 'La zone de dessin n’a pas pu être initialisée.', localBadge: 'Enregistré dans ce coffre', hint: 'Dessinez librement. La scène modifiable et l’aperçu PNG sont enregistrés ensemble.' },
  search: { open: 'Rechercher dans les notes', shortcutHint: 'Rechercher dans les notes avec {shortcut}' }
}

const spanish = {
  common: { cancel: 'Cancelar', close: 'Cerrar', save: 'Guardar', saving: 'Guardando…', search: 'Buscar', settings: 'Ajustes', remove: 'Quitar', add: 'Añadir', words: 'palabras', characters: 'caracteres', localFirst: 'Local primero' },
  navigation: { appearance: 'Apariencia', editor: 'Editor', vaults: 'Bóvedas', sync: 'Sincronización', ai: 'IA', sites: 'Sitios', import: 'Importar' },
  settings: { searchPlaceholder: 'Buscar en todos los ajustes', searchEmptyTitle: 'No se encontró ningún ajuste', searchEmptyDescription: 'Prueba otra palabra o función.', results: '{count} resultado | {count} resultados', language: 'Idioma', languageDescription: 'Usa cualquier idioma ISO compatible o un paquete de traducción instalado.', systemLanguage: 'Idioma del sistema', colorMode: 'Modo de color', colorModeDescription: 'Usa la variante clara u oscura del tema.', light: 'Claro', dark: 'Oscuro', theme: 'Tema', themeDescription: 'Elige la familia visual de Elephant.', sidebarWidth: 'Ancho de la barra lateral', sidebarWidthDescription: 'Redimensiona la navegación principal.' },
  note: { untitled: 'Sin título', titleLabel: 'Título de la nota', addTag: 'Añadir etiqueta', pin: 'Fijar nota', unpin: 'Desfijar nota', close: 'Cerrar nota', graph: 'Grafo', openGraph: 'Abrir grafo', toggleTheme: 'Cambiar tema', focusMode: 'Modo enfoque', findInNotes: 'Buscar en notas' },
  excalidraw: { title: 'Excalidraw', drawingName: 'Nombre del dibujo', drawingPlaceholder: 'dibujo', save: 'Guardar dibujo', cancel: 'Cancelar dibujo', saving: 'Guardando dibujo…', failedTitle: 'Excalidraw no pudo abrirse.', failedSave: 'No se pudo guardar el dibujo.', failedInitialize: 'No se pudo iniciar el lienzo.', localBadge: 'Guardado en esta bóveda', hint: 'Dibuja libremente. La escena editable y la vista PNG se guardan juntas.' },
  search: { open: 'Buscar notas', shortcutHint: 'Buscar notas con {shortcut}' }
}

const german = {
  common: { cancel: 'Abbrechen', close: 'Schließen', save: 'Speichern', saving: 'Speichern…', search: 'Suchen', settings: 'Einstellungen', remove: 'Entfernen', add: 'Hinzufügen', words: 'Wörter', characters: 'Zeichen', localFirst: 'Lokal zuerst' },
  navigation: { appearance: 'Darstellung', editor: 'Editor', vaults: 'Tresore', sync: 'Synchronisierung', ai: 'KI', sites: 'Websites', import: 'Import' },
  settings: { searchPlaceholder: 'Alle Einstellungen durchsuchen', searchEmptyTitle: 'Keine Einstellung gefunden', searchEmptyDescription: 'Versuche ein anderes Wort oder eine Funktion.', results: '{count} Ergebnis | {count} Ergebnisse', language: 'Sprache', languageDescription: 'Verwende jede unterstützte ISO-Sprache oder ein installiertes Übersetzungspaket.', systemLanguage: 'Systemsprache', colorMode: 'Farbmodus', colorModeDescription: 'Helle oder dunkle Variante des Themes verwenden.', light: 'Hell', dark: 'Dunkel', theme: 'Theme', themeDescription: 'Wähle die visuelle Familie von Elephant.', sidebarWidth: 'Breite der Seitenleiste', sidebarWidthDescription: 'Hauptnavigation skalieren.' },
  note: { untitled: 'Ohne Titel', titleLabel: 'Notiztitel', addTag: 'Tag hinzufügen', pin: 'Notiz anheften', unpin: 'Notiz lösen', close: 'Notiz schließen', graph: 'Graph', openGraph: 'Graph öffnen', toggleTheme: 'Theme wechseln', focusMode: 'Fokusmodus', findInNotes: 'Notizen durchsuchen' },
  excalidraw: { title: 'Excalidraw', drawingName: 'Zeichnungsname', drawingPlaceholder: 'zeichnung', save: 'Zeichnung speichern', cancel: 'Zeichnung abbrechen', saving: 'Zeichnung wird gespeichert…', failedTitle: 'Excalidraw konnte nicht geöffnet werden.', failedSave: 'Die Zeichnung konnte nicht gespeichert werden.', failedInitialize: 'Die Zeichenfläche konnte nicht initialisiert werden.', localBadge: 'In diesem Tresor gespeichert', hint: 'Zeichne frei. Bearbeitbare Szene und PNG-Vorschau werden zusammen gespeichert.' },
  search: { open: 'Notizen durchsuchen', shortcutHint: 'Notizen mit {shortcut} durchsuchen' }
}

const builtInMessages = {
  en: english,
  fr: french,
  es: spanish,
  de: german,
  ...additionalAppLocales
}

const isPlainObject = (value) => Object.prototype.toString.call(value) === '[object Object]'

export const mergeMessages = (base = {}, override = {}) => {
  const output = { ...base }
  for (const [key, value] of Object.entries(override || {})) {
    output[key] = isPlainObject(value) && isPlainObject(output[key])
      ? mergeMessages(output[key], value)
      : value
  }
  return output
}

export const normalizeAppLocale = (locale = '') => {
  const raw = String(locale || '').trim().replaceAll('_', '-')
  if (!raw) return APP_DEFAULT_LOCALE
  try {
    const parsed = new Intl.Locale(raw)
    const language = parsed.language.toLowerCase()
    if (language === 'zh') {
      const region = String(parsed.region || '').toUpperCase()
      const script = String(parsed.script || '')
      return region === 'TW' || region === 'HK' || region === 'MO' || script === 'Hant' ? 'zh-TW' : 'zh-CN'
    }
    return language
  } catch {
    return raw.toLowerCase().split('-')[0] || APP_DEFAULT_LOCALE
  }
}

export const isRtlLocale = (locale = '') => ['ar', 'fa', 'he', 'ur', 'ps', 'sd', 'ug', 'yi'].includes(normalizeAppLocale(locale))

export const getAppMessages = (locale = APP_DEFAULT_LOCALE) => {
  const normalized = normalizeAppLocale(locale)
  return mergeMessages(english, builtInMessages[normalized] || {})
}

export const getSupportedLanguageOptions = (displayLocale = APP_DEFAULT_LOCALE) => {
  const displayNames = typeof Intl.DisplayNames === 'function'
    ? new Intl.DisplayNames([normalizeAppLocale(displayLocale)], { type: 'language' })
    : null
  const browserLocale = normalizeAppLocale(globalThis.navigator?.language || APP_DEFAULT_LOCALE)
  const codes = ISO6391.getAllCodes()
  const options = codes.map((code) => ({
    code,
    nativeName: ISO6391.getNativeName(code) || ISO6391.getName(code) || code,
    displayName: displayNames?.of(code) || ISO6391.getName(code) || code,
    hasBuiltInAppMessages: Boolean(builtInMessages[normalizeAppLocale(code)])
  }))
  options.sort((a, b) => a.displayName.localeCompare(b.displayName, normalizeAppLocale(displayLocale)))
  return [
    { code: 'system', nativeName: 'System', displayName: 'System', resolvedLocale: browserLocale, hasBuiltInAppMessages: true },
    ...options
  ]
}

export const resolveStoredLocale = (value = '') => {
  if (!value || value === 'system') return normalizeAppLocale(globalThis.navigator?.language || APP_DEFAULT_LOCALE)
  return normalizeAppLocale(value)
}
