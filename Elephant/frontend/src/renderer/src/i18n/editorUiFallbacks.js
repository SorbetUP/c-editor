import { i18n } from './index'

export const EDITOR_UI_ENGLISH_FALLBACKS = Object.freeze({
  common: {
    cancel: 'Cancel',
    ok: 'OK'
  },
  editor: {
    emptyMermaidBlock: 'Empty Mermaid block',
    insertColumnLeft: 'Insert column left',
    insertColumnRight: 'Insert column right',
    insertRowAbove: 'Insert row above',
    insertRowBelow: 'Insert row below',
    insertTable: {
      title: 'Insert table',
      rows: 'Rows',
      columns: 'Columns'
    },
    removeColumn: 'Remove column',
    removeRow: 'Remove row'
  },
  frontMenu: {
    paragraph: 'Paragraph'
  },
  table: {
    alignCenter: 'Align center',
    alignLeft: 'Align left',
    alignRight: 'Align right',
    deleteTable: 'Delete table',
    resizeTable: 'Resize table'
  }
})

export const installEditorUiI18nFallbacks = () => {
  const globalI18n = i18n?.global
  if (typeof globalI18n?.mergeLocaleMessage !== 'function') return false
  globalI18n.mergeLocaleMessage('en', EDITOR_UI_ENGLISH_FALLBACKS)
  return true
}

installEditorUiI18nFallbacks()
