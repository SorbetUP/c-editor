import fs from 'fs-extra'
import os from 'os'
import path from 'path'
import { execFile } from 'child_process'
import { promisify } from 'util'
import { nextAvailableName } from './core'

const execFileAsync = promisify(execFile)

const serializeFrontmatterValue = (value) => {
  if (Array.isArray(value)) {
    return `[${value.map((item) => JSON.stringify(item)).join(', ')}]`
  }
  return JSON.stringify(value)
}

const normalizeText = (value) => {
  return String(value || '')
    .replace(/\r\n/g, '\n')
    .replace(/\r/g, '\n')
    .trim()
}

export const sanitizeFileStem = (value, fallback = 'Imported note') => {
  const base = normalizeText(value) || fallback
  const safe = base
    .normalize('NFKC')
    .replace(/[<>:"/\\|?*]/g, ' ')
    .replace(/\s+/g, ' ')
    .replace(/[. ]+$/g, '')
    .trim()
  return safe || fallback
}

const isGoogleKeepNoteLike = (value) => {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return false
  }

  return Boolean(
    value.title ||
    value.textContent ||
    (Array.isArray(value.listContent) && value.listContent.length > 0) ||
    (Array.isArray(value.attachments) && value.attachments.length > 0)
  )
}

export const parseGoogleKeepTimestamp = (value) => {
  if (value === undefined || value === null) {
    return undefined
  }

  const numeric = typeof value === 'string' ? Number(value) : value
  if (!Number.isFinite(numeric) || numeric <= 0) {
    return undefined
  }

  return new Date(numeric / 1000).toISOString()
}

export const parseGoogleKeepNote = (raw) => {
  if (!isGoogleKeepNoteLike(raw)) {
    return null
  }

  const listContent = Array.isArray(raw.listContent)
    ? raw.listContent
      .map((item) => ({
        text: normalizeText(item?.text),
        isChecked: Boolean(item?.isChecked)
      }))
      .filter((item) => item.text)
    : []

  const attachments = Array.isArray(raw.attachments)
    ? raw.attachments
      .map((attachment) => ({
        filePath: normalizeText(attachment?.filePath),
        mimetype: normalizeText(attachment?.mimetype)
      }))
      .filter((attachment) => attachment.filePath || attachment.mimetype)
    : []

  return {
    title: normalizeText(raw.title),
    textContent: normalizeText(raw.textContent),
    listContent,
    attachments,
    createdAt: parseGoogleKeepTimestamp(raw.createdTimestampUsec),
    updatedAt: parseGoogleKeepTimestamp(raw.userEditedTimestampUsec),
    isChecklist: listContent.length > 0
  }
}

export const buildGoogleKeepMarkdown = (note, fallbackTitle = 'Imported note') => {
  const title = normalizeText(note?.title) || fallbackTitle
  const createdAt = note?.createdAt || new Date().toISOString()
  const updatedAt = note?.updatedAt || createdAt
  const type = note?.isChecklist ? 'task' : 'note'
  const bodyParts = []

  if (note?.textContent) {
    bodyParts.push(note.textContent)
  }

  if (Array.isArray(note?.listContent) && note.listContent.length > 0) {
    bodyParts.push(
      note.listContent
        .map((item) => `- [${item.isChecked ? 'x' : ' '}] ${item.text}`)
        .join('\n')
    )
  }

  if (Array.isArray(note?.attachments) && note.attachments.length > 0) {
    bodyParts.push(
      note.attachments
        .map((attachment) => {
          const attachmentName = path.basename(attachment.filePath || '') || 'attachment'
          return `- Attachment: ${attachmentName}`
        })
        .join('\n')
    )
  }

  const body = bodyParts.filter(Boolean).join('\n\n').trim()

  return [
    '---',
    `title: ${serializeFrontmatterValue(title)}`,
    `type: ${serializeFrontmatterValue(type)}`,
    'tags: []',
    `createdAt: ${serializeFrontmatterValue(createdAt)}`,
    `updatedAt: ${serializeFrontmatterValue(updatedAt)}`,
    '---',
    '',
    `# ${title}`,
    body ? '' : null,
    body
  ].filter((line) => line !== null).join('\n').trimEnd()
}

const collectJsonFiles = async(rootDir) => {
  const jsonFiles = []

  const walk = async(dir) => {
    const entries = await fs.readdir(dir, { withFileTypes: true })
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name)
      if (entry.isDirectory()) {
        await walk(fullPath)
      } else if (entry.isFile() && entry.name.toLowerCase().endsWith('.json')) {
        jsonFiles.push(fullPath)
      }
    }
  }

  await walk(rootDir)
  return jsonFiles
}

const extractZip = async(zipPath) => {
  const tempRoot = await fs.mkdtemp(path.join(os.tmpdir(), 'elephantnote-keep-'))

  try {
    const extractZipModule = await import('extract-zip')
    const extractZip = extractZipModule.default || extractZipModule
    await extractZip(zipPath, { dir: tempRoot })
    return tempRoot
  } catch (_error) {
    try {
      await execFileAsync('unzip', ['-oq', zipPath, '-d', tempRoot])
      return tempRoot
    } catch (unzipError) {
      await fs.remove(tempRoot)
      throw new Error(
        'Unable to extract the Google Keep ZIP export. Unzip the archive first or install a ZIP extractor.'
      )
    }
  }
}

const collectSourceFiles = async(sourcePath) => {
  const fileExt = path.extname(sourcePath).toLowerCase()
  if (fileExt === '.json') {
    return {
      rootDir: path.dirname(sourcePath),
      filePaths: [sourcePath],
      cleanupPath: null
    }
  }

  if (fileExt !== '.zip') {
    throw new Error('Unsupported file type. Please choose a .zip or .json Google Keep export.')
  }

  const extractedRoot = await extractZip(sourcePath)
  return {
    rootDir: extractedRoot,
    filePaths: await collectJsonFiles(extractedRoot),
    cleanupPath: extractedRoot
  }
}

export const importGoogleKeepExport = async({
  sourcePath,
  destinationPath
}) => {
  if (!sourcePath) {
    throw new Error('A Google Keep export file is required.')
  }
  if (!destinationPath) {
    throw new Error('A destination folder is required.')
  }

  const { rootDir, filePaths, cleanupPath } = await collectSourceFiles(sourcePath)
  const parsedNotes = []

  try {
    for (const filePath of filePaths) {
      try {
        const raw = await fs.readJson(filePath)
        const note = parseGoogleKeepNote(raw)
        if (note) {
          parsedNotes.push({ note, filePath })
        }
      } catch (_error) {
        // Ignore malformed files and keep the import moving.
      }
    }

    if (parsedNotes.length === 0) {
      throw new Error('No Google Keep notes were found in the selected export.')
    }

    await fs.ensureDir(destinationPath)

    let imported = 0
    let skipped = 0
    const files = []

    for (let index = 0; index < parsedNotes.length; index += 1) {
      const { note, filePath } = parsedNotes[index]
      const fallbackTitle = path.basename(filePath, path.extname(filePath)) || `Imported note ${index + 1}`
      const markdown = buildGoogleKeepMarkdown(note, fallbackTitle)

      if (!markdown.trim()) {
        skipped += 1
        continue
      }

      const baseName = `${sanitizeFileStem(note.title, fallbackTitle)}.md`
      const targetName = nextAvailableName(
        baseName,
        (candidate) => fs.pathExistsSync(path.join(destinationPath, candidate))
      )
      const targetPath = path.join(destinationPath, targetName)

      await fs.writeFile(targetPath, markdown, 'utf8')
      imported += 1
      files.push(targetPath)
    }

    return {
      imported,
      skipped,
      destinationPath,
      sourceRoot: rootDir,
      files
    }
  } finally {
    if (cleanupPath) {
      await fs.remove(cleanupPath)
    }
  }
}
