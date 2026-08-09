import chokidar from 'chokidar'
import path from 'path'
import { isIgnoredPath, isMarkdownFile } from './pathSafety'

const DEFAULT_DEBOUNCE_MS = 1500

const toRelativePath = (vaultRoot, absolutePath) => {
  return path.relative(vaultRoot, absolutePath).split(path.sep).join('/')
}

export class VaultSearchWatcher {
  constructor({ onAddOrChange, onDelete, debounceMs = DEFAULT_DEBOUNCE_MS } = {}) {
    this._onAddOrChange = onAddOrChange
    this._onDelete = onDelete
    this._debounceMs = debounceMs
    this._watcher = null
    this._timers = new Map()
    this._vaultRoot = ''
  }

  async start(vaultRoot) {
    const root = path.resolve(vaultRoot || '')
    if (!root) {
      throw new Error('A vault root is required.')
    }

    if (this._watcher) {
      await this.stop()
    }

    this._vaultRoot = root
    this._watcher = chokidar.watch(root, {
      ignoreInitial: true,
      persistent: true,
      awaitWriteFinish: {
        stabilityThreshold: this._debounceMs,
        pollInterval: 100
      },
      ignored: (targetPath) => {
        const relativePath = toRelativePath(root, targetPath)
        return isIgnoredPath(relativePath)
      }
    })

    this._watcher
      .on('add', (absolutePath) => {
        this._scheduleUpsert(absolutePath)
      })
      .on('change', (absolutePath) => {
        this._scheduleUpsert(absolutePath)
      })
      .on('unlink', (absolutePath) => {
        this._clearTimer(absolutePath)
        this._onDelete?.(absolutePath)
      })

    return this
  }

  _clearTimer(absolutePath) {
    const timer = this._timers.get(absolutePath)
    if (timer) {
      clearTimeout(timer)
      this._timers.delete(absolutePath)
    }
  }

  _scheduleUpsert(absolutePath) {
    this._clearTimer(absolutePath)
    const timer = setTimeout(() => {
      this._timers.delete(absolutePath)
      const relativePath = toRelativePath(this._vaultRoot, absolutePath)
      if (!isMarkdownFile(absolutePath) || isIgnoredPath(relativePath)) {
        return
      }
      this._onAddOrChange?.(absolutePath)
    }, this._debounceMs)
    this._timers.set(absolutePath, timer)
  }

  async stop() {
    for (const timer of this._timers.values()) {
      clearTimeout(timer)
    }
    this._timers.clear()

    if (this._watcher) {
      await this._watcher.close()
      this._watcher = null
    }
  }
}
