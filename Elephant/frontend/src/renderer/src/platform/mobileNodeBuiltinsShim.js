const desktopOnly = (name) => {
  throw new Error(`${name} is unavailable in the Android renderer. Use the Tauri bridge instead.`)
}

const fnv1a = (value = '') => {
  let hash = 0x811c9dc5
  const text = typeof value === 'string' ? value : String(value ?? '')
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index)
    hash = Math.imul(hash, 0x01000193)
  }
  return (hash >>> 0).toString(16).padStart(8, '0')
}

export const createHash = () => {
  let buffered = ''
  return {
    update(value) {
      buffered += typeof value === 'string' ? value : String(value ?? '')
      return this
    },
    digest(encoding = 'hex') {
      const hex = fnv1a(buffered).repeat(5).slice(0, 40)
      if (encoding === 'hex') return hex
      return hex
    }
  }
}

export const spawn = () => desktopOnly('child_process.spawn')
export const exec = () => desktopOnly('child_process.exec')
export const execFile = () => desktopOnly('child_process.execFile')
export const fork = () => desktopOnly('child_process.fork')

export const constants = Object.freeze({ F_OK: 0, R_OK: 4, W_OK: 2, X_OK: 1 })

export const statSync = () => desktopOnly('fs.statSync')
export const readFileSync = () => desktopOnly('fs.readFileSync')
export const writeFileSync = () => desktopOnly('fs.writeFileSync')
export const existsSync = () => false
export const accessSync = () => desktopOnly('fs.accessSync')
export const mkdirSync = () => desktopOnly('fs.mkdirSync')
export const unlinkSync = () => desktopOnly('fs.unlinkSync')
export const createReadStream = () => desktopOnly('fs.createReadStream')
export const createWriteStream = () => desktopOnly('fs.createWriteStream')

const rejected = (name) => async () => desktopOnly(name)
export const readFile = rejected('fs/promises.readFile')
export const writeFile = rejected('fs/promises.writeFile')
export const access = rejected('fs/promises.access')
export const mkdir = rejected('fs/promises.mkdir')
export const stat = rejected('fs/promises.stat')
export const unlink = rejected('fs/promises.unlink')
export const rename = rejected('fs/promises.rename')
export const copyFile = rejected('fs/promises.copyFile')

export const promises = { readFile, writeFile, access, mkdir, stat, unlink, rename, copyFile }

export const tmpdir = () => '/tmp'
export const homedir = () => '/'
export const platform = () => 'android'

export const deflateSync = () => desktopOnly('zlib.deflateSync')
export const gzipSync = () => desktopOnly('zlib.gzipSync')
export const inflateSync = () => desktopOnly('zlib.inflateSync')

export default {
  access,
  accessSync,
  constants,
  copyFile,
  createHash,
  createReadStream,
  createWriteStream,
  deflateSync,
  exec,
  execFile,
  existsSync,
  fork,
  gzipSync,
  homedir,
  inflateSync,
  mkdir,
  mkdirSync,
  platform,
  promises,
  readFile,
  readFileSync,
  rename,
  spawn,
  stat,
  statSync,
  tmpdir,
  unlink,
  unlinkSync,
  writeFile,
  writeFileSync
}
