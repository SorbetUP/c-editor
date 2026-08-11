import path from 'node:path'

import {
  expectedActions,
  expectedCheckpoints,
  loadScenario,
  materializeFixture
} from '../../tools/freya-differential/lib/scenario.mjs'

export const DEFAULT_SHARED_SCENARIO = path.resolve(
  import.meta.dirname,
  '../../migration/freya/differential-scenarios.json'
)

export const loadSharedScenario = async (filename = DEFAULT_SHARED_SCENARIO) => {
  const scenario = await loadScenario(filename)
  return {
    scenario,
    actions: expectedActions(scenario),
    checkpoints: expectedCheckpoints(scenario)
  }
}

export const materializeSharedFixture = (scenario, fixtureRoot) => materializeFixture(scenario, fixtureRoot)
