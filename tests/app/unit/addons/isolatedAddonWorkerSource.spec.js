import vm from 'node:vm'
import { describe, expect, it } from 'vitest'

import { createIsolatedAddonWorkerSource } from '../../../../Elephant/frontend/src/renderer/src/addons/isolatedAddonWorkerSource.js'

const createRuntime = () => {
  const entry = `
self.elephantAddon = {
  activate(api) {
    self.__api = api;
    self.__aborted = false;
    api.lifecycle.onAbort(() => { self.__aborted = true; });
    api.commands.register({
      id: api.ids.qualify('run'),
      title: 'Run',
      run: async (payload) => payload.value + 1
    });
    api.views.register({
      id: api.ids.qualify('view'),
      kind: 'task-manager-v1',
      getState: async () => ({ ok: true }),
      dispatch: async () => ({ ok: true })
    });
    api.events.on('ping', (payload) => { self.__ping = payload; });
    api.log.info('activated');
    return { dispose() { self.__disposed = true; } };
  },
  deactivate(api) { api.log.info('deactivating'); }
};
`
  const source = createIsolatedAddonWorkerSource(entry, 'com.example.test', {
    permissions: { commands: true, storage: true },
    contributes: { views: [{ id: 'com.example.test.view', kind: 'task-manager-v1' }] }
  })
  const messages = []
  const self = { postMessage: (message) => messages.push(message) }
  const context = vm.createContext({
    self,
    console,
    setTimeout,
    clearTimeout,
    setInterval,
    clearInterval,
    AbortController,
    structuredClone,
    Object,
    Map,
    Set,
    Promise,
    Error,
    TypeError,
    String,
    Number,
    JSON,
    Array
  })
  vm.runInContext(source, context)
  return { messages, self, source }
}

const findRpc = (messages, method) => messages.findLast((message) => (
  message.type === 'rpc' && message.method === method
))

describe('isolated addon Worker API', () => {
  it('adds capabilities while keeping the original API surface', async () => {
    const { messages, self } = createRuntime()

    await self.onmessage({ data: { type: 'activate', id: 1 } })

    expect(self.__api.app.info).toBeTypeOf('function')
    expect(self.__api.notes.list).toBeTypeOf('function')
    expect(self.__api.notes.read).toBeTypeOf('function')
    expect(self.__api.notes.write).toBeTypeOf('function')
    expect(self.__api.http.request).toBeTypeOf('function')
    expect(self.__api.storage.entries).toBeTypeOf('function')
    expect(self.__api.commands.register).toBeTypeOf('function')
    expect(self.__api.views.register).toBeTypeOf('function')

    expect(self.__api.capabilities.runtime).toBe('isolated-worker')
    expect(self.__api.app.capabilities()).toBe(self.__api.capabilities)
    expect(self.__api.ids.qualify('run')).toBe('com.example.test.run')
    expect(self.__api.ids.owns('com.example.test.run')).toBe(true)
    expect(messages).toContainEqual(expect.objectContaining({ type: 'register-action' }))
    expect(messages).toContainEqual(expect.objectContaining({ type: 'register-view' }))
    expect(messages).toContainEqual(expect.objectContaining({ type: 'log', level: 'info' }))

    self.__api.events.emit('ping', { ok: true })
    expect(self.__ping).toEqual({ ok: true })

    const keysPromise = self.__api.storage.keys()
    const storageRpc = findRpc(messages, 'storage.entries')
    await self.onmessage({
      data: { type: 'rpc-result', id: storageRpc.id, ok: true, result: { z: 1, a: 2 } }
    })
    expect(await keysPromise).toEqual(['a', 'z'])
  })

  it('updates notes through read and overwrite-safe write RPCs', async () => {
    const { messages, self } = createRuntime()
    await self.onmessage({ data: { type: 'activate', id: 1 } })

    const updatePromise = self.__api.notes.update(
      'Inbox/Test.md',
      (markdown, document) => `${markdown}\n${document.path}`
    )
    const readRpc = findRpc(messages, 'notes.read')
    await self.onmessage({
      data: {
        type: 'rpc-result',
        id: readRpc.id,
        ok: true,
        result: { path: 'Inbox/Test.md', markdown: '# Before' }
      }
    })
    await Promise.resolve()

    const writeRpc = findRpc(messages, 'notes.write')
    expect(writeRpc.params).toMatchObject({
      path: 'Inbox/Test.md',
      content: '# Before\nInbox/Test.md',
      markdown: '# Before\nInbox/Test.md',
      overwrite: true
    })
    await self.onmessage({
      data: {
        type: 'rpc-result',
        id: writeRpc.id,
        ok: true,
        result: { ok: true, path: 'Inbox/Test.md' }
      }
    })

    await expect(updatePromise).resolves.toMatchObject({
      ok: true,
      path: 'Inbox/Test.md',
      markdown: '# Before\nInbox/Test.md',
      content: '# Before\nInbox/Test.md'
    })
  })

  it('executes registered commands and cleans every dynamic resource on deactivation', async () => {
    const { messages, self } = createRuntime()
    await self.onmessage({ data: { type: 'activate', id: 1 } })

    await self.onmessage({
      data: {
        type: 'run-command',
        id: 2,
        commandId: 'com.example.test.run',
        payload: { value: 4 }
      }
    })
    expect(messages).toContainEqual(expect.objectContaining({
      type: 'command-result',
      id: 2,
      ok: true,
      result: 5
    }))

    await self.onmessage({ data: { type: 'deactivate', id: 3 } })

    expect(self.__aborted).toBe(true)
    expect(self.__disposed).toBe(true)
    expect(messages).toContainEqual(expect.objectContaining({ type: 'unregister-action' }))
    expect(messages).toContainEqual(expect.objectContaining({ type: 'unregister-view' }))
    expect(messages).toContainEqual(expect.objectContaining({
      type: 'deactivation-result',
      id: 3,
      ok: true
    }))
  })
})
