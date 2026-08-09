import {
  createApiCaller,
  isElephantNoteApiAvailable,
  requireElephantNoteApi
} from './elephantnoteClient/apiRuntime'
import { requireAtomicFeatureApi } from './elephantnoteClient/atomicFeatureApi'
import { createDomainClients } from './elephantnoteClient/domainClients'
import { COMPATIBILITY_CALLS } from './elephantnoteClient/compatibilityCalls'

export { isElephantNoteApiAvailable }

const call = createApiCaller(COMPATIBILITY_CALLS)

export const elephantnoteClient = {
  describe: () => requireElephantNoteApi().describe(),
  call,
  ...createDomainClients(call, requireAtomicFeatureApi)
}
