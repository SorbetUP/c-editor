import vm from 'node:vm'
import { describe, expect, it } from 'vitest'

import { createIsolatedAddonWorkerSource } from '../../../../Elephant/frontend/src/renderer/src/addons/isolatedAddonWorkerSource.js'

const boot = async () => {
  const messages = []
  const self = { postMessage: (message) => messages.push(message) }
  const source = createIsolatedAddonWorkerSource(`
self.elephantAddon = {
  activate(api) { self.__api = api; }
};
`, 'com.example.replace', {
    permissions: { commands: true, notes: { read: ['Inbox/**'], write: [] } }
  })
  vm.runInContext(source, vm.createContext({
    self,
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
  }))
  await self.onmessage({ data: { type: 'activate', id: 1 } })
  return { messages, self }
}

describe('isolated addon replacement safety', () => {
  it('deep-freezes manifest capabilities exposed to addon code', async () => {
    const { self } = await boot()

    expect(Object.isFrozen(self.__api.capabilities)).toBe(true)
    expect(Object.isFrozen(self.__api.capabilities.permissions)).toBe(true)
    expect(Object.isFrozen(self.__api.capabilities.permissions.notes)).toBe(true)
    expect(Object.isFrozen(self.__api.capabilities.permissions.notes.read)).toBe(true)
  })

  it('does not let an old disposer unregister a newer command with the same id', async () => {
    const { messages, self } = await boot()
    const id = 'com.example.replace.run'
    const first = self.__api.commands.register({ id, title: 'First', run: () => 1 })
    const second = self.__api.commands.register({ id, title: 'Second', run: () => 2 })
    const unregisterCount = () => messages.filter((message) => (
      message.type === 'unregister-action' && message.id === id
    )).length

    first()
    expect(unregisterCount()).toBe(0)

    await self.onmessage({
      data: { type: 'run-command', id: 2, commandId: id, payload: null }
    })
    expect(messages).toContainEqual(expect.objectContaining({
      type: 'command-result',
      id: 2,
      ok: true,
      result: 2
    }))

    second()
    expect(unregisterCount()).toBe(1)
  })
})
