/* @vitest-environment node */

import fs from 'fs-extra'
import path from 'path'

describe('ElephantNote website settings boundary', () => {
  it('keeps website controls out of library cards and inside settings', async() => {
    const folderCard = await fs.readFile(path.resolve('Elephant/frontend/app/components/library/FolderCard.vue'), 'utf8')
    const noteCard = await fs.readFile(path.resolve('Elephant/frontend/app/components/library/NoteCard.vue'), 'utf8')
    const settings = await fs.readFile(path.resolve('Elephant/frontend/src/renderer/src/addons/builtin/ui/SitesSettings.vue'), 'utf8')

    expect(folderCard).not.to.contain('View as website')
    expect(folderCard).not.to.contain('sitePreviewStore')
    expect(noteCard).not.to.contain('View as website')
    expect(noteCard).not.to.contain('Build static website')
    expect(settings).to.contain('sitePreviewStore')
  })
})
