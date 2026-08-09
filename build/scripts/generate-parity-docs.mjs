import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const root = path.resolve(__dirname, '../..')
const outDir = path.join(root, 'docs', 'parity', 'generated')
const tasksPerFeature = 410

const features = [
  {
    slug: '01-build-runtime-dependencies',
    title: 'Build and runtime dependencies',
    prefix: 'BUILD',
    observed: [
      'Electron dev build fails when runtime dependencies are imported but missing from package.json.',
      'Tauri must not require Electron-only packages in the renderer bundle.',
      'Vitest aliases can hide missing production dependencies if not audited separately.'
    ],
    files: [
      'package.json',
      'pnpm-lock.yaml',
      'vite.tauri.config.js',
      'Elephant/backend/tauri/**',
      'Elephant/frontend/src/renderer/src/**'
    ],
    verbs: [
      'audit',
      'resolve',
      'externalize',
      'declare',
      'lock',
      'validate',
      'document',
      'test',
      'log',
      'fail fast'
    ]
  },
  {
    slug: '02-renderer-bootstrap-blank-page',
    title: 'Renderer bootstrap and blank page prevention',
    prefix: 'BOOT',
    observed: [
      'The app can mount a white page while unit tests remain green.',
      'Renderer import errors must surface in terminal and debug overlay.',
      'The real app root must be mounted by tests, not only fake harnesses.'
    ],
    files: [
      'Elephant/frontend/src/renderer/src/main.js',
      'Elephant/frontend/src/renderer/src/pages/app.vue',
      'Elephant/frontend/src/renderer/src/platform/runtimeBridge.js',
      'Elephant/frontend/app/components/shell/AppShell.vue'
    ],
    verbs: [
      'mount',
      'assert',
      'log',
      'detect',
      'surface',
      'recover',
      'snapshot',
      'compare',
      'route',
      'bootstrap'
    ]
  },
  {
    slug: '03-runtime-facade-parity',
    title: 'Electron and Tauri runtime facade parity',
    prefix: 'RUNTIME',
    observed: [
      'Renderer code depends on Electron-style globals and path/file facades.',
      'Tauri facades must be API-compatible with Electron renderer expectations.',
      'Missing facade methods cause white pages and action failures.'
    ],
    files: [
      'Elephant/frontend/src/renderer/src/platform/runtimeBridge.js',
      'Elephant/frontend/src/renderer/src/platform/rendererPathFacade.js',
      'Elephant/frontend/src/preload/**',
      'Elephant/backend/tauri/src/**'
    ],
    verbs: [
      'normalize',
      'polyfill',
      'bridge',
      'assert',
      'document',
      'compare',
      'mock',
      'call',
      'guard',
      'trace'
    ]
  },
  {
    slug: '04-vault-loading-workspace-state',
    title: 'Vault loading and workspace state',
    prefix: 'VAULT',
    observed: [
      'Vault state must load consistently across Electron and Tauri.',
      'Internal files must not be opened as normal notes during bootstrap.',
      'Workspace view and entries must reflect the selected vault.'
    ],
    files: [
      'Elephant/frontend/app/stores/vaultStore.js',
      'Elephant/frontend/app/services/elephantnoteClient.js',
      'Elephant/backend/js/vaults/**'
    ],
    verbs: [
      'load',
      'select',
      'persist',
      'hydrate',
      'filter',
      'open',
      'refresh',
      'recover',
      'migrate',
      'verify'
    ]
  },
  {
    slug: '05-notes-cards-preview',
    title: 'Notes list, cards, preview and metadata',
    prefix: 'CARD',
    observed: [
      'Note cards show frontmatter/debug metadata instead of useful note text.',
      'Card preview should use the note body, not raw YAML frontmatter.',
      'Card visuals must align with Electron baseline.'
    ],
    files: [
      'Elephant/frontend/app/utils/noteCardView.js',
      'Elephant/frontend/app/components/**/Note*.vue',
      'Elephant/shared/markdownDocument.js'
    ],
    verbs: [
      'strip',
      'render',
      'truncate',
      'preview',
      'resolve',
      'style',
      'sort',
      'filter',
      'refresh',
      'snapshot'
    ]
  },
  {
    slug: '06-editor-content-save-reopen',
    title: 'Editor content, save and reopen workflow',
    prefix: 'EDITOR',
    observed: [
      'Editor may receive markdown length but not display the note text correctly.',
      'Saving must update file, cards, search, graph and opened state.',
      'Opening a note after save must restore exact saved content.'
    ],
    files: [
      'Elephant/frontend/app/components/editor/NoteEditorHost.vue',
      'Elephant/frontend/app/components/editor/NoteEditorTopBar.vue',
      'Elephant/frontend/src/renderer/src/muya/**'
    ],
    verbs: [
      'open',
      'edit',
      'save',
      'reopen',
      'sync',
      'index',
      'render',
      'diff',
      'persist',
      'recover'
    ]
  },
  {
    slug: '07-title-metadata-consistency',
    title: 'Title and metadata consistency',
    prefix: 'TITLE',
    observed: [
      'Editor title and note card title can diverge.',
      'Filename, frontmatter title and first heading require a single policy.',
      'Rename flows must update every UI surface.'
    ],
    files: [
      'Elephant/shared/markdownDocument.js',
      'Elephant/frontend/app/utils/noteCardView.js',
      'Elephant/frontend/app/stores/vaultStore.js'
    ],
    verbs: [
      'resolve',
      'rename',
      'sync',
      'prioritize',
      'fallback',
      'refresh',
      'compare',
      'migrate',
      'normalize',
      'assert'
    ]
  },
  {
    slug: '08-pin-unpin-workflow',
    title: 'Pin and unpin workflow',
    prefix: 'PIN',
    observed: [
      'NoteEditorHost calls store.togglePin but the store does not expose it.',
      'Pin state must be consistent between editor, card and sidebar.',
      'Pinned paths must migrate on rename and move.'
    ],
    files: [
      'Elephant/frontend/app/stores/vaultStore.js',
      'Elephant/frontend/app/components/editor/NoteEditorHost.vue',
      'Elephant/frontend/app/components/**/Note*.vue'
    ],
    verbs: [
      'restore',
      'alias',
      'toggle',
      'persist',
      'migrate',
      'dedupe',
      'render',
      'sort',
      'log',
      'test'
    ]
  },
  {
    slug: '09-tags-frontmatter',
    title: 'Tags and frontmatter editing',
    prefix: 'TAGS',
    observed: [
      'Tag submission crashes when nextTags is not an array.',
      'Tag payload shape differs across NoteTagForm, top bar and host.',
      'Frontmatter must not be corrupted by tag edits.'
    ],
    files: [
      'Elephant/frontend/app/components/editor/NoteTagForm.vue',
      'Elephant/frontend/app/components/editor/NoteEditorTopBar.vue',
      'Elephant/shared/markdownDocument.js'
    ],
    verbs: [
      'normalize',
      'submit',
      'validate',
      'dedupe',
      'escape',
      'write',
      'preserve',
      'remove',
      'log',
      'recover'
    ]
  },
  {
    slug: '10-search-indexing-ui',
    title: 'Search indexing and UI',
    prefix: 'SEARCH',
    observed: [
      'Search finds nothing and updateNoteIndex is missing.',
      'Search must index title, body, tags and path.',
      'Search state must update after save, rename, move and delete.'
    ],
    files: [
      'Elephant/frontend/app/stores/searchStore.js',
      'Elephant/frontend/app/components/search/**',
      'Elephant/frontend/app/stores/vaultStore.js'
    ],
    verbs: [
      'index',
      'query',
      'rank',
      'highlight',
      'open',
      'update',
      'delete',
      'refresh',
      'debounce',
      'log'
    ]
  },
  {
    slug: '11-file-folder-operations',
    title: 'File and folder operations',
    prefix: 'FILES',
    observed: [
      'Moving a note or folder into a folder does nothing.',
      'Create, rename, move and delete must refresh all dependent stores.',
      'Open note paths must follow move and rename operations.'
    ],
    files: [
      'Elephant/frontend/app/stores/vaultStore.js',
      'Elephant/backend/js/vaults/**',
      'Elephant/backend/tauri/src/**'
    ],
    verbs: [
      'create',
      'rename',
      'move',
      'delete',
      'refresh',
      'migrate',
      'reject',
      'confirm',
      'persist',
      'reload'
    ]
  },
  {
    slug: '12-hidden-internal-storage',
    title: 'Hidden internal storage filtering',
    prefix: 'HIDDEN',
    observed: [
      'Internal .elephantnote files can leak into normal notes view.',
      'Dashboard.md should not open as a user note.',
      'Wiki files are visible only from the Wiki view.'
    ],
    files: [
      'Elephant/frontend/app/stores/vaultStore.js',
      'Elephant/shared/**',
      'Elephant/backend/js/vaults/**'
    ],
    verbs: [
      'filter',
      'hide',
      'expose',
      'scope',
      'migrate',
      'assert',
      'exclude',
      'include',
      'document',
      'warn'
    ]
  },
  {
    slug: '13-wiki-view',
    title: 'Wiki view',
    prefix: 'WIKI',
    observed: [
      'Wiki must be a dedicated view rooted in the hidden wiki folder.',
      'Normal explorer must not show the wiki internal folder.',
      'Wiki proposals must load, accept and dismiss without crashes.'
    ],
    files: [
      'Elephant/frontend/app/components/views/WikiView.vue',
      'Elephant/frontend/app/stores/vaultStore.js',
      'Elephant/backend/js/wiki/**'
    ],
    verbs: [
      'list',
      'open',
      'create',
      'accept',
      'dismiss',
      'regenerate',
      'filter',
      'render',
      'log',
      'test'
    ]
  },
  {
    slug: '14-graph-view',
    title: 'Graph view',
    prefix: 'GRAPH',
    observed: [
      'Graph shows only one note.',
      'Clicking a graph note teleports far away.',
      'Graph timeline can render Invalid Date.'
    ],
    files: [
      'Elephant/frontend/app/components/views/GraphView.vue',
      'Elephant/frontend/app/stores/vaultStore.js',
      'Elephant/frontend/app/stores/searchStore.js'
    ],
    verbs: [
      'model',
      'count',
      'layout',
      'focus',
      'clamp',
      'select',
      'preview',
      'timeline',
      'exclude',
      'snapshot'
    ]
  },
  {
    slug: '15-models-page-ai',
    title: 'Models page and AI provider state',
    prefix: 'MODELS',
    observed: [
      'Model page does not work completely.',
      'Backend capabilities must be honestly shown when not implemented.',
      'Download, search, uninstall and role assignment need real integration.'
    ],
    files: [
      'Elephant/frontend/app/components/views/ModelsView.vue',
      'Elephant/frontend/app/components/views/modelsViewHelpers.js',
      'Elephant/backend/js/models/**'
    ],
    verbs: [
      'search',
      'list',
      'download',
      'cancel',
      'uninstall',
      'assign',
      'persist',
      'readme',
      'render',
      'fallback'
    ]
  },
  {
    slug: '16-settings-parity',
    title: 'Settings parity',
    prefix: 'SETTINGS',
    observed: [
      'Settings render differently compared to Electron baseline.',
      'Every settings page needs functional and visual parity.',
      'Provider/model/vault settings must persist changes.'
    ],
    files: [
      'Elephant/frontend/app/components/settings/**',
      'Elephant/frontend/app/components/settings/SettingsPanel.vue',
      'Elephant/frontend/src/renderer/src/store/preferences.js'
    ],
    verbs: [
      'render',
      'toggle',
      'persist',
      'compare',
      'snapshot',
      'style',
      'align',
      'validate',
      'reset',
      'recover'
    ]
  },
  {
    slug: '17-titlebar-draggable-area',
    title: 'Titlebar and draggable area',
    prefix: 'TITLEBAR',
    observed: [
      'The Tauri drag area does not match the Electron titlebar.',
      'Mac traffic lights and app title spacing must be preserved.',
      'Drag zones must not block interactive controls.'
    ],
    files: [
      'Elephant/frontend/app/components/shell/**',
      'Elephant/frontend/src/renderer/src/pages/app.vue',
      'Elephant/backend/tauri/tauri.conf.json'
    ],
    verbs: [
      'measure',
      'drag',
      'click',
      'align',
      'style',
      'compare',
      'mask',
      'snapshot',
      'clamp',
      'document'
    ]
  },
  {
    slug: '18-chat-layout',
    title: 'Chat layout',
    prefix: 'CHAT',
    observed: [
      'Chat overlays the note instead of resizing the editor.',
      'Chat close button can disappear when the panel is enlarged.',
      'Chat must degrade honestly when AI backend is unavailable.'
    ],
    files: [
      'Elephant/frontend/app/components/chat/**',
      'Elephant/frontend/app/components/editor/NoteEditorHost.vue',
      'Elephant/frontend/app/components/shell/AppShell.vue'
    ],
    verbs: [
      'open',
      'close',
      'resize',
      'clamp',
      'preserve',
      'fallback',
      'visible',
      'persist',
      'layout',
      'snapshot'
    ]
  },
  {
    slug: '19-slash-excalidraw-rich-blocks',
    title: 'Slash commands, Excalidraw and rich markdown blocks',
    prefix: 'SLASH',
    observed: [
      'Excalidraw does not work.',
      'Features triggered with / in markdown appear broken.',
      'Rich blocks must save and reopen correctly.'
    ],
    files: [
      'Elephant/frontend/src/renderer/src/muya/**',
      'Elephant/frontend/app/components/editor/**',
      'Elephant/frontend/app/components/rich-blocks/**'
    ],
    verbs: [
      'trigger',
      'filter',
      'insert',
      'render',
      'persist',
      'reload',
      'fallback',
      'mount',
      'bridge',
      'test'
    ]
  },
  {
    slug: '20-scroll-layout-overflow',
    title: 'Scroll, layout and overflow parity',
    prefix: 'SCROLL',
    observed: [
      'The app scrolls horizontally instead of vertically.',
      'Normal note grid should wrap and vertical-scroll.',
      'Every page must define its intended scroll container.'
    ],
    files: [
      'Elephant/frontend/app/components/shell/**',
      'Elephant/frontend/app/components/views/**',
      'Elephant/frontend/src/renderer/src/pages/app.vue'
    ],
    verbs: [
      'measure',
      'clamp',
      'wrap',
      'scroll',
      'contain',
      'hide',
      'audit',
      'warn',
      'snapshot',
      'compare'
    ]
  },
  {
    slug: '21-logging-diagnostics',
    title: 'Logging and diagnostics',
    prefix: 'LOG',
    observed: [
      'Dev terminal logs do not cover enough user actions yet.',
      'Renderer errors need actionable context.',
      'Manual testing must produce enough logs to fix first error quickly.'
    ],
    files: [
      'Elephant/frontend/src/renderer/src/diagnostics/**',
      'Elephant/frontend/app/stores/**',
      'Elephant/frontend/src/renderer/src/main.js',
      'Elephant/backend/tauri/src/**'
    ],
    verbs: [
      'log',
      'forward',
      'rate-limit',
      'capture',
      'serialize',
      'copy',
      'tag',
      'buffer',
      'assert',
      'report'
    ]
  },
  {
    slug: '22-visual-parity-screenshots',
    title: 'Visual parity and screenshot baselines',
    prefix: 'VISUAL',
    observed: [
      'Pixel-perfect cannot be proven with jsdom only.',
      'Visual parity needs real browser/window screenshots after functional stability.',
      'Electron and Tauri must be compared on the same seed vault.'
    ],
    files: ['tests/app/e2e/**', 'playwright.config.*', 'agent/docs/project/parity/**'],
    verbs: [
      'seed',
      'capture',
      'mask',
      'compare',
      'threshold',
      'freeze',
      'artifact',
      'approve',
      'diff',
      'document'
    ]
  },
  {
    slug: '23-real-test-strategy',
    title: 'Real test strategy',
    prefix: 'TEST',
    observed: [
      'Large artificial test counts gave false confidence.',
      'Real tests must import production code or launch the real app.',
      'Every regression test must be mapped to a user-visible behavior.'
    ],
    files: [
      'tests/app/unit/**',
      'tests/app/e2e/**',
      'vitest.config.js',
      'agent/docs/project/parity/**'
    ],
    verbs: [
      'delete',
      'replace',
      'mount',
      'import',
      'launch',
      'assert',
      'link',
      'classify',
      'fail',
      'document'
    ]
  },
  {
    slug: '24-cleanup-generated-tests',
    title: 'Cleanup of artificial generated tests',
    prefix: 'CLEAN',
    observed: [
      'Some generated tests are not close enough to production behavior.',
      'Useful cases should be migrated to real components or stores.',
      'Test value must be tracked separately from test count.'
    ],
    files: [
      'tests/app/unit/*Generated*.spec.js',
      'tests/app/unit/**/*Generated*.spec.js',
      'agent/docs/project/parity/**'
    ],
    verbs: [
      'remove',
      'migrate',
      'keep',
      'replace',
      'review',
      'tag',
      'classify',
      'document',
      'justify',
      'verify'
    ]
  },
  {
    slug: '25-immediate-next-fixes',
    title: 'Immediate next fixes',
    prefix: 'NEXT',
    observed: [
      'Current manual testing revealed concrete runtime defects.',
      'Fix the first runtime error before cosmetic parity work.',
      'Keep this file updated after every fix commit.'
    ],
    files: [
      'agent/docs/project/TAURI_ELECTRON_PARITY_TODO.md',
      'agent/docs/project/parity/generated/**',
      'all affected production files'
    ],
    verbs: [
      'fix',
      'verify',
      'test',
      'commit',
      'link',
      'log',
      'rerun',
      'document',
      'prioritize',
      'close'
    ]
  }
]

const phases = [
  'reproduce in Electron baseline',
  'reproduce in Tauri runtime',
  'identify production source file',
  'write failing real test',
  'fix production implementation',
  'verify with unit test',
  'verify with component test',
  'verify with runtime smoke test',
  'verify with visual snapshot when applicable',
  'document remaining gap honestly'
]

const surfaces = [
  'empty state',
  'loading state',
  'normal state',
  'error state',
  'keyboard path',
  'mouse path',
  'touch/trackpad path',
  'small-window layout',
  'large-window layout',
  'runtime parity path'
]

const assertions = [
  'must not produce a blank page',
  'must not throw a renderer error',
  'must not log an uncaught exception',
  'must update the visible UI',
  'must update the store state',
  'must update persisted state when required',
  'must be deterministic after reload',
  'must match Electron behavior',
  'must expose an honest unavailable state if missing',
  'must have at least one regression test'
]

fs.rmSync(outDir, { recursive: true, force: true })
fs.mkdirSync(outDir, { recursive: true })

const indexLines = [
  '# Generated parity backlog index',
  '',
  `Generated by \`build/scripts/generate-parity-docs.mjs\`.`,
  '',
  `Features: ${features.length}`,
  `Tasks per feature: ${tasksPerFeature}`,
  `Minimum task lines: ${features.length * tasksPerFeature}`,
  '',
  '## Files',
  ''
]

for (const feature of features) {
  const filename = `${feature.slug}.md`
  const lines = []
  lines.push(`# ${feature.title}`)
  lines.push('')
  lines.push(`Prefix: \`${feature.prefix}\``)
  lines.push('')
  lines.push('## Observed problems')
  lines.push('')
  for (const problem of feature.observed) lines.push(`- ${problem}`)
  lines.push('')
  lines.push('## Production files to inspect')
  lines.push('')
  for (const file of feature.files) lines.push(`- \`${file}\``)
  lines.push('')
  lines.push('## Generated actionable tasks')
  lines.push('')

  for (let i = 1; i <= tasksPerFeature; i += 1) {
    const phase = phases[(i - 1) % phases.length]
    const surface = surfaces[Math.floor((i - 1) / phases.length) % surfaces.length]
    const assertion =
      assertions[Math.floor((i - 1) / (phases.length * surfaces.length)) % assertions.length]
    const verb = feature.verbs[(i - 1) % feature.verbs.length]
    const file = feature.files[(i - 1) % feature.files.length]
    const id = `${feature.prefix}-${String(i).padStart(4, '0')}`
    lines.push(
      `- [ ] ${id} ${verb} ${surface}: ${phase}; inspect \`${file}\`; acceptance: ${assertion}.`
    )
  }

  lines.push('')
  lines.push('## Completion rule')
  lines.push('')
  lines.push(
    `Do not close ${feature.prefix} tasks by changing tests only. Each closed item must be linked to a production-code change, a real production import/mount test, or a runtime/visual parity result.`
  )
  lines.push('')

  fs.writeFileSync(path.join(outDir, filename), `${lines.join('\n')}\n`, 'utf8')
  indexLines.push(`- [${feature.title}](generated/${filename}) — ${tasksPerFeature} task lines`)
}

indexLines.push('')
indexLines.push('## Regeneration')
indexLines.push('')
indexLines.push('```bash')
indexLines.push('pnpm docs:parity')
indexLines.push('wc -l agent/docs/project/parity/generated/*.md agent/docs/project/parity/index.md')
indexLines.push('```')
indexLines.push('')

fs.writeFileSync(
  path.join(root, 'docs', 'parity', 'index.md'),
  `${indexLines.join('\n')}\n`,
  'utf8'
)

const total = features.length * tasksPerFeature
console.log(
  `Generated ${features.length} feature files with ${total} task lines in ${path.relative(root, outDir)}`
)
