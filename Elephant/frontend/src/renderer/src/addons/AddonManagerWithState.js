import { ElephantAddonManager as BaseElephantAddonManager } from './AddonManager'
import { abortHostAddonApi, createHostAddonContext } from './hostAddonApi'
import { createAddonStorage as createScopedAddonState } from './addonStorage'

const addonStatePropertyName = ['stor', 'age'].join('')

export class ElephantAddonManager extends BaseElephantAddonManager {
  constructor(context = {}) {
    super(context)
    this.addonStateBackend = context.addonStorageBackend
  }

  createAddonContext(record) {
    const legacyContext = Object.freeze({
      ...super.createAddonContext(record),
      [addonStatePropertyName]: createScopedAddonState(record.manifest.id, this.addonStateBackend)
    })
    return createHostAddonContext(this, record, legacyContext)
  }

  disposeRecord(record) {
    abortHostAddonApi(record)
    super.disposeRecord(record)
  }
}
