import { snapshotFiles } from './fixture.mjs'

export const waitForReadyObservation = async (client, delay) => {
  const selectors = ['.en-library-grid', '.en-empty-card', '.en-no-vault', '.en-shell']
  const deadline = Date.now() + 20000
  let last = null
  while (Date.now() < deadline) {
    for (const selector of selectors) {
      last = await client.command('readDom', selector)
      if (last.exists && last.visible) return { selector, observation: last }
    }
    await delay(100)
  }
  throw new Error(`Tauri ready observation timed out: ${JSON.stringify(last)}`)
}

export const readObservedState = async (client, fixture) => {
  const state = await client.command('readState')
  const selectors = ['.en-shell', '.en-library-grid', '.en-search-overlay', '.en-note-editor-shell', '.en-create-menu-popover']
  const dom = {}
  for (const selector of selectors) dom[selector] = await client.command('readDom', selector)
  return { observed: { state, dom }, fixtureVault: snapshotFiles(fixture.vaultRoot) }
}
