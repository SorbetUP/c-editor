import { describe, expect, it } from 'vitest'
import {
  ATOMIC_AI_FEATURES,
  ATOMIC_MODEL_CATALOG,
  ATOMIC_PLUGIN_MANIFESTS,
  PROGRAMMATIC_TASK_TEMPLATES,
  createDefaultPluginState,
  createDefaultTaskState,
  createDefaultModelSelection,
  getModelsByPurpose,
  mergePluginState,
  mergeTaskState,
  normalizePluginManifest,
  normalizeProgrammaticTask,
  updatePluginState,
  updateTaskState
} from 'common/elephantnote/atomicWorkspace'

describe('Atomic workspace catalog', () => {
  it('tracks the Atomic AI capability surface as a shared manifest', () => {
    expect(ATOMIC_AI_FEATURES.map((feature) => feature.id)).toEqual(expect.arrayContaining([
      'embedding-search',
      'semantic-links',
      'source-extraction',
      'auto-tagging',
      'cited-rag-chat',
      'wiki-synthesis',
      'local-model-runtimes',
      'mcp-tools'
    ]))
  })

  it('groups built-in models by purpose', () => {
    expect(getModelsByPurpose('embedding').map((model) => model.id)).toContain('smollm2-node-llama-cpp')
    expect(getModelsByPurpose('chat').map((model) => model.id)).toContain('smollm2-node-llama-cpp-chat')
    expect(getModelsByPurpose('ocr').map((model) => model.id)).toContain('local-tesseract-ocr')
    expect(getModelsByPurpose('speech-to-text').map((model) => model.id)).toContain('whisper-tiny-browser')
    expect(getModelsByPurpose('unknown')).toEqual([])
  })

  it('creates a default model selection for every production model slot', () => {
    expect(createDefaultModelSelection(ATOMIC_MODEL_CATALOG)).toEqual({
      embedding: '',
      tagging: '',
      naming: '',
      wiki: '',
      summary: '',
      chat: '',
      agent: '',
      ocr: '',
      'speech-to-text': '',
      'text-to-speech': ''
    })
  })

  it('normalizes plugin manifests with explicit permissions and surfaces', () => {
    const manifest = normalizePluginManifest({
      id: 'google-calendar',
      name: 'Google Calendar',
      status: 'enabled',
      runtime: 'calendar.google.sync',
      permissions: ['calendar:read', null, 'calendar:write'],
      surfaces: ['settings', 'calendar']
    })

    expect(manifest).toEqual({
      id: 'google-calendar',
      name: 'Google Calendar',
      status: 'enabled',
      runtime: 'calendar.google.sync',
      permissions: ['calendar:read', 'calendar:write'],
      surfaces: ['settings', 'calendar']
    })
    expect(ATOMIC_PLUGIN_MANIFESTS.map((plugin) => plugin.id)).toContain('mcp-memory')
  })

  it('merges persistent plugin state into plugin manifests', () => {
    const defaults = createDefaultPluginState(ATOMIC_PLUGIN_MANIFESTS)
    expect(defaults['google-calendar']).toEqual({ enabled: false, config: {} })

    const state = updatePluginState(ATOMIC_PLUGIN_MANIFESTS, defaults, {
      id: 'google-calendar',
      enabled: true,
      config: { calendarId: 'primary' }
    })
    const plugins = mergePluginState(ATOMIC_PLUGIN_MANIFESTS, state)

    expect(plugins.find((plugin) => plugin.id === 'google-calendar')).toMatchObject({
      enabled: true,
      status: 'enabled',
      config: { calendarId: 'primary' }
    })
    expect(() => updatePluginState(ATOMIC_PLUGIN_MANIFESTS, state, { id: 'missing' }))
      .toThrow('Unknown plugin.')
  })

  it('normalizes task templates into executable task manifests', () => {
    expect(normalizeProgrammaticTask({ template: 'daily-briefing' })).toMatchObject({
      id: 'daily-briefing',
      name: 'Daily briefing',
      description: 'Summarize recent vault activity and suggest wiki updates.',
      cadence: 'daily',
      enabled: true,
      prompt: 'Create a short daily briefing from recent notes and calendar context.',
      actions: ['search:recent', 'wiki:propose', 'calendar:summary']
    })
  })

  it('merges persistent task state into task templates', () => {
    const defaults = createDefaultTaskState(PROGRAMMATIC_TASK_TEMPLATES)
    expect(defaults['daily-briefing']).toMatchObject({ enabled: false, lastRunAt: '' })

    const state = updateTaskState(PROGRAMMATIC_TASK_TEMPLATES, defaults, {
      id: 'daily-briefing',
      enabled: true,
      lastRunAt: '2026-05-25T06:00:00.000Z',
      lastResult: { ok: true }
    })
    const tasks = mergeTaskState(PROGRAMMATIC_TASK_TEMPLATES, state)

    expect(tasks.find((task) => task.id === 'daily-briefing')).toMatchObject({
      enabled: true,
      lastRunAt: '2026-05-25T06:00:00.000Z',
      lastResult: { ok: true }
    })
    expect(updateTaskState(PROGRAMMATIC_TASK_TEMPLATES, state, {
      id: 'custom-review',
      name: 'Custom review',
      enabled: true
    })['custom-review']).toMatchObject({
      id: 'custom-review',
      name: 'Custom review',
      enabled: true
    })
  })
})
