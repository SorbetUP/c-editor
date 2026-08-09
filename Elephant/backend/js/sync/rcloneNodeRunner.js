import { execFile } from 'child_process'
import { promisify } from 'util'

const execFileAsync = promisify(execFile)

export const createRcloneExecutor = () => async(binary, args, options = {}) => execFileAsync(binary, args, {
  cwd: options.cwd || process.cwd(),
  timeout: options.timeout || 600000,
  maxBuffer: 64 * 1024 * 1024
})
