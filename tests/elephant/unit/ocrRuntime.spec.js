/* @vitest-environment node */

import fs from 'fs-extra'
import os from 'node:os'
import path from 'node:path'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { OcrRuntime } from 'main_renderer/elephantnote/runtime/ocrRuntime'

describe('OcrRuntime', () => {
  const tempFiles = []

  afterEach(async() => {
    await Promise.all(tempFiles.map((file) => fs.remove(file)))
    tempFiles.length = 0
  })

  it('reports Tesseract availability', async() => {
    const execFile = vi.fn(async() => ({ stdout: 'tesseract 5.5.1\n', stderr: '' }))
    const runtime = new OcrRuntime({ execFile, candidates: ['tesseract'] })

    await expect(runtime.status()).resolves.toMatchObject({
      provider: 'local-ocr',
      engine: 'tesseract',
      available: true,
      binaryPath: 'tesseract'
    })
  })

  it('extracts image text with the selected language and segmentation mode', async() => {
    const imagePath = path.join(os.tmpdir(), `elephantnote-ocr-${Date.now()}.png`)
    tempFiles.push(imagePath)
    await fs.writeFile(imagePath, 'fake image bytes')
    const execFile = vi.fn(async(command, args) => {
      if (args[0] === '--version') return { stdout: 'tesseract 5.5.1\n', stderr: '' }
      return { stdout: 'ElephantNote Settings\nSearch Chat OCR\n', stderr: 'warning\n' }
    })
    const runtime = new OcrRuntime({ execFile, candidates: ['tesseract'] })

    await expect(runtime.extractImageText({
      imagePath,
      language: 'eng',
      pageSegmentationMode: '11'
    })).resolves.toMatchObject({
      provider: 'local-ocr',
      engine: 'tesseract',
      imagePath,
      text: 'ElephantNote Settings\nSearch Chat OCR',
      warning: 'warning'
    })
    expect(execFile).toHaveBeenLastCalledWith('tesseract', [
      imagePath,
      'stdout',
      '-l',
      'eng',
      '--psm',
      '11'
    ], expect.objectContaining({ maxBuffer: 10 * 1024 * 1024 }))
  })

  it('fails clearly when the image path is missing', async() => {
    const runtime = new OcrRuntime({
      execFile: vi.fn(async() => ({ stdout: 'tesseract 5.5.1\n', stderr: '' })),
      candidates: ['tesseract']
    })

    await expect(runtime.extractImageText({ imagePath: '/tmp/does-not-exist.png' }))
      .rejects.toThrow('Image not found')
  })
})
