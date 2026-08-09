import { RcloneVaultEngine } from '../sync/rcloneVaultEngine.js'
import { ModelRuntime } from '../modelRuntime'
import { ProgramRuntime } from '../programRuntime'

export const syncEngine = new RcloneVaultEngine()
export const modelRuntime = new ModelRuntime()
export const programRuntime = new ProgramRuntime()
