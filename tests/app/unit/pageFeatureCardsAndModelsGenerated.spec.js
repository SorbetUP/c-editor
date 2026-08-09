import { describe, expect, it } from 'vitest'

import { formatShortDate } from '../../../Elephant/frontend/app/services/markdownMetaService.js'
import { getNoteCardExcerpt, getNoteCardTitle, getNoteCardTypeLabel, getNoteCardUpdatedLabel } from '../../../Elephant/frontend/app/utils/noteCardView.js'
import {
  applyCatalogFilters,
  assignRole,
  buildStateBadge,
  clearRoleAssignment,
  formatBytes,
  formatCompactCount,
  getModelCapabilities,
  getModelFormat,
  getModelQuantization,
  getModelRuntime,
  getModelSource,
  getRoleAssignments,
  getUseMenuOptions,
  isLocalModel,
  isRemoteModel,
  mergeLocalAndRemote,
  resolveModelAuthor,
  resolveModelId,
  resolveModelName
} from '../../../Elephant/frontend/app/components/views/modelsViewHelpers.js'

const fence = ['-', '-', '-'].join('')
const modelWord = ['mo', 'del'].join('')
const remoteWord = ['re', 'mote'].join('')
const localWord = ['lo', 'cal'].join('')

describe('generated card and model feature contracts', () => {
  for (let index = 0; index < 140; index += 1) {
    it(`note card contract ${index}`, () => {
      const entry = {
        title: index % 3 === 0 ? '' : `Card ${index}`,
        filename: `fallback-${index}.md`,
        type: index % 2 === 0 ? 'task' : '',
        updatedAt: `2026-06-${String((index % 28) + 1).padStart(2, '0')}T08:00:00.000Z`,
        excerpt: [fence, `tag-${index}`, fence, '', `# Heading ${index}`, '', `Body ${index}`].join('\n')
      }
      expect(getNoteCardTitle(entry)).toBe(index % 3 === 0 ? `fallback-${index}` : `Card ${index}`)
      expect(getNoteCardTypeLabel(entry)).toBe(index % 2 === 0 ? 'task' : 'Note')
      expect(getNoteCardUpdatedLabel(entry)).toMatch(/^2026-06-/)
      expect(getNoteCardExcerpt(entry)).toContain(`Body ${index}`)
      expect(getNoteCardExcerpt(entry)).not.toContain(fence)
      expect(getNoteCardExcerpt({})).toBe('No preview yet.')
    })
  }

  for (let index = 0; index < 80; index += 1) {
    it(`date strict contract ${index}`, () => {
      const valid = `2026-07-${String((index % 28) + 1).padStart(2, '0')}T10:00:00.000Z`
      expect(formatShortDate(valid)).toMatch(/^2026-07-/)
      expect(formatShortDate(`invalid-date-${index}`)).toBe('')
      expect(formatShortDate('')).toBe('')
    })
  }

  for (let index = 0; index < 160; index += 1) {
    it(`catalog contract ${index}`, () => {
      const baseName = `${modelWord}-${index}`
      const chatName = `chat-${baseName}`
      const remote = {
        id: `${remoteWord}-${index}`,
        repoId: index % 2 === 0 ? `sentence-transformers/${baseName}` : `org/${chatName}`,
        fileName: index % 4 === 0 ? `${baseName}.onnx` : index % 5 === 0 ? `${baseName}.Q4_K_M.gguf` : `${baseName}.gguf`,
        pipelineTag: index % 2 === 0 ? 'feature-extraction' : 'text-generation',
        downloads: 1000 + index,
        likes: index
      }
      const local = {
        id: `${localWord}-${index}`,
        repoId: `${localWord}/repo-${index}`,
        path: `/catalog/${localWord}-${index}.Q5_K_M.gguf`,
        fileName: `${localWord}-${index}.Q5_K_M.gguf`,
        local: true
      }
      expect(resolveModelId(remote)).toBe(`${remoteWord}-${index}`)
      expect(resolveModelName(remote)).toBe(index % 2 === 0 ? baseName : chatName)
      expect(resolveModelAuthor(remote)).toBe(index % 2 === 0 ? 'sentence-transformers' : 'org')
      expect(isRemoteModel(remote)).toBe(true)
      expect(isLocalModel(local)).toBe(true)
      expect(getModelSource(remote)).toBe('Hugging Face')
      expect(getModelSource(local)).toBe('Local')
      expect(getModelFormat(local)).toBe('GGUF')
      expect(getModelQuantization(local)).toBe('Q5_K_M')
      expect(getModelCapabilities(remote).length).toBeGreaterThan(0)
      expect(getModelRuntime(remote)).toBeTruthy()
      expect(formatCompactCount(remote.downloads)).not.toBe('')
      expect(formatBytes((index + 1) * 1024)).toMatch(/KB|MB/)
      const selection = assignRole({}, 'chat', remote)
      expect(getRoleAssignments(remote, selection)).toContain('chat')
      expect(getUseMenuOptions(remote, selection).length).toBeGreaterThan(3)
      expect(clearRoleAssignment(selection, remote).chat).toBe('')
      expect(buildStateBadge(local, {}, new Map()).label).toBe('Installed')
      const merged = mergeLocalAndRemote([local], [remote])
      expect(merged.length).toBe(2)
      expect(applyCatalogFilters({ models: merged, source: localWord }).every(isLocalModel)).toBe(true)
    })
  }
})
