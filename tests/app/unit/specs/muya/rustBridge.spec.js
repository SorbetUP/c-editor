import { describe, expect, it, vi } from 'vitest'

import {
  MuyaRustBridge,
  MuyaRustPatchError,
  createMuyaRustBridge,
  editorCommands
} from '../../../../../Elephant/frontend/src/muya/lib/rust/bridge'

const snapshotResponse = (revision = 0, markdown = '') => ({
  type: 'snapshot',
  payload: {
    markdown,
    revision,
    selection: {
      anchor: { node: 1, offset_utf16: 0 },
      focus: { node: 1, offset_utf16: 0 }
    },
    can_undo: revision > 0,
    can_redo: false,
    composition_active: false
  }
})

const updateResponse = (revision) => ({
  type: 'update',
  payload: {
    revision,
    selection: {
      anchor: { node: 1, offset_utf16: revision },
      focus: { node: 1, offset_utf16: revision }
    },
    patches: [
      {
        type: 'replace_text',
        node: 1,
        range: { start: revision - 1, end: revision - 1 },
        inserted: String(revision)
      }
    ],
    can_undo: true,
    can_redo: false,
    composition_active: false
  }
})

const createEngine = (responses) => {
  const requests = []
  return {
    requests,
    handle_json: vi.fn(async (rawRequest) => {
      requests.push(JSON.parse(rawRequest))
      return JSON.stringify(responses.shift())
    }),
    snapshot_json: vi.fn(async () => JSON.stringify(snapshotResponse()))
  }
}

describe('MuyaRustBridge', () => {
  it('serializes concurrent commands and advances expected revisions', async () => {
    const engine = createEngine([updateResponse(1), updateResponse(2)])
    const applied = []
    const bridge = await createMuyaRustBridge(() => engine, '', {
      applyPatches: async (_patches, update) => {
        await Promise.resolve()
        applied.push(update.revision)
      }
    })

    await Promise.all([
      bridge.dispatch(editorCommands.insertText('a')),
      bridge.dispatch(editorCommands.insertText('b'))
    ])

    expect(engine.requests.map((request) => request.expected_revision)).toEqual([0, 1])
    expect(applied).toEqual([1, 2])
    expect(bridge.revision).toBe(2)
  })

  it('enters a desynchronized state after a patch failure and recovers from a snapshot', async () => {
    const engine = createEngine([updateResponse(1)])
    let failPatch = true
    const snapshots = []
    const bridge = new MuyaRustBridge(engine, {
      applyPatches: () => {
        if (failPatch) throw new Error('DOM rejected patch')
      },
      applySnapshot: (snapshot) => snapshots.push(snapshot.revision)
    })

    await bridge.snapshot()
    await expect(bridge.dispatch(editorCommands.insertText('a'))).rejects.toBeInstanceOf(
      MuyaRustPatchError
    )
    await expect(bridge.dispatch(editorCommands.insertText('b'))).rejects.toBeInstanceOf(
      MuyaRustPatchError
    )

    failPatch = false
    engine.snapshot_json.mockResolvedValueOnce(JSON.stringify(snapshotResponse(1, 'a')))
    await bridge.recover()

    expect(bridge.desynchronized).toBe(false)
    expect(bridge.revision).toBe(1)
    expect(snapshots).toEqual([0, 1])
  })

  it('rejects malformed response envelopes before applying patches', async () => {
    const engine = createEngine([{ type: 'unknown', payload: {} }])
    const bridge = new MuyaRustBridge(engine)

    await expect(bridge.dispatch(editorCommands.undo())).rejects.toBeInstanceOf(TypeError)
    expect(bridge.revision).toBe(0)
  })
})
