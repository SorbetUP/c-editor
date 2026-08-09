import { describe, expect, it } from 'vitest'
import {
  ATOMIC_PLUGIN_MANIFESTS,
  EXTENSION_PLUGIN_RUNTIMES,
  EXTENSION_TASK_ACTIONS,
  PROGRAMMATIC_TASK_TEMPLATES,
  createTaskRunResult,
  createTaskStepResult,
  isExecutableTaskAction,
  normalizePluginManifest,
  resolvePluginRuntime
} from 'common/elephantnote/extensions'

describe('ElephantNote extension contract', () => {
  it('keeps plugin manifests portable while naming their runtime adapter', () => {
    expect(ATOMIC_PLUGIN_MANIFESTS.map((plugin) => [plugin.id, plugin.runtime])).toEqual([
      ['google-calendar', EXTENSION_PLUGIN_RUNTIMES.GOOGLE_CALENDAR_SYNC],
      ['mcp-memory', EXTENSION_PLUGIN_RUNTIMES.MCP_TOOL_CALL],
      ['web-clipper', EXTENSION_PLUGIN_RUNTIMES.SOURCE_INGEST_URL]
    ])
    expect(normalizePluginManifest({
      id: 'mcp-memory',
      name: 'MCP Memory',
      runtime: EXTENSION_PLUGIN_RUNTIMES.MCP_TOOL_CALL,
      permissions: ['notes:read', null],
      surfaces: ['agents']
    })).toMatchObject({
      id: 'mcp-memory',
      runtime: EXTENSION_PLUGIN_RUNTIMES.MCP_TOOL_CALL,
      permissions: ['notes:read'],
      surfaces: ['agents']
    })
  })

  it('resolves compatibility plugin ids to adapter routes', () => {
    expect(resolvePluginRuntime({ id: 'google-calendar' }))
      .toBe(EXTENSION_PLUGIN_RUNTIMES.GOOGLE_CALENDAR_SYNC)
    expect(resolvePluginRuntime({ id: 'web-clipper' }))
      .toBe(EXTENSION_PLUGIN_RUNTIMES.SOURCE_INGEST_URL)
    expect(resolvePluginRuntime({ id: 'mcp-memory' }))
      .toBe(EXTENSION_PLUGIN_RUNTIMES.MCP_TOOL_CALL)
    expect(resolvePluginRuntime({ id: 'custom', runtime: 'custom.runtime' }))
      .toBe('custom.runtime')
  })

  it('defines task actions separately from task execution results', () => {
    expect(PROGRAMMATIC_TASK_TEMPLATES.find((task) => task.id === 'daily-briefing').actions).toEqual([
      EXTENSION_TASK_ACTIONS.SEARCH_RECENT,
      EXTENSION_TASK_ACTIONS.WIKI_PROPOSE,
      EXTENSION_TASK_ACTIONS.CALENDAR_SUMMARY
    ])
    expect(isExecutableTaskAction(EXTENSION_TASK_ACTIONS.SEARCH_RECENT)).toBe(true)
    expect(isExecutableTaskAction(EXTENSION_TASK_ACTIONS.MODEL_TAGGING)).toBe(false)

    const result = createTaskRunResult([
      createTaskStepResult({ action: 'search:recent', ok: true, summary: '2 recent notes' }),
      createTaskStepResult({ action: 'model:tagging', ok: false, summary: 'Not executable yet' })
    ])

    expect(result).toEqual({
      ok: false,
      steps: [
        { action: 'search:recent', ok: true, summary: '2 recent notes' },
        { action: 'model:tagging', ok: false, summary: 'Not executable yet' }
      ]
    })
  })
})
