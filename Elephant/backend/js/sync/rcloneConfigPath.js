import path from 'path'

export const getRcloneConfigPath = (vaultPath = '') => path.join(vaultPath, '.elephantnote', 'sync-config.json')
