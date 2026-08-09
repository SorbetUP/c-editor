export const RCLONE_SYNC_COMMAND = 'bisync'

export const buildRcloneFilterRules = () => [
  '- .git/**',
  '- .DS_Store',
  '- Thumbs.db',
  '- desktop.ini',
  '- ~$*',
  '- ~*.tmp',
  '- .~*',
  '- *.swp',
  '- *.tmp',
  '- .elephantnote/cache/**',
  '- .elephantnote/logs/**',
  '- .elephantnote/search/**',
  '- .elephantnote/sync/**',
  '- .elephantnote/sync-config.json',
  '- .elephantnote/sync-log.json',
  '- .elephantnote/sync-local.json',
  '+ **'
]

export const buildBisyncArgs = ({ localPath, remotePath, resync = true, filtersFile = '' } = {}) => {
  const args = [
    RCLONE_SYNC_COMMAND,
    localPath,
    remotePath,
    '--check-access',
    '--create-empty-src-dirs',
    '--conflict-resolve',
    'none',
    '--conflict-suffix',
    'elephant-conflict'
  ]
  if (filtersFile) args.push('--filters-file', filtersFile)
  if (resync) args.push('--resync')
  return args
}
