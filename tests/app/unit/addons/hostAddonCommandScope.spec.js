import { describe, expect, it } from 'vitest'

import { ElephantAddonManager } from '@/addons'

const manifest = (id) => ({
  id,
  name: id,
  version: '1.0.0',
  apiVersion: 1,
  source: 'builtin',
  defaultEnabled: false
})

describe('host addon command scoping', () => {
  it('executes the command owned by the calling addon when legacy ids collide', async () => {
    let callerContext
    const manager = new ElephantAddonManager()

    manager.register({
      manifest: manifest('com.example.first'),
      activate(context) {
        context.addAction({ id: 'shared.action', title: 'First', run: () => 'first' })
      }
    })
    manager.register({
      manifest: manifest('com.example.caller'),
      activate(context) {
        callerContext = context
        context.api.commands.register({ id: 'shared.action', title: 'Caller', run: () => 'caller' })
      }
    })

    await manager.enable('com.example.first')
    await manager.enable('com.example.caller')

    expect(await manager.runAction('shared.action')).toBe('first')
    expect(await callerContext.api.commands.execute('shared.action')).toBe('caller')
    expect(callerContext.api.commands.get('shared.action')).toMatchObject({
      addonId: 'com.example.caller'
    })
  })

  it('passes the same execution metadata as the historical manager API', async () => {
    let callerContext
    let receivedMetadata
    const manager = new ElephantAddonManager()

    manager.register({
      manifest: manifest('com.example.metadata'),
      activate(context) {
        callerContext = context
        context.api.commands.register({
          id: 'metadata.action',
          title: 'Metadata',
          run(payload, metadata) {
            receivedMetadata = metadata
            return payload
          }
        })
      }
    })

    await manager.enable('com.example.metadata')
    await expect(callerContext.api.commands.execute('metadata.action', { ok: true }))
      .resolves.toEqual({ ok: true })
    expect(receivedMetadata).toMatchObject({
      addonId: 'com.example.metadata',
      actionId: 'metadata.action',
      addons: manager
    })
  })
})
