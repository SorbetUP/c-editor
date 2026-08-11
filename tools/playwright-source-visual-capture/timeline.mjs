import { createHash } from 'node:crypto'
import { appendFile, mkdir } from 'node:fs/promises'
import path from 'node:path'
import { pngDimensions } from '../freya-differential/lib/common.mjs'

const sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds))

export class SourceCapture {
  constructor (run) {
    this.run = run
    this.events = []
    this.errors = []
  }

  async progress (event, details = {}) {
    await appendFile(path.join(this.run.outputRoot, 'progress.log'), JSON.stringify({ at: new Date().toISOString(), event, ...details }) + '\n')
  }

  async frame (page, action, index, relativeMs) {
    await this.progress('frame:start', { actionId: action.id, index, relativeMs })
    const directory = path.join(this.run.outputRoot, 'frames', action.checkpoint)
    await mkdir(directory, { recursive: true })
    const file = path.join(directory, `frame-${String(index).padStart(3, '0')}-${String(relativeMs).padStart(4, '0')}ms.png`)
    const bytes = await page.screenshot({ path: file, fullPage: false })
    const sourcePath = path.resolve(file)
    const sha256 = createHash('sha256').update(bytes).digest('hex')
    const dimensions = pngDimensions(bytes, sourcePath)
    this.events.push({ event: 'frame:capture', actionId: action.id, checkpoint: action.checkpoint, index, relativeMs, sourcePath, sha256 })
    await this.progress('frame:done', { actionId: action.id, index, relativeMs, sha256 })
    return {
      index,
      relativeMs,
      path: path.relative(this.run.outputRoot, sourcePath),
      sourcePath,
      sha256,
      dimensions
    }
  }

  async runAction (page, action, dispatch) {
    const frameTimes = action.frames || [0]
    const frames = []
    const started = Date.now()
    if (action.id === 'launch') {
      for (const [index, relativeMs] of frameTimes.entries()) {
        await waitUntil(started, relativeMs)
        frames.push(await this.frame(page, action, index, relativeMs))
      }
      return frames
    }
    frames.push(await this.frame(page, action, 0, frameTimes[0]))
    const dispatchPromise = Promise.resolve().then(dispatch)
    dispatchPromise.catch(() => {})
    for (let index = 1; index < frameTimes.length; index += 1) {
      const relativeMs = frameTimes[index]
      await waitUntil(started, relativeMs)
      frames.push(await this.frame(page, action, index, relativeMs))
    }
    await dispatchPromise
    await page.waitForTimeout(250)
    return frames
  }
}

async function waitUntil (started, relativeMs) {
  const remaining = relativeMs - (Date.now() - started)
  if (remaining > 0) await sleep(remaining)
}

export function frameHashesChanged (frames) {
  return new Set(frames.map((frame) => frame.sha256)).size > 1
}
