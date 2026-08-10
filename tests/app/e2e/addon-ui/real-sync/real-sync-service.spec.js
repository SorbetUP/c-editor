const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')
const { test, expect } = require('playwright/test')
const { runRealSyncScenario } = require('./real-sync-service')

test.describe('elephant.sync real native Iroh integration', () => {
  test('pairs two real service processes and transfers vault files in both directions', async ({}, testInfo) => {
    test.setTimeout(180_000)
    const artifactPath = path.join(os.tmpdir(), 'elephant-real-sync-service.json')
    try {
      const result = await runRealSyncScenario({ artifactPath })
      await testInfo.attach('real-sync-service', { path: artifactPath, contentType: 'application/json' })

      expect(result.success).toBe(true)
      expect(result.pairing.folderIdShared).toBe(true)
      expect(result.transfers).toHaveLength(2)
      expect(result.transfers[0].transferredFiles).toBe(2)
      expect(result.transfers[1].transferredFiles).toBe(2)
      expect(result.cleanup.processesExited).toBe(true)
      expect(result.cleanup.vaultsRemoved).toBe(true)
    } catch (error) {
      if (fs.existsSync(artifactPath)) {
        await testInfo.attach('real-sync-service-failure', { path: artifactPath, contentType: 'application/json' })
      }
      throw error
    }
  })
})
