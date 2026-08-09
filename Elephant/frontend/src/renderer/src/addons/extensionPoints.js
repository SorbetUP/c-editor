export const ADDON_EXTENSION_POINTS = Object.freeze({
  actions: 'actions',
  sidebarItems: 'sidebar.items',
  settingsSections: 'settings.sections',
  settingsPages: 'settings.pages',
  views: 'views',
  workspacePanels: 'workspace.panels',
  editorExtensions: 'editor.extensions',
  editorBlockTypes: 'editor.block-types',
  editorInlineTypes: 'editor.inline-types',
  editorInputRules: 'editor.input-rules',
  editorToolbarItems: 'editor.toolbar-items',
  editorFooterItems: 'editor.footer-items',
  editorPasteHandlers: 'editor.paste-handlers',
  markdownPostProcessors: 'markdown.post-processors',
  markdownCodeBlockProcessors: 'markdown.code-block-processors',
  markdownEmbedRenderers: 'markdown.embed-renderers',
  layoutItems: 'layout.items',
  layoutZones: 'layout.zones',
  aiProviders: 'ai.providers',
  importers: 'importers',
  siteGenerators: 'site.generators',
  statusBarItems: 'statusbar.items'
})

export const isKnownExtensionPoint = (area) => {
  return Object.values(ADDON_EXTENSION_POINTS).includes(area)
}
