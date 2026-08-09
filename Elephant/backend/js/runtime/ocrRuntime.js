import fs from 'fs-extra'
import { execFile as execFileCallback } from 'node:child_process'
import { promisify } from 'node:util'

const defaultExecFile = promisify(execFileCallback)

export class OcrRuntime {
  constructor({
    execFile = defaultExecFile,
    candidates = [
      process.env.TESSERACT_PATH,
      'tesseract',
      '/opt/homebrew/bin/tesseract',
      '/usr/local/bin/tesseract'
    ]
  } = {}) {
    this.execFile = execFile
    this.candidates = candidates.filter(Boolean)
    this.binaryPath = ''
  }

  async status() {
    try {
      const binaryPath = await this.resolveBinary()
      return {
        provider: 'local-ocr',
        engine: 'tesseract',
        available: true,
        binaryPath,
        message: 'Tesseract OCR is available.'
      }
    } catch (error) {
      return {
        provider: 'local-ocr',
        engine: 'tesseract',
        available: false,
        binaryPath: '',
        message: error instanceof Error ? error.message : 'Tesseract OCR is not available.'
      }
    }
  }

  async extractImageText({ imagePath, language = 'eng', pageSegmentationMode = '6' } = {}) {
    const normalizedPath = String(imagePath || '').trim()
    if (!normalizedPath) throw new Error('Image path is required for OCR.')
    if (!(await fs.pathExists(normalizedPath))) {
      throw new Error(`Image not found: ${normalizedPath}`)
    }

    const binaryPath = await this.resolveBinary()
    const args = [
      normalizedPath,
      'stdout',
      '-l',
      String(language || 'eng'),
      '--psm',
      String(pageSegmentationMode || '6')
    ]
    const { stdout = '', stderr = '' } = await this.execFile(binaryPath, args, {
      maxBuffer: 10 * 1024 * 1024
    })
    return {
      provider: 'local-ocr',
      engine: 'tesseract',
      binaryPath,
      imagePath: normalizedPath,
      language: String(language || 'eng'),
      text: String(stdout || '').trim(),
      raw: stdout,
      warning: String(stderr || '').trim()
    }
  }

  async resolveBinary() {
    if (this.binaryPath) return this.binaryPath

    const errors = []
    for (const candidate of this.candidates) {
      try {
        await this.execFile(candidate, ['--version'], { maxBuffer: 1024 * 1024 })
        this.binaryPath = candidate
        return candidate
      } catch (error) {
        errors.push(error instanceof Error ? error.message : String(error))
      }
    }

    throw new Error(`Tesseract OCR was not found. Tried: ${this.candidates.join(', ')}. ${errors[0] || ''}`.trim())
  }
}
