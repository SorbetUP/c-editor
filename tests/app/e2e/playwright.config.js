const path = require('node:path')

const resultsRoot = path.resolve(__dirname, '../../../test-results')
const reportRoot = path.resolve(__dirname, '../../../playwright-report')
const exhaustiveEvidence = process.env.ELEPHANT_E2E_EXHAUSTIVE_EVIDENCE !== '0'

const config = {
  workers: 1,
  timeout: 60000,
  expect: {
    timeout: 15000
  },
  outputDir: resultsRoot,
  reporter: [
    ['list'],
    ['junit', { outputFile: path.join(resultsRoot, 'e2e-junit.xml') }],
    ['json', { outputFile: path.join(resultsRoot, 'e2e-results.json') }],
    ['html', { outputFolder: reportRoot, open: 'never' }]
  ],
  use: {
    headless: true,
    viewport: { width: 1280, height: 720 },
    actionTimeout: 15000,
    navigationTimeout: 30000,
    screenshot: exhaustiveEvidence ? 'on' : 'only-on-failure',
    trace: exhaustiveEvidence ? 'on' : 'retain-on-failure',
    video: exhaustiveEvidence ? 'on' : 'retain-on-failure'
  }
}
module.exports = config
