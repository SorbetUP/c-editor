import {
  ATOMIC_MODEL_CATALOG,
  createDefaultModelSelection
} from 'common/elephantnote/atomicWorkspace'
import { AI_SETUP_RECOMMENDED_IDS, isRunnableSetupModel } from 'common/elephantnote/aiSetup'

export const MODEL_ROLES = Object.freeze([
  { id: 'embedding', label: 'Embedding', hint: 'Semantic search and note graph retrieval' },
  { id: 'chat', label: 'Chat', hint: 'Assistant, RAG chat and agent bridging' },
  { id: 'ocr', label: 'OCR', hint: 'Extract text from images and scans' }
])

export const ROLE_IDS = Object.freeze(MODEL_ROLES.map((role) => role.id))
export const USE_NONE = 'none'

const isDateLikeString = (value) => /^\d{4}-\d{2}-\d{2}(?:T|$)/.test(String(value || ''))
const getRepoParts = (model = {}) => String(model?.repoId || model?.modelId || model?.id || '')
  .trim()
  .split('/')
  .filter(Boolean)
const getSiblingName = (item = {}) => String(item?.rfilename || item?.path || item?.name || '').trim()
const getSiblingSizeBytes = (item = {}) => Number(item?.sizeBytes || item?.size || item?.lfs?.size || item?.blob?.size || 0) || 0
const getGgufSiblings = (model = {}) => Array.isArray(model?.siblings)
  ? model.siblings
    .map((item) => ({ ...item, fileName: getSiblingName(item), sizeBytes: getSiblingSizeBytes(item) }))
    .filter((item) => item.fileName.toLowerCase().endsWith('.gguf'))
  : []
const getPreferredGgufSibling = (model = {}) => {
  const siblings = getGgufSiblings(model)
  return siblings.find((item) => /q4_k_m/i.test(item.fileName)) || siblings.find((item) => /q4/i.test(item.fileName)) || siblings[0] || null
}
const normalizeDateValue = (value) => {
  if (value == null || value === '') return null
  if (typeof value === 'number') return new Date(value > 10_000_000_000 ? value : value * 1000)
  const text = String(value).trim()
  if (/^\d+$/.test(text)) {
    const numeric = Number(text)
    return new Date(numeric > 10_000_000_000 ? numeric : numeric * 1000)
  }
  if (!isDateLikeString(text)) return null
  return new Date(text)
}

export const formatBytes = (value) => {
  let bytes = Number(value) || 0
  if (!bytes) return ''
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let index = 0
  while (bytes >= 1024 && index < units.length - 1) {
    bytes /= 1024
    index += 1
  }
  const decimals = index === 0 || bytes >= 100 || Math.abs(bytes - Math.round(bytes)) < 0.05 ? 0 : 1
  return `${bytes.toFixed(decimals)} ${units[index]}`
}

export const formatCompactCount = (value) => {
  const count = Number(value) || 0
  if (count >= 1e6) return `${(count / 1e6).toFixed(count >= 1e7 ? 0 : 1)}M`
  if (count >= 1e3) return `${(count / 1e3).toFixed(count >= 1e4 ? 0 : 1)}K`
  return `${count}`
}

export const resolveModelId = (model) =>
  String(model?.id || model?.repoId || model?.modelId || model?.modelPath || model?.path || model?.name || '').trim()

export const resolveModelName = (model) => {
  const raw = String(model?.name || model?.id || model?.repoId || model?.modelId || 'Untitled model').trim()
  const repoParts = getRepoParts(model)
  if (repoParts.length >= 2 && (raw === repoParts.join('/') || raw === model?.repoId || raw === model?.id)) {
    return repoParts.slice(1).join('/')
  }
  return raw.replace(/^.*[/\\]([^/\\]+)$/u, '$1')
}

export const resolveModelAuthor = (model) => {
  const author = String(model?.author || '').trim()
  if (author) return author
  const repoParts = getRepoParts(model)
  return repoParts.length >= 2 ? repoParts[0] : ''
}

export const isLocalModel = (model) =>
  model?.provider === 'local-ocr' || Boolean(model?.path || model?.modelPath || model?.local)

export const isRemoteModel = (model) =>
  !isLocalModel(model) &&
  Boolean(model?.provider === 'huggingface' || model?.repoId || model?.source === 'huggingface' || model?.pull || model?.uri)

export const isRunnableModel = (model) => isRunnableSetupModel(model)

export const isDownloading = (model = {}, downloads = new Map()) => {
  const id = resolveModelId(model)
  const state = id ? downloads.get(id) : null
  if (!state) return false
  const percent = Number(state.percent || 0)
  return percent > 0 && percent < 100
}

export const downloadProgress = (model = {}, downloads = new Map()) =>
  Number(downloads.get(resolveModelId(model))?.percent || 0)

export const downloadMessage = (model = {}, downloads = new Map()) =>
  String(downloads.get(resolveModelId(model))?.message || '')

export const getRoleAssignments = (model = {}, selection = {}) => {
  const id = resolveModelId(model)
  return id ? ROLE_IDS.filter((role) => selection[role] === id) : []
}

export const isAssignedToRole = (model = {}, role = '', selection = {}) =>
  Boolean(resolveModelId(model) && role && selection[role] === resolveModelId(model))

export const assignRole = (selection = {}, role = '', model = {}) =>
  ROLE_IDS.includes(role) && resolveModelId(model)
    ? { ...selection, [role]: resolveModelId(model) }
    : { ...selection }

export const clearRoleAssignment = (selection = {}, model = {}) => {
  const id = resolveModelId(model)
  const next = { ...selection }
  for (const role of ROLE_IDS) {
    if (next[role] === id) next[role] = ''
  }
  return next
}

export const clearSpecificRole = (selection = {}, role = '') =>
  ROLE_IDS.includes(role) ? { ...selection, [role]: '' } : { ...selection }

export const applyRoleChoice = (selection = {}, role = '', model = {}, choice = '') =>
  choice === USE_NONE ? clearRoleAssignment(selection, model) : assignRole(selection, role, model)

export const countAssignedRoles = (selection) =>
  ROLE_IDS.reduce((count, role) => count + (selection?.[role] ? 1 : 0), 0)

export const getAssignedModelForRole = (role = '', catalog = [], selection = {}) =>
  catalog.find((model) => resolveModelId(model) === selection[role]) || null

export const normalizeSelection = (selection) => ({
  ...createDefaultModelSelection(),
  ...(selection && typeof selection === 'object' ? selection : {})
})

export const getModelSummary = (model) =>
  String(model?.summary || model?.notes || model?.pipelineTag || model?.message || model?.repoId || 'No description available.').trim()

export const getModelSizeLabel = (model) => {
  const sibling = getPreferredGgufSibling(model)
  return formatBytes(model?.sizeBytes || model?.size || sibling?.sizeBytes) || String(model?.size || '').trim()
}

export const getModelTags = (model) =>
  Array.from(
    new Set([
      model?.pipelineTag,
      model?.purpose && model.purpose !== 'chat' ? model.purpose : '',
      ...(Array.isArray(model?.tags) ? model.tags : [])
    ].filter(Boolean))
  ).slice(0, 4)

export const filterModelsByName = (models = [], query = '') => {
  const needle = String(query).trim().toLowerCase()
  if (!needle) return [...models]
  return models.filter((model) =>
    [
      model.name,
      model.id,
      model.repoId,
      model.modelId,
      model.author,
      model.pipelineTag,
      model.fileName,
      model.filename,
      ...(Array.isArray(model.siblings) ? model.siblings.map(getSiblingName) : []),
      ...(Array.isArray(model.tags) ? model.tags : [])
    ]
      .filter(Boolean)
      .join(' ')
      .toLowerCase()
      .includes(needle)
  )
}

const dedupeKey = (model) =>
  String(model?.repoId || model?.originalRepoId || model?.id || model?.modelId || model?.name || model?.path || model?.modelPath || '').trim().toLowerCase()

export const dedupeModelsById = (models = []) =>
  Array.from(
    models.reduce((map, model) => {
      const key = dedupeKey(model)
      if (!key) return map
      const previous = map.get(key)
      if (
        !previous ||
        (isLocalModel(model) && !isLocalModel(previous)) ||
        Number(model?.downloads || 0) > Number(previous?.downloads || 0)
      ) {
        map.set(key, model)
      }
      return map
    }, new Map()).values()
  )

export const sortByPopularity = (models = []) =>
  dedupeModelsById(models).sort(
    (a, b) =>
      Number(b.downloads || 0) - Number(a.downloads || 0) ||
      Number(b.likes || 0) - Number(a.likes || 0) ||
      resolveModelName(a).localeCompare(resolveModelName(b))
  )

export const mergeLocalAndRemote = (localModels = [], remoteModels = []) =>
  dedupeModelsById([...(Array.isArray(localModels) ? localModels : []), ...(Array.isArray(remoteModels) ? remoteModels : [])])

export const getPopularModels = ({ catalog = ATOMIC_MODEL_CATALOG, remote = [], limit = 12 } = {}) =>
  sortByPopularity(mergeLocalAndRemote(catalog.filter(isRunnableModel), remote)).slice(0, Number(limit) || 12)

export const getRecommendedModelId = (role = '') => (AI_SETUP_RECOMMENDED_IDS['tauri-rust'] || AI_SETUP_RECOMMENDED_IDS['node-llama-cpp'] || {})[role] || ''

export const getRecommendedModel = (role = '', catalog = ATOMIC_MODEL_CATALOG) =>
  catalog.find((model) => model.id === getRecommendedModelId(role)) || null

export const buildStateBadge = (model = {}, selection = {}, downloads = new Map()) => {
  if (isDownloading(model, downloads)) {
    return { tone: 'downloading', label: 'Downloading' }
  }
  const assignments = getRoleAssignments(model, selection)
  if (assignments.length) {
    return {
      tone: 'active',
      label: `Active · ${assignments.join(', ')}`
    }
  }
  if (isLocalModel(model)) return { tone: 'installed', label: 'Installed' }
  return { tone: 'available', label: 'Available' }
}

export const createInitialSelection = () => normalizeSelection({})

export const getUseMenuOptions = (model = {}, selection = {}) => [
  ...MODEL_ROLES.map((role) => ({
    ...role,
    selected: getRoleAssignments(model, selection).includes(role.id),
    recommended: getRecommendedModelId(role.id) === resolveModelId(model)
  })),
  {
    id: USE_NONE,
    label: 'None',
    hint: getRoleAssignments(model, selection).length ? 'Unassign from every role' : 'Not in use',
    selected: getRoleAssignments(model, selection).length === 0
  }
]

export const FORMAT_FILTERS = Object.freeze([
  { id: 'all', label: 'All' },
  { id: 'gguf', label: 'GGUF' },
  { id: 'mlx', label: 'MLX' },
  { id: 'onnx', label: 'ONNX' }
])

export const SOURCE_FILTERS = Object.freeze([
  { id: 'all', label: 'All' },
  { id: 'installed', label: 'Installed' },
  { id: 'local', label: 'Local' },
  { id: 'remote', label: 'Remote' }
])

export const SORT_OPTIONS = Object.freeze([
  { id: 'best', label: 'Best Match' },
  { id: 'downloads', label: 'Downloads' },
  { id: 'updated', label: 'Recently Updated' },
  { id: 'likes', label: 'Likes' },
  { id: 'name', label: 'Name' }
])

export const getModelFormat = (model) => {
  const sibling = getPreferredGgufSibling(model)
  const name = String(model?.fileName || model?.filename || sibling?.fileName || model?.name || model?.id || '').toLowerCase()
  if (name.includes('mlx')) return 'MLX'
  if (name.includes('onnx')) return 'ONNX'
  if (name.includes('gguf') || sibling) return 'GGUF'
  if (model?.provider === 'local-ocr' || model?.task === 'ocr') return 'OCR'
  return 'GGUF'
}

export const getModelQuantization = (model) => {
  const sibling = getPreferredGgufSibling(model)
  const name = String(model?.fileName || model?.filename || sibling?.fileName || model?.name || model?.id || '')
  const quantization = name.match(/q([0-9]+(?:_[a-z0-9_]+)*)/i)
  if (quantization) return `Q${quantization[1].toUpperCase()}`
  if (model?.dtype) return String(model.dtype).toUpperCase()
  if (/f16|fp16/i.test(name)) return 'F16'
  if (/q8/i.test(name)) return 'Q8'
  return ''
}

export const getDownloadOption = (model = {}) => {
  const sibling = getPreferredGgufSibling(model)
  const fileName = String(model?.fileName || model?.filename || sibling?.fileName || model?.name || resolveModelName(model) || '').trim()
  const sizeBytes = Number(model?.sizeBytes || model?.size || model?.lfs?.size || sibling?.sizeBytes || 0) || 0
  const formatModel = { ...model, fileName, filename: fileName }
  const installed = isLocalModel(model)
  return {
    fileName: fileName || 'Unknown model file',
    format: getModelFormat(formatModel),
    installed,
    status: installed ? 'Applicable model file already downloaded' : 'Available for download',
    quantization: getModelQuantization(formatModel),
    sizeBytes,
    sizeLabel: formatBytes(sizeBytes)
  }
}

export const getModelCapabilities = (model = {}) => {
  const caps = new Set()
  const purpose = String(model?.purpose || '').toLowerCase()
  const task = String(model?.task || '').toLowerCase()
  const pipeline = String(model?.pipelineTag || '').toLowerCase()
  const name = [model?.repoId, model?.id, model?.modelId, model?.name, model?.fileName, model?.filename, getPreferredGgufSibling(model)?.fileName]
    .filter(Boolean)
    .join(' ')
    .toLowerCase()
  const tags = Array.isArray(model?.tags) ? model.tags.map((tag) => String(tag).toLowerCase()) : []
  const all = [name, pipeline, purpose, task, ...tags].join(' ')

  if (purpose === 'ocr' || task.includes('ocr')) caps.add('OCR')
  if (purpose === 'agent' || task.includes('agent') || pipeline.includes('agent') || tags.includes('agent')) caps.add('Agent')
  if (tags.some((tag) => ['vision', 'vlm', 'image'].includes(tag))) caps.add('Vision')
  if (tags.some((tag) => ['tool-use', 'tools', 'function-calling'].includes(tag))) caps.add('Tool Use')

  if (
    /embedding|embed|sentence-transformers|all-minilm|bge-|bge_|e5-|gte-|nomic-embed|gte|jina-embeddings/.test(all) ||
    purpose === 'embedding' ||
    task.includes('embedding') ||
    pipeline.includes('feature-extraction')
  ) {
    caps.add('Embedding')
  } else if (
    !caps.size &&
    (purpose === 'chat' ||
      task.includes('chat') ||
      task.includes('text-generation') ||
      pipeline.includes('text-generation') ||
      (Boolean(name) && getModelFormat(model) === 'GGUF'))
  ) {
    caps.add('Chat')
  }

  if (!caps.size && isRunnableModel(model)) caps.add('Chat')
  return [...caps]
}

export const getModelUpdatedDate = (model) => {
  const raw = model?.updatedAt || model?.modifiedAt || model?.lastModified || model?.created_at || model?.createdAt
  const date = normalizeDateValue(raw)
  return !date || Number.isNaN(date.getTime()) ? '' : date.toISOString().slice(0, 10)
}

export const formatRelativeDate = (value = '', now = new Date()) => {
  if (!value) return ''
  const date = normalizeDateValue(value)
  if (!date || Number.isNaN(date.getTime())) return ''
  const days = Math.floor((now - date) / 86400000)
  if (days < 1) return 'today'
  if (days < 30) return days === 1 ? '1 day ago' : `${days} days ago`
  const months = Math.floor(days / 30)
  if (months < 12) return months === 1 ? '1 month ago' : `${months} months ago`
  const years = Math.floor(days / 365)
  return years === 1 ? '1 year ago' : `${years} years ago`
}

export const getModelSource = (model) =>
  isLocalModel(model) ? 'Local' : isRemoteModel(model) ? 'Hugging Face' : model?.provider || 'Unknown'

export const getModelLicense = (model) => String(model?.cardData?.license || model?.license || '').trim() || 'unknown'

export const getModelRuntime = (model) => {
  const provider = String(model?.provider || '').toLowerCase()
  const engine = String(model?.engine || '').toLowerCase()
  if (provider === 'tauri-rust') return 'Tauri Rust'
  if (provider === 'node-llama-cpp') return 'llama.cpp'
  if (provider === 'browser-webllm' || provider === 'webllm' || engine === 'webllm') return 'WebLLM'
  if (provider === 'local-ocr' || engine === 'tesseract') return 'Tesseract'
  return model?.engine || model?.provider || 'llama.cpp'
}

export const getModelReadme = (model) => {
  const description = getModelSummary(model)
  return {
    creator: resolveModelAuthor(model) || 'unknown',
    original: String(model?.repoId || model?.id || '').trim() || 'unknown',
    format: getModelFormat(model),
    runtime: getModelRuntime(model),
    license: getModelLicense(model),
    description: description === 'No description available.' ? 'No description available for this model.' : description
  }
}

export const filterByFormat = (models = [], format = 'all') =>
  !format || format === 'all' ? [...models] : models.filter((model) => getModelFormat(model).toLowerCase() === format.toLowerCase())

export const filterBySource = (models = [], source = 'all') =>
  !source || source === 'all'
    ? [...models]
    : source === 'remote'
      ? models.filter((model) => isRemoteModel(model) && !isLocalModel(model))
      : source === 'installed' || source === 'local'
        ? models.filter(isLocalModel)
        : [...models]

export const sortModels = (models = [], sort = 'best') => {
  const list = dedupeModelsById(models)
  if (sort === 'downloads') return list.sort((a, b) => Number(b.downloads || 0) - Number(a.downloads || 0))
  if (sort === 'likes') return list.sort((a, b) => Number(b.likes || 0) - Number(a.likes || 0))
  if (sort === 'updated') {
    return list.sort(
      (a, b) =>
        (normalizeDateValue(b.updatedAt || b.modifiedAt)?.getTime() || 0) -
        (normalizeDateValue(a.updatedAt || a.modifiedAt)?.getTime() || 0)
    )
  }
  if (sort === 'name') return list.sort((a, b) => resolveModelName(a).localeCompare(resolveModelName(b)))
  return sortByPopularity(list)
}

export const applyCatalogFilters = ({ models = [], query = '', format = 'all', source = 'all', sort = 'best' } = {}) =>
  sortModels(filterBySource(filterByFormat(filterModelsByName(models, query), format), source), sort)
