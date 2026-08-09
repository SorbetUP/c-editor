import { ELEPHANTNOTE_API_ACTIONS } from '../api'

export const registerCompatibilityElephantNoteIpc = ({ ipcMain, api }) => {
  ipcMain.handle('elephantnote:getVaults', async() => api.call(ELEPHANTNOTE_API_ACTIONS.VAULTS_GET))
  ipcMain.handle('elephantnote:selectVault', async(event) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.VAULTS_SELECT, {}, { event }))
  ipcMain.handle('elephantnote:setActiveVault', async(_event, vaultId) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.VAULTS_SET_ACTIVE, { vaultId }))
  ipcMain.handle('elephantnote:setVaultIcon', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.VAULTS_SET_ICON, payload))
  ipcMain.handle('elephantnote:setVaultName', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.VAULTS_SET_NAME, payload))
  ipcMain.handle('elephantnote:removeVault', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.VAULTS_REMOVE, payload))
  ipcMain.handle('elephantnote:listDirectory', async(_event, relativePath = '') =>
    api.call(ELEPHANTNOTE_API_ACTIONS.DIRECTORY_LIST, { relativePath }))
  ipcMain.handle('elephantnote:createNote', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.NOTES_CREATE, payload))
  ipcMain.handle('elephantnote:createFolder', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.FOLDERS_CREATE, payload))
  ipcMain.handle('elephantnote:attachSidebarEntry', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.SIDEBAR_ATTACH, payload))
  ipcMain.handle('elephantnote:detachSidebarEntry', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.SIDEBAR_DETACH, payload))
  ipcMain.handle('elephantnote:importGoogleKeep', async(event) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.IMPORT_GOOGLE_KEEP, {}, { event }))
  ipcMain.handle('elephantnote:renameEntry', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.ENTRIES_RENAME, payload))
  ipcMain.handle('elephantnote:moveEntry', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.ENTRIES_MOVE, payload))
  ipcMain.handle('elephantnote:deleteEntry', async(_event, payload) =>
    api.call(ELEPHANTNOTE_API_ACTIONS.ENTRIES_DELETE, payload))
}
