<template>
  <section class="en-graph-premium">
    <div
      ref="containerRef"
      class="en-graph-stage"
      @click="onStageClick"
    />

    <div
      v-if="statusMessage"
      class="en-graph-status"
    >
      {{ statusMessage }}
    </div>

    <div class="en-graph-floating-icons">
      <button
        type="button"
        class="en-graph-floating-icon"
        :class="{ active: timelapseActive }"
        title="Démarrer le timelapse"
        @click="toggleTimelapse"
      >
        <Wand2 class="en-fi-svg" />
      </button>
      <button
        type="button"
        class="en-graph-floating-icon"
        :class="{ active: panelOpen }"
        title="Options du graphe"
        @click="panelOpen = !panelOpen"
      >
        <Settings class="en-fi-svg" />
      </button>
    </div>

    <div
      v-if="!timelapseActive"
      class="en-graph-bottom-left"
    >
      <div
        v-if="showStats"
        class="en-graph-stats"
      >
        {{ displayData.nodes.length }} nœuds · {{ displayData.edges.length }} liens
        <span
          v-if="indexReady"
          class="en-graph-stats-dot"
        />
        <span
          v-else-if="indexBuilding"
          class="en-graph-stats-dot en-graph-stats-dot-building"
        />
      </div>
      <div class="en-graph-zoom-control">
        <button
          type="button"
          class="en-graph-zoom-button"
          title="Recentrer"
          @click="resetView"
        >
          <Crosshair class="en-gz-svg" />
        </button>
        <input
          type="range"
          class="en-graph-zoom-slider"
          min="0.1"
          max="4"
          step="0.01"
          :value="zoomValue"
          @input="onZoomSlider"
        >
        <span class="en-graph-zoom-pct">{{ Math.round(zoomValue * 100) }}%</span>
      </div>
    </div>

    <transition name="en-panel">
      <aside
        v-if="panelOpen"
        class="en-graph-settings-panel"
      >
        <div class="en-panel-topbar">
          <span class="en-panel-topbar-title">Options</span>
          <div class="en-panel-topbar-actions">
            <button
              type="button"
              class="en-panel-icon-btn"
              title="Réinitialiser"
              @click="resetOptions"
            >
              <RotateCcw class="en-panel-icn" />
            </button>
            <button
              type="button"
              class="en-panel-icon-btn"
              title="Fermer"
              @click="panelOpen = false"
            >
              <X class="en-panel-icn" />
            </button>
          </div>
        </div>

        <div class="en-panel-scroll">
          <div
            v-for="section in sections"
            :key="section.id"
            class="en-panel-section"
          >
            <button
              type="button"
              class="en-section-header"
              :class="{ open: section.open }"
              @click="toggleSection(section)"
            >
              <ChevronDown class="en-section-chevron" />
              <span class="en-section-title">{{ section.title }}</span>
            </button>
            <transition name="en-accordion">
              <div
                v-if="section.open"
                class="en-section-body"
              >
                <template v-if="section.id === 'filters'">
                  <div class="en-filter-search">
                    <Search class="en-filter-search-icon" />
                    <input
                      v-model="filterQuery"
                      type="text"
                      class="en-filter-input"
                      placeholder="Filtrer les notes…"
                    >
                  </div>
                  <div class="en-filter-row">
                    <span class="en-filter-label">Mots-clés</span>
                    <button
                      type="button"
                      class="en-switch"
                      :class="{ active: filterTags }"
                      @click="filterTags = !filterTags"
                    >
                      <span class="en-switch-thumb" />
                    </button>
                  </div>
                  <div class="en-filter-row">
                    <span class="en-filter-label">Fichiers existants uniquement</span>
                    <button
                      type="button"
                      class="en-switch"
                      :class="{ active: filterExisting }"
                      @click="filterExisting = !filterExisting"
                    >
                      <span class="en-switch-thumb" />
                    </button>
                  </div>
                  <div class="en-filter-row">
                    <span class="en-filter-label">Orphelins</span>
                    <button
                      type="button"
                      class="en-switch"
                      :class="{ active: filterOrphans }"
                      @click="filterOrphans = !filterOrphans"
                    >
                      <span class="en-switch-thumb" />
                    </button>
                  </div>
                </template>

                <template v-if="section.id === 'groups'">
                  <button
                    type="button"
                    class="en-primary-button"
                    @click="addGroup"
                  >
                    <Plus class="en-btn-icn" />
                    Nouveau groupe
                  </button>
                  <div
                    v-if="groups.length"
                    class="en-group-list"
                  >
                    <div
                      v-for="group in groups"
                      :key="group.id"
                      class="en-group-item"
                    >
                      <span
                        class="en-group-dot"
                        :style="{ background: group.color }"
                      />
                      <span class="en-group-name">{{ group.name }}</span>
                      <span class="en-group-count">{{ group.count }}</span>
                    </div>
                  </div>
                </template>

                <template v-if="section.id === 'display'">
                  <div class="en-filter-row">
                    <span class="en-filter-label">Afficher les labels</span>
                    <button
                      type="button"
                      class="en-switch"
                      :class="{ active: showLabels }"
                      @click="showLabels = !showLabels"
                    >
                      <span class="en-switch-thumb" />
                    </button>
                  </div>
                  <div class="en-filter-row">
                    <span class="en-filter-label">Afficher les statistiques</span>
                    <button
                      type="button"
                      class="en-switch"
                      :class="{ active: showStats }"
                      @click="showStats = !showStats"
                    >
                      <span class="en-switch-thumb" />
                    </button>
                  </div>
                  <div class="en-slider-control">
                    <span class="en-slider-label">Seuil d'apparition du texte</span>
                    <input
                      v-model.number="labelThreshold"
                      type="range"
                      class="en-slider"
                      min="2"
                      max="20"
                      step="0.5"
                    >
                  </div>
                  <div class="en-slider-control">
                    <span class="en-slider-label">Taille des nœuds</span>
                    <input
                      v-model.number="nodeSizeScale"
                      type="range"
                      class="en-slider"
                      min="0.5"
                      max="2.5"
                      step="0.05"
                    >
                  </div>
                  <div class="en-slider-control">
                    <span class="en-slider-label">Épaisseur des liens</span>
                    <input
                      v-model.number="linkThickness"
                      type="range"
                      class="en-slider"
                      min="0.3"
                      max="2.5"
                      step="0.05"
                    >
                  </div>
                  <button
                    type="button"
                    class="en-primary-button"
                    :disabled="animating"
                    @click="animateGraph"
                  >
                    <Play
                      v-if="!animating"
                      class="en-btn-icn"
                    />
                    <Loader
                      v-else
                      class="en-btn-icn en-spin"
                    />
                    {{ animating ? 'Animation…' : 'Animer' }}
                  </button>
                </template>

                <template v-if="section.id === 'forces'">
                  <div class="en-slider-control">
                    <span class="en-slider-label">Force centrale</span>
                    <input
                      v-model.number="forceCenter"
                      type="range"
                      class="en-slider"
                      min="0.05"
                      max="0.6"
                      step="0.01"
                    >
                  </div>
                  <div class="en-slider-control">
                    <span class="en-slider-label">Force de répulsion</span>
                    <input
                      v-model.number="forceRepulsion"
                      type="range"
                      class="en-slider"
                      min="100"
                      max="900"
                      step="10"
                    >
                  </div>
                  <div class="en-slider-control">
                    <span class="en-slider-label">Force de liaison</span>
                    <input
                      v-model.number="forceLink"
                      type="range"
                      class="en-slider"
                      min="0.1"
                      max="1.2"
                      step="0.01"
                    >
                  </div>
                  <div class="en-slider-control">
                    <span class="en-slider-label">Distance des liens</span>
                    <input
                      v-model.number="forceLinkDistance"
                      type="range"
                      class="en-slider"
                      min="40"
                      max="220"
                      step="2"
                    >
                  </div>
                  <button
                    type="button"
                    class="en-primary-button"
                    @click="animateGraph"
                  >
                    <Zap class="en-btn-icn" />
                    Relancer la simulation
                  </button>
                </template>
              </div>
            </transition>
          </div>
        </div>
      </aside>
    </transition>

    <transition name="en-timeline">
      <div
        v-if="timelapseActive"
        class="en-timeline-panel"
      >
        <button
          type="button"
          class="en-timeline-play"
          @click="timelapsePlaying = !timelapsePlaying"
        >
          <Play
            v-if="!timelapsePlaying"
            class="en-tl-icn"
          />
          <Pause
            v-else
            class="en-tl-icn"
          />
        </button>
        <input
          type="range"
          class="en-timeline-slider"
          min="0"
          max="100"
          step="1"
          :value="timelapseProgress"
          @input="onTimelapseSeek"
        >
        <span class="en-timeline-date">{{ timelapseLabel }}</span>
        <button
          type="button"
          class="en-timeline-speed"
          @click="cycleTimelapseSpeed"
        >
          x{{ timelapseSpeed }}
        </button>
        <button
          type="button"
          class="en-timeline-close"
          @click="stopTimelapse"
        >
          <X class="en-tl-icn" />
        </button>
      </div>
    </transition>

    <transition name="en-card">
      <div
        v-if="selectedNode"
        class="en-note-preview-card"
        :class="{ collapsed: cardCollapsed, dragging: cardDragging }"
        :style="cardStyle"
      >
        <div
          class="en-card-header"
          @mousedown.prevent="startCardDrag"
        >
          <h3 class="en-card-title">
            {{ selectedNode.title }}
          </h3>
          <div class="en-card-header-actions">
            <button
              type="button"
              class="en-card-mini-btn"
              :title="cardCollapsed ? 'Agrandir' : 'Réduire'"
              @click="cardCollapsed = !cardCollapsed"
            >
              <ChevronDown
                v-if="!cardCollapsed"
                class="en-card-icn"
              />
              <ChevronUp
                v-else
                class="en-card-icn"
              />
            </button>
            <button
              type="button"
              class="en-card-mini-btn"
              title="Fermer"
              @click="deselectNode"
            >
              <X class="en-card-icn" />
            </button>
          </div>
        </div>

        <template v-if="!cardCollapsed">
          <div class="en-card-meta">
            <span>{{ selectedNode.kind || 'note' }}</span>
            <span>{{ selectedNode.sourceCount || 0 }} sources</span>
            <span>{{ selectedNode.chunkCount || 0 }} chunks</span>
          </div>

          <p class="en-card-summary">
            {{ selectedNode.summary || 'Aucun résumé pour cette note.' }}
          </p>

          <div
            v-if="selectedNode.tags?.length"
            class="en-card-tags"
          >
            <span
              v-for="tag in selectedNode.tags.slice(0, 12)"
              :key="tag"
              class="en-card-tag"
            >#{{ tag }}</span>
          </div>

          <button
            type="button"
            class="en-card-open"
            @click="openSelectedNode"
          >
            Ouvrir la note
            <ArrowRight class="en-card-open-icn" />
          </button>
        </template>
      </div>
    </transition>
  </section>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import Graph from 'graphology'
import Sigma from 'sigma'
import EdgeCurveProgram from '@sigma/edge-curve'
import {
  ArrowRight,
  ChevronDown,
  ChevronUp,
  Crosshair,
  Loader,
  Pause,
  Play,
  Plus,
  RotateCcw,
  Search,
  Settings,
  Wand2,
  X,
  Zap
} from '@lucide/vue'
import { useVaultStore } from '../../stores/vaultStore'
import { useSearchStore } from '../../stores/searchStore'
import { useCanvasStore } from '../../stores/canvasStore'
import {
  buildSemanticViewModel,
  buildGraphFromVaultEntries,
  selectSemanticGraphSource
} from './semanticGraphViewHelpers'
import { nodeColor } from '../../graph/graphThemes'

const store = useVaultStore()
const searchStore = useSearchStore()
const canvasStore = useCanvasStore()

const NODE_PALETTE = [
  '#7c3aed', '#3b82f6', '#22d3ee', '#4ade80',
  '#eab308', '#ec4899', '#ef4444', '#a78bfa'
]

const containerRef = ref(null)
const graphData = ref(null)
const indexBuilding = ref(false)

let renderer = null
let graphInstance = null
let labelCanvas = null
let neighborsMap = new Map()

let hoveredNodeRef = null
let selectedNodeRef = null
let hoverAnimRef = 0
let selectAnimRef = 0
let hoverTargetRef = 0
let selectTargetRef = 0
let simRaf = null
let timelapseRaf = null
let hoverRaf = null
let graphMountRaf = null

const selectedNode = ref(null)
const panelOpen = ref(true)
const timelapseActive = ref(false)
const timelapsePlaying = ref(false)
const timelapseProgress = ref(0)
const timelapseSpeed = ref(1)
const animating = ref(false)
const zoomValue = ref(1)
const cardCollapsed = ref(false)
const cardPos = reactive({ x: null, y: null })
const cardDragging = ref(false)

const filterQuery = ref('')
const filterTags = ref(true)
const filterExisting = ref(true)
const filterOrphans = ref(false)

const showLabels = ref(true)
const showStats = ref(true)
const labelThreshold = ref(7)
const nodeSizeScale = ref(1)
const linkThickness = ref(1)

const forceCenter = ref(0.32)
const forceRepulsion = ref(420)
const forceLink = ref(0.55)
const forceLinkDistance = ref(95)

const sections = ref([
  { id: 'filters', title: 'Filtres', open: true },
  { id: 'groups', title: 'Groupes', open: false },
  { id: 'display', title: 'Afficher', open: false },
  { id: 'forces', title: 'Forces', open: false }
])

const groups = ref([])

const theme = computed(() => canvasStore.activeTheme)
const isLight = computed(() => theme.value?.isLight)

const indexReady = computed(() => {
  const g = searchStore.indexInspection?.graph
  return Array.isArray(g?.nodes) && g.nodes.length > 0
})

const fallbackGraph = computed(() => {
  if (indexReady.value) return null
  const entries = store.rootEntries || []
  if (entries.length === 0) return null
  return buildGraphFromVaultEntries(entries)
})

const rawGraph = computed(() => {
  if (indexReady.value) {
    return selectSemanticGraphSource({
      inspectionGraph: searchStore.indexInspection?.graph
    })
  }
  return fallbackGraph.value || { nodes: [], edges: [], clusters: [] }
})

const semanticModel = computed(() => {
  const nodeCount = rawGraph.value?.nodes?.length || 0
  const scale = Math.max(1, Math.sqrt(nodeCount) / 12)
  return buildSemanticViewModel({
    graph: rawGraph.value,
    savedPositions: canvasStore.canvasPositions,
    width: Math.round(1800 * scale),
    height: Math.round(1200 * scale)
  })
})

const filteredNodes = computed(() => {
  const nodes = semanticModel.value.nodes
  const q = filterQuery.value.trim().toLowerCase()
  if (!q) return nodes
  return nodes.filter((n) => {
    const title = String(n.title || '').toLowerCase()
    const tags = Array.isArray(n.tags) ? n.tags.join(' ').toLowerCase() : ''
    return title.includes(q) || tags.includes(q)
  })
})

const filteredNodeIds = computed(() => new Set(filteredNodes.value.map((n) => n.id)))

const MAX_VISIBLE_EDGES = 3000

const filteredEdges = computed(() => {
  const candidate = semanticModel.value.edges.filter((e) =>
    filteredNodeIds.value.has(e.source) && filteredNodeIds.value.has(e.target)
  )
  if (candidate.length <= MAX_VISIBLE_EDGES) return candidate
  const sorted = [...candidate].sort((a, b) => Number(b.weight || 0) - Number(a.weight || 0))
  return sorted.slice(0, MAX_VISIBLE_EDGES)
})

const renderEdges = computed(() => {
  const edges = semanticModel.value.edges
  if (edges.length <= MAX_VISIBLE_EDGES) return edges
  const sorted = [...edges].sort((a, b) => Number(b.weight || 0) - Number(a.weight || 0))
  return sorted.slice(0, MAX_VISIBLE_EDGES)
})

const displayData = computed(() => ({
  nodes: filteredNodes.value,
  edges: filteredEdges.value,
  edgeCounts: semanticModel.value.edgeCounts,
  maxEdges: semanticModel.value.maxEdges,
  clusters: semanticModel.value.clusters || []
}))

const statusMessage = computed(() => {
  if (indexBuilding.value) return 'Construction de l\'index sémantique…'
  if (searchStore.status.status === 'indexing') return searchStore.status.message || 'Indexation…'
  if (displayData.value.nodes.length === 0) {
    if (store.rootEntries?.length > 0) return 'Aucune note à afficher. Ajustez les filtres.'
    return 'Aucun vault chargé.'
  }
  return ''
})

const cardStyle = computed(() => {
  if (cardPos.x === null) return {}
  return { left: `${cardPos.x}px`, top: `${cardPos.y}px` }
})

const timelapseLabel = computed(() => {
  const nodes = displayData.value.nodes
  if (!nodes.length) return '—'
  const sorted = [...nodes].sort((a, b) => {
    const da = new Date(a.updatedAt || 0).getTime()
    const db = new Date(b.updatedAt || 0).getTime()
    return da - db
  })
  const idx = Math.min(sorted.length - 1, Math.floor((timelapseProgress.value / 100) * sorted.length))
  const node = sorted[idx]
  if (!node?.updatedAt) return `Note ${idx + 1}/${sorted.length}`
  return new Date(node.updatedAt).toLocaleDateString('fr-FR', { day: '2-digit', month: 'short', year: 'numeric' })
})

function truncLabel (str, max) {
  if (!str) return ''
  return str.length > max ? str.substring(0, max - 1) + '\u2026' : str
}

function edgeColorFor (type) {
  if (type === 'semantic') return '#6d5fd3'
  if (type === 'explicit-link') return '#3b9b96'
  if (type === 'folder') return '#d98a3b'
  if (type === 'tag') return '#9b6cff'
  if (type === 'lexical') return '#9b6cff'
  return '#5a6478'
}

function rgbToHex (rgb) {
  if (!Array.isArray(rgb)) return '#1e1e1e'
  const toHex = (n) => Math.max(0, Math.min(255, Math.round(n))).toString(16).padStart(2, '0')
  return `#${toHex(rgb[0])}${toHex(rgb[1])}${toHex(rgb[2])}`
}

function hexToRgb (hex) {
  const m = String(hex || '').match(/^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i)
  return m ? [parseInt(m[1], 16), parseInt(m[2], 16), parseInt(m[3], 16)] : null
}

function dimColor (rgb, target, amount) {
  if (!rgb) return Array.isArray(target) ? rgbToHex(target) : '#1e1e1e'
  const r = Math.round(rgb[0] + (target[0] - rgb[0]) * amount)
  const g = Math.round(rgb[1] + (target[1] - rgb[1]) * amount)
  const b = Math.round(rgb[2] + (target[2] - rgb[2]) * amount)
  return rgbToHex([r, g, b])
}

function dimHex (hexColor, target, amount) {
  const rgb = hexToRgb(hexColor)
  if (!rgb) return hexColor
  return dimColor(rgb, target, amount)
}

function buildGraph (data) {
  const graph = new Graph()
  const t = theme.value
  const { nodes, edges, edgeCounts, maxEdges } = data

  for (const node of nodes) {
    const id = node.id
    const connectivity = (edgeCounts.get(id) || 0) / maxEdges
    const clusterIdx = node.clusterIndex || 0
    const baseSize = 3 + connectivity * 6 + (node.kind === 'folder' ? 3 : 0)
    graph.addNode(id, {
      x: node.x,
      y: node.y,
      size: baseSize,
      color: nodeColor(t, Math.min(1, 0.3 + connectivity), clusterIdx),
      label: truncLabel(node.title, 28),
      fullLabel: node.title,
      connectivity,
      clusterIndex: clusterIdx,
      tagIds: node.tags || [],
      data: node
    })
  }

  let minW = 1
  let maxW = 0
  for (const edge of edges) {
    if (edge.weight < minW) minW = edge.weight
    if (edge.weight > maxW) maxW = edge.weight
  }
  const wRange = Math.max(maxW - minW, 0.001)

  for (const edge of edges) {
    if (!graph.hasNode(edge.source) || !graph.hasNode(edge.target)) continue
    if (graph.hasEdge(edge.source, edge.target) || graph.hasEdge(edge.target, edge.source)) continue
    const w = (edge.weight - minW) / wRange
    graph.addEdge(edge.source, edge.target, {
      weight: w,
      type: 'curved',
      edgeType: edge.type || 'semantic',
      color: edgeColorFor(edge.type)
    })
  }

  return graph
}

function getTimelapseVisibleIds (progress, nodes = []) {
  if (!nodes.length) return new Set()
  const sorted = [...nodes].sort((a, b) => {
    const da = new Date(a.updatedAt || 0).getTime()
    const db = new Date(b.updatedAt || 0).getTime()
    return da - db
  })
  const visibleCount = Math.max(1, Math.floor((progress / 100) * sorted.length))
  return new Set(sorted.slice(0, visibleCount).map((node) => node.id))
}

function buildNeighbors (graph) {
  const neighbors = new Map()
  graph.forEachEdge((edge, _attrs, source, target) => {
    if (!neighbors.has(source)) neighbors.set(source, new Set())
    if (!neighbors.has(target)) neighbors.set(target, new Set())
    neighbors.get(source).add(target)
    neighbors.get(target).add(source)
  })
  neighborsMap = neighbors
}

function drawLabels (sigma, graph, container) {
  const width = container.clientWidth
  const height = container.clientHeight
  if (!width || !height) return
  const ratio = window.devicePixelRatio || 1
  labelCanvas.width = width * ratio
  labelCanvas.height = height * ratio
  labelCanvas.style.width = `${width}px`
  labelCanvas.style.height = `${height}px`
  const ctx = labelCanvas.getContext('2d')
  if (!ctx) return
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0)
  ctx.clearRect(0, 0, width, height)

  const t = theme.value
  const placed = []

  function collides (rect, pad) {
    for (const p of placed) {
      if (rect.x - pad < p.x + p.w && rect.x + rect.w + pad > p.x &&
          rect.y - pad < p.y + p.h && rect.y + rect.h + pad > p.y) return true
    }
    return false
  }

  const sel = selectedNodeRef
  const hov = hoveredNodeRef
  const selAnim = selectAnimRef
  const hovAnim = hoverAnimRef

  const candidates = []
  graph.forEachNode((id, attrs) => {
    const pos = sigma.graphToViewport({ x: attrs.x, y: attrs.y })
    if (pos.x < -200 || pos.x > width + 50 || pos.y < -30 || pos.y > height + 50) return
    const rsize = sigma.scaleSize(attrs.size || 4) * nodeSizeScale.value
    candidates.push({
      id,
      vx: pos.x,
      vy: pos.y,
      rsize,
      label: attrs.label || '',
      fullLabel: attrs.fullLabel || attrs.label || ''
    })
  })

  candidates.sort((a, b) => {
    const aImp = (a.id === sel ? 100 : a.id === hov ? 50 : 0) + a.rsize
    const bImp = (b.id === sel ? 100 : b.id === hov ? 50 : 0) + b.rsize
    return bImp - aImp
  })

  const isPinned = (id) => id === sel || id === hov

  for (const c of candidates) {
    const pinned = isPinned(c.id)
    const isSel = c.id === sel
    const isHov = c.id === hov
    const cameraRatio = sigma.getCamera().getState().ratio
    const effectiveThreshold = labelThreshold.value * Math.max(0.5, cameraRatio)
    if (!pinned && !showLabels.value) continue
    if (!pinned && c.rsize < effectiveThreshold) continue

    const fontSize = isSel ? 14 : isHov ? 13 : 12
    const fontWeight = isSel ? 700 : isHov ? 600 : 500
    const maxChars = isSel ? 48 : isHov ? 36 : 26
    const labelText = truncLabel(c.fullLabel || c.label, maxChars)

    ctx.font = `${fontWeight} ${fontSize}px system-ui, -apple-system, sans-serif`
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    const tw = ctx.measureText(labelText).width
    const padX = pinned ? 10 : 8
    const padY = pinned ? 5 : 4
    const pillW = tw + padX * 2
    const pillH = fontSize + padY * 2
    const labelY = c.vy + c.rsize + pillH / 2 + 4
    const rect = { x: c.vx - pillW / 2, y: labelY - pillH / 2, w: pillW, h: pillH }

    if (!pinned && collides(rect, 6)) continue
    placed.push(rect)

    ctx.globalAlpha = isSel ? selAnim : isHov ? hovAnim : 0.85

    const labelBg = isLight.value ? 'rgba(18,22,30,0.9)' : 'rgba(18,22,30,0.92)'
    ctx.fillStyle = labelBg
    ctx.beginPath()
    ctx.roundRect(rect.x, rect.y, pillW, pillH, pillH / 2)
    ctx.fill()

    if (isSel) {
      ctx.strokeStyle = 'rgba(168,150,255,0.7)'
      ctx.lineWidth = 1.5
    } else if (isHov) {
      ctx.strokeStyle = 'rgba(170,160,255,0.45)'
      ctx.lineWidth = 1.2
    } else {
      ctx.strokeStyle = t.labelBorder || 'rgba(120,130,160,0.3)'
      ctx.lineWidth = 1
    }
    ctx.stroke()

    ctx.fillStyle = isSel ? '#f0edff' : isHov ? '#f0edff' : (t.nodeLabelColor || '#c8c8d4')
    ctx.fillText(labelText, c.vx, labelY)
    ctx.globalAlpha = 1

    if (isSel) {
      ctx.beginPath()
      ctx.arc(c.vx, c.vy, c.rsize + 4, 0, Math.PI * 2)
      ctx.strokeStyle = `rgba(168,150,255,${0.5 * selAnim})`
      ctx.lineWidth = 2
      ctx.stroke()
    } else if (isHov) {
      ctx.beginPath()
      ctx.arc(c.vx, c.vy, c.rsize + 3, 0, Math.PI * 2)
      ctx.strokeStyle = `rgba(255,255,255,${0.45 * hovAnim})`
      ctx.lineWidth = 1.5
      ctx.stroke()
    }
  }
}

function updateGraphVisibility (progress = null) {
  if (!graphInstance || !graphData.value) return

  const queryVisibleIds = filteredNodeIds.value
  const timelapseVisibleIds = timelapseActive.value
    ? getTimelapseVisibleIds(
      Number.isFinite(progress) ? progress : timelapseProgress.value,
      filteredNodes.value
    )
    : null

  graphInstance.forEachNode((id) => {
    const hiddenByQuery = !queryVisibleIds.has(id)
    const hiddenByTimelapse = timelapseVisibleIds ? !timelapseVisibleIds.has(id) : false
    graphInstance.setNodeAttribute(id, 'hidden', hiddenByQuery || hiddenByTimelapse)
  })
  graphInstance.forEachEdge((edge, _attrs, source, target) => {
    const hiddenByQuery = !queryVisibleIds.has(source) || !queryVisibleIds.has(target)
    const hiddenByTimelapse = timelapseVisibleIds
      ? !timelapseVisibleIds.has(source) || !timelapseVisibleIds.has(target)
      : false
    graphInstance.setEdgeAttribute(edge, 'hidden', hiddenByQuery || hiddenByTimelapse)
  })
}

function refreshGraphVisibility (progress = null) {
  if (!graphInstance || !renderer) return
  updateGraphVisibility(progress)
  renderer.refresh()
}

function scheduleGraphMount () {
  if (graphMountRaf) cancelAnimationFrame(graphMountRaf)
  graphMountRaf = requestAnimationFrame(() => {
    graphMountRaf = null
    mountSigma()
  })
}

function mountSigma () {
  const container = containerRef.value
  const data = graphData.value
  if (!container || !data || data.nodes.length === 0) return

  destroySigma()

  const graph = buildGraph(data)
  graphInstance = graph
  buildNeighbors(graph)

  hoveredNodeRef = null
  selectedNodeRef = null
  hoverAnimRef = 0
  selectAnimRef = 0
  hoverTargetRef = 0
  selectTargetRef = 0

  const bgDimRgb = isLight.value ? [245, 245, 245] : [30, 30, 30]

  let sigma
  try {
    sigma = new Sigma(graph, container, {
      renderLabels: false,
      labelFont: 'system-ui, -apple-system, sans-serif',
      defaultEdgeColor: '#333',
      defaultNodeColor: '#555',
      defaultEdgeType: 'curved',
      zIndex: true,
      edgeProgramClasses: { curved: EdgeCurveProgram },
      minCameraRatio: 0.05,
      maxCameraRatio: 8,
      stagePadding: 40,
      defaultDrawNodeHover: () => {},
      nodeReducer: (node, attrs) => {
        const sel = selectedNodeRef
        const hov = hoveredNodeRef
        const focusActive = sel && selectAnimRef > 0.1
        const hoverActive = hov && hoverAnimRef > 0.1
        const active = focusActive || hoverActive
        if (!active) {
          return { ...attrs, size: (attrs.size || 4) * nodeSizeScale.value }
        }
        if (node === sel) {
          return { ...attrs, size: (attrs.size || 4) * nodeSizeScale.value * (1 + 0.45 * selectAnimRef), zIndex: 3 }
        }
        if (node === hov) {
          return { ...attrs, size: (attrs.size || 4) * nodeSizeScale.value * (1 + 0.35 * hoverAnimRef), zIndex: 2 }
        }
        const selNeighbor = sel && neighborsMap.get(sel)?.has(node)
        const hovNeighbor = hov && neighborsMap.get(hov)?.has(node)
        if (selNeighbor || hovNeighbor) {
          return { ...attrs, size: (attrs.size || 4) * nodeSizeScale.value * 1.12, zIndex: 1 }
        }
        const dim = Math.max(selectAnimRef, hoverAnimRef)
        return {
          ...attrs,
          color: dimHex(attrs.color, bgDimRgb, dim * 0.88),
          size: (attrs.size || 4) * nodeSizeScale.value * (1 - 0.5 * dim)
        }
      },
      edgeReducer: (edge, attrs) => {
        const w = attrs.weight ?? 0.5
        const sel = selectedNodeRef
        const hov = hoveredNodeRef
        const g = graphInstance
        if (!g) return attrs
        const src = g.source(edge)
        const dst = g.target(edge)
        const baseSize = (0.25 + w * 0.7) * linkThickness.value
        const baseColor = edgeColorFor(attrs.edgeType) || attrs.color

        const touchesSel = sel && (src === sel || dst === sel)
        const touchesHov = hov && (src === hov || dst === hov)
        const focusActive = sel && selectAnimRef > 0.1
        const hoverActive = hov && hoverAnimRef > 0.1

        if (touchesSel && focusActive) {
          return { ...attrs, color: baseColor, size: baseSize + 0.6 * selectAnimRef, zIndex: 2 }
        }
        if (touchesHov && hoverActive) {
          return { ...attrs, color: baseColor, size: baseSize + 0.5 * hoverAnimRef, zIndex: 1 }
        }
        if (focusActive || hoverActive) {
          const dim = Math.max(selectAnimRef, hoverAnimRef)
          return {
            ...attrs,
            color: dimHex(baseColor, bgDimRgb, dim * 0.92),
            size: baseSize * (1 - 0.7 * dim)
          }
        }
        return { ...attrs, color: baseColor, size: baseSize }
      }
    })
  } catch (err) {
    console.error('AtomicGraphView: sigma init failed', err)
    return
  }

  renderer = sigma
  updateGraphVisibility()

  labelCanvas = document.createElement('canvas')
  labelCanvas.style.position = 'absolute'
  labelCanvas.style.inset = '0'
  labelCanvas.style.pointerEvents = 'none'
  labelCanvas.style.zIndex = '10'
  container.appendChild(labelCanvas)

  const drawLabelsBound = () => {
    if (renderer && graphInstance && containerRef.value) {
      drawLabels(renderer, graphInstance, containerRef.value)
    }
  }
  renderer.on('afterRender', drawLabelsBound)
  requestAnimationFrame(drawLabelsBound)

  let xMin = Infinity; let xMax = -Infinity; let yMin = Infinity; let yMax = -Infinity
  graph.forEachNode((id, attrs) => {
    if (attrs.x < xMin) xMin = attrs.x
    if (attrs.x > xMax) xMax = attrs.x
    if (attrs.y < yMin) yMin = attrs.y
    if (attrs.y > yMax) yMax = attrs.y
  })
  renderer.setCustomBBox({ x: [xMin, xMax], y: [yMin, yMax] })

  renderer.getCamera().on('updated', () => {
    const state = renderer.getCamera().getState()
    zoomValue.value = 1 / state.ratio
  })

  hoverRaf = null
  const tickHover = () => {
    const diff = hoverTargetRef - hoverAnimRef
    const diffSel = selectTargetRef - selectAnimRef
    if (Math.abs(diff) < 0.005 && Math.abs(diffSel) < 0.005) {
      hoverAnimRef = hoverTargetRef
      selectAnimRef = selectTargetRef
      if (hoverTargetRef === 0) hoveredNodeRef = null
      renderer.refresh()
      hoverRaf = null
      return
    }
    hoverAnimRef += diff * 0.22
    selectAnimRef += diffSel * 0.22
    renderer.refresh()
    hoverRaf = requestAnimationFrame(tickHover)
  }
  const startAnim = () => {
    if (hoverRaf !== null) return
    hoverRaf = requestAnimationFrame(tickHover)
  }

  renderer.on('enterNode', ({ node }) => {
    hoveredNodeRef = node
    hoverTargetRef = 1
    startAnim()
  })
  renderer.on('leaveNode', () => {
    hoverTargetRef = 0
    startAnim()
  })
  renderer.on('clickNode', ({ node }) => {
    const data = graph.getNodeAttribute(node, 'data')
    selectNode(data, node)
  })
  renderer.on('clickStage', () => {
    deselectNode()
  })

  renderer.refresh()
}

function destroySigma () {
  if (hoverRaf) {
    cancelAnimationFrame(hoverRaf)
    hoverRaf = null
  }
  if (renderer) {
    renderer.kill()
    renderer = null
  }
  if (labelCanvas && labelCanvas.parentNode) {
    labelCanvas.remove()
    labelCanvas = null
  }
  graphInstance = null
  neighborsMap = new Map()
}

function selectNode (data, nodeId) {
  selectedNodeRef = nodeId
  selectTargetRef = 1
  selectedNode.value = data
  cardCollapsed.value = false
  if (cardPos.x === null) positionCardNearNode(nodeId)
  if (renderer && graphInstance && graphInstance.hasNode(nodeId)) {
    const gx = graphInstance.getNodeAttribute(nodeId, 'x')
    const gy = graphInstance.getNodeAttribute(nodeId, 'y')
    let xMin = Infinity; let xMax = -Infinity; let yMin = Infinity; let yMax = -Infinity
    graphInstance.forEachNode((id, attrs) => {
      if (attrs.x < xMin) xMin = attrs.x
      if (attrs.x > xMax) xMax = attrs.x
      if (attrs.y < yMin) yMin = attrs.y
      if (attrs.y > yMax) yMax = attrs.y
    })
    const bboxW = xMax - xMin || 1
    const bboxH = yMax - yMin || 1
    const camX = (gx - xMin) / bboxW
    const camY = (gy - yMin) / bboxH
    renderer.getCamera().animate(
      { x: camX, y: camY, ratio: 0.4 },
      { duration: 520, easing: (k) => 1 - Math.pow(1 - k, 3) }
    )
  }
  if (renderer) renderer.refresh()
}

function positionCardNearNode (nodeId) {
  if (!renderer || !graphInstance || !containerRef.value) return
  if (!graphInstance.hasNode(nodeId)) return
  const attrs = graphInstance.getNodeAttributes(nodeId)
  if (typeof attrs.x !== 'number' || typeof attrs.y !== 'number') return
  const pos = renderer.graphToViewport({ x: attrs.x, y: attrs.y })
  const w = containerRef.value.clientWidth
  const cardW = 380
  let x = pos.x + 40
  if (x + cardW > w - 16) x = Math.max(16, pos.x - cardW - 40)
  const y = Math.max(16, pos.y - 60)
  cardPos.x = x
  cardPos.y = y
}

function deselectNode () {
  selectedNodeRef = null
  selectTargetRef = 0
  selectedNode.value = null
  cardPos.x = null
  cardPos.y = null
  cardCollapsed.value = false
  if (renderer) renderer.refresh()
}

function onStageClick (event) {
  if (event.target === containerRef.value) {
    deselectNode()
  }
}

function startCardDrag (event) {
  if (cardCollapsed.value) return
  cardDragging.value = true
  const startX = event.clientX
  const startY = event.clientY
  const startPX = cardPos.x || 0
  const startPY = cardPos.y || 0

  const onMove = (e) => {
    if (!cardDragging.value) return
    const w = containerRef.value?.clientWidth || 800
    const h = containerRef.value?.clientHeight || 600
    let nx = startPX + (e.clientX - startX)
    let ny = startPY + (e.clientY - startY)
    nx = Math.max(12, Math.min(w - 392, nx))
    ny = Math.max(12, Math.min(h - 80, ny))
    cardPos.x = nx
    cardPos.y = ny
  }
  const onUp = () => {
    cardDragging.value = false
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
  }
  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
}

function resetView () {
  if (!renderer) return
  renderer.getCamera().animatedReset({ duration: 500 })
}

function onZoomSlider (event) {
  const val = Number(event.target.value)
  zoomValue.value = val
  if (renderer) {
    renderer.getCamera().animate({ ratio: 1 / val }, { duration: 120 })
  }
}

function toggleSection (section) {
  section.open = !section.open
}

function resetOptions () {
  filterQuery.value = ''
  filterTags.value = true
  filterExisting.value = true
  filterOrphans.value = false
  showLabels.value = true
  showStats.value = true
  labelThreshold.value = 7
  nodeSizeScale.value = 1
  linkThickness.value = 1
  forceCenter.value = 0.32
  forceRepulsion.value = 420
  forceLink.value = 0.55
  forceLinkDistance.value = 95
  if (renderer) renderer.refresh()
}

function addGroup () {
  const idx = groups.value.length
  groups.value.push({
    id: `group-${Date.now()}`,
    name: `Groupe ${idx + 1}`,
    color: NODE_PALETTE[idx % NODE_PALETTE.length],
    count: 0
  })
}

function runForceSimulation (duration = 1500) {
  if (!graphInstance || !renderer) return
  animating.value = true
  const start = performance.now()
  const nodes = graphInstance.nodes()
  if (nodes.length === 0) {
    animating.value = false
    return
  }

  const tick = (now) => {
    const elapsed = now - start
    const progress = Math.min(1, elapsed / duration)
    const damping = 1 - progress * 0.7
    const iterations = 3

    for (let iter = 0; iter < iterations; iter++) {
      const forces = new Map(nodes.map((n) => [n, { fx: 0, fy: 0 }]))

      for (let i = 0; i < nodes.length; i++) {
        const a = graphInstance.getNodeAttributes(nodes[i])
        for (let j = i + 1; j < nodes.length; j++) {
          const b = graphInstance.getNodeAttributes(nodes[j])
          let dx = a.x - b.x
          let dy = a.y - b.y
          let dist = Math.sqrt(dx * dx + dy * dy) || 1
          if (dist < 1) { dx = Math.random() - 0.5; dy = Math.random() - 0.5; dist = 1 }
          const f = (forceRepulsion.value * damping) / (dist * dist)
          forces.get(nodes[i]).fx += (dx / dist) * f
          forces.get(nodes[i]).fy += (dy / dist) * f
          forces.get(nodes[j]).fx -= (dx / dist) * f
          forces.get(nodes[j]).fy -= (dy / dist) * f
        }
      }

      graphInstance.forEachEdge((edge, _attrs, s, t) => {
        const a = graphInstance.getNodeAttributes(s)
        const b = graphInstance.getNodeAttributes(t)
        const dx = b.x - a.x
        const dy = b.y - a.y
        const dist = Math.sqrt(dx * dx + dy * dy) || 1
        const f = forceLink.value * (dist - forceLinkDistance.value) * damping
        forces.get(s).fx += (dx / dist) * f
        forces.get(s).fy += (dy / dist) * f
        forces.get(t).fx -= (dx / dist) * f
        forces.get(t).fy -= (dy / dist) * f
      })

      for (const n of nodes) {
        const a = graphInstance.getNodeAttributes(n)
        forces.get(n).fx -= a.x * forceCenter.value * damping
        forces.get(n).fy -= a.y * forceCenter.value * damping
      }

      for (const n of nodes) {
        const a = graphInstance.getNodeAttributes(n)
        const f = forces.get(n)
        a.x += f.fx * 0.08 * damping
        a.y += f.fy * 0.08 * damping
      }
    }

    renderer.refresh()
    if (progress < 1) {
      simRaf = requestAnimationFrame(tick)
    } else {
      animating.value = false
      simRaf = null
      const positions = {}
      graphInstance.forEachNode((id, attrs) => {
        positions[id] = { x: attrs.x, y: attrs.y }
      })
      canvasStore.persistPositions(store.activeVaultId || 'default', positions)
    }
  }
  simRaf = requestAnimationFrame(tick)
}

function animateGraph () {
  runForceSimulation(1500)
}

function toggleTimelapse () {
  if (timelapseActive.value) {
    stopTimelapse()
    return
  }
  timelapseActive.value = true
  timelapsePlaying.value = true
  timelapseProgress.value = 0
  updateTimelapseVisibility(0)
  runTimelapse()
}

function stopTimelapse () {
  timelapseActive.value = false
  timelapsePlaying.value = false
  if (timelapseRaf) {
    cancelAnimationFrame(timelapseRaf)
    timelapseRaf = null
  }
  refreshGraphVisibility()
}

function runTimelapse () {
  if (!timelapseActive.value) return
  const start = performance.now()
  const startProgress = timelapseProgress.value
  const speed = timelapseSpeed.value
  const duration = (100 - startProgress) * 120 / speed

  const tick = (now) => {
    if (!timelapseActive.value || !timelapsePlaying.value) return
    const elapsed = now - start
    const progress = Math.min(100, startProgress + (elapsed / duration) * 100)
    timelapseProgress.value = progress
    updateTimelapseVisibility(progress)
    if (progress >= 100) {
      timelapsePlaying.value = false
      timelapseRaf = null
      return
    }
    timelapseRaf = requestAnimationFrame(tick)
  }
  timelapseRaf = requestAnimationFrame(tick)
}

function updateTimelapseVisibility (progress) {
  refreshGraphVisibility(progress)
}

function onTimelapseSeek (event) {
  timelapseProgress.value = Number(event.target.value)
  updateTimelapseVisibility(timelapseProgress.value)
}

function cycleTimelapseSpeed () {
  const speeds = [1, 2, 4]
  const idx = speeds.indexOf(timelapseSpeed.value)
  timelapseSpeed.value = speeds[(idx + 1) % speeds.length]
  if (timelapsePlaying.value) {
    if (timelapseRaf) cancelAnimationFrame(timelapseRaf)
    runTimelapse()
  }
}

function openSelectedNode () {
  if (!selectedNode.value) return
  const notePath = selectedNode.value.relativePath || selectedNode.value.path || selectedNode.value.id
  if (!notePath) return
  const note = [...store.entries, ...store.rootEntries, ...store.openedNotes]
    .find((entry) => entry?.path === notePath)
  if (note) {
    store.openNote(note)
    return
  }
  store.openNote({
    kind: 'note',
    type: 'note',
    path: notePath,
    title: selectedNode.value.title,
    updatedAt: selectedNode.value.updatedAt
  })
}

function loadGraphData () {
  const model = semanticModel.value
  graphData.value = {
    nodes: model.nodes,
    edges: renderEdges.value,
    edgeCounts: model.edgeCounts,
    maxEdges: model.maxEdges,
    clusters: model.clusters || []
  }
}

async function rebuildIndex () {
  indexBuilding.value = true
  try {
    await searchStore.rebuild()
    await searchStore.inspect()
  } catch (err) {
    console.error('rebuildIndex failed', err)
  } finally {
    indexBuilding.value = false
  }
}

async function ensureGraphData () {
  const activeVaultPath = store.activeVault?.path || ''
  const graphVaultPath = searchStore.vaultPath || ''
  const hasFreshGraph = indexReady.value && graphVaultPath === activeVaultPath

  if (!hasFreshGraph && graphVaultPath !== activeVaultPath) {
    await searchStore.inspect()
  }

  if (!indexReady.value && store.rootEntries?.length > 0) {
    loadGraphData()
    scheduleGraphMount()
    rebuildIndex().catch(() => {})
  } else {
    loadGraphData()
    scheduleGraphMount()
  }
}

watch(() => searchStore.indexInspection?.graph, () => {
  loadGraphData()
  scheduleGraphMount()
})

watch(() => store.activeVault?.path, () => {
  ensureGraphData()
})

watch(() => store.rootEntries, () => {
  if (!indexReady.value) {
    loadGraphData()
    scheduleGraphMount()
  }
}, { deep: false })

watch(filterQuery, () => {
  refreshGraphVisibility()
})

watch([nodeSizeScale, linkThickness, labelThreshold, showLabels], () => {
  if (renderer) renderer.refresh()
})

watch(theme, () => {
  if (!graphInstance || !renderer) return
  const t = theme.value
  const { edgeCounts, maxEdges } = graphData.value || {}
  graphInstance.forEachNode((node, attrs) => {
    const connectivity = (edgeCounts?.get(node) || 0) / (maxEdges || 1)
    graphInstance.setNodeAttribute(node, 'color', nodeColor(t, Math.min(1, 0.3 + connectivity), attrs.clusterIndex))
  })
  renderer.refresh()
})

onMounted(() => {
  canvasStore.loadPositions(store.activeVaultId || 'default')
  ensureGraphData()
})

onBeforeUnmount(() => {
  if (simRaf) cancelAnimationFrame(simRaf)
  if (timelapseRaf) cancelAnimationFrame(timelapseRaf)
  if (graphMountRaf) cancelAnimationFrame(graphMountRaf)
  destroySigma()
})
</script>

<style scoped>
.en-graph-premium {
  position: relative;
  min-height: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--en-bg);
}

.en-graph-stage {
  position: absolute;
  inset: 0;
  overflow: hidden;
  cursor: grab;
  background:
    radial-gradient(circle at center,
      color-mix(in srgb, var(--en-surface) 80%, var(--en-bg)) 0%,
      var(--en-bg) 65%,
      color-mix(in srgb, var(--en-bg) 90%, black) 100%);
}

.en-graph-stage:active {
  cursor: grabbing;
}

.en-graph-status {
  position: absolute;
  left: 50%;
  top: 14px;
  transform: translateX(-50%);
  padding: 6px 14px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--en-surface) 90%, transparent);
  border: 1px solid var(--en-border);
  color: var(--en-muted);
  font-size: 12px;
  backdrop-filter: blur(12px);
  z-index: 15;
  pointer-events: none;
}

.en-graph-floating-icons {
  position: absolute;
  top: 14px;
  right: 22px;
  display: flex;
  gap: 8px;
  z-index: 20;
}

.en-graph-floating-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  border: 1px solid var(--en-border);
  background: color-mix(in srgb, var(--en-surface) 92%, transparent);
  color: var(--en-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 0.16s, color 0.16s, border-color 0.16s;
  backdrop-filter: blur(10px);
}

.en-graph-floating-icon:hover {
  background: var(--en-soft);
  color: var(--en-text);
  border-color: color-mix(in srgb, var(--en-primary) 50%, var(--en-border));
}

.en-graph-floating-icon.active {
  background: color-mix(in srgb, var(--en-primary) 18%, var(--en-surface));
  color: var(--en-primary);
  border-color: color-mix(in srgb, var(--en-primary) 60%, transparent);
}

.en-fi-svg {
  width: 18px;
  height: 18px;
}

.en-graph-bottom-left {
  position: absolute;
  left: 22px;
  bottom: 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: 20;
}

.en-graph-stats {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--en-surface) 88%, transparent);
  border: 1px solid var(--en-border);
  color: var(--en-muted);
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  backdrop-filter: blur(10px);
}

.en-graph-stats-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #4ade80;
  flex-shrink: 0;
}

.en-graph-stats-dot-building {
  background: #eab308;
  animation: en-pulse 1s ease-in-out infinite;
}

@keyframes en-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.en-graph-zoom-control {
  display: flex;
  align-items: center;
  gap: 10px;
}

.en-graph-zoom-button {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  border: none;
  background: var(--en-primary);
  color: #ffffff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 0 12px color-mix(in srgb, var(--en-primary) 45%, transparent);
  transition: transform 0.15s, box-shadow 0.15s;
}

.en-graph-zoom-button:hover {
  transform: scale(1.08);
  box-shadow: 0 0 16px color-mix(in srgb, var(--en-primary) 60%, transparent);
}

.en-gz-svg {
  width: 15px;
  height: 15px;
}

.en-graph-zoom-slider {
  width: 100px;
  height: 4px;
  appearance: none;
  background: var(--en-border);
  border-radius: 999px;
  cursor: pointer;
}

.en-graph-zoom-slider::-webkit-slider-thumb {
  appearance: none;
  width: 13px;
  height: 13px;
  border-radius: 50%;
  background: var(--en-text);
  cursor: pointer;
}

.en-graph-zoom-slider::-moz-range-thumb {
  width: 13px;
  height: 13px;
  border-radius: 50%;
  background: var(--en-text);
  border: none;
  cursor: pointer;
}

.en-graph-zoom-pct {
  color: var(--en-muted);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  min-width: 36px;
}

.en-graph-settings-panel {
  position: absolute;
  top: 62px;
  right: 70px;
  width: 340px;
  max-height: calc(100% - 90px);
  display: flex;
  flex-direction: column;
  background: var(--en-surface);
  border: 1px solid var(--en-border);
  border-radius: 16px;
  box-shadow: 0 18px 48px rgba(0, 0, 0, 0.35);
  overflow: hidden;
  color: var(--en-text);
  z-index: 25;
}

.en-panel-topbar {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 18px;
  border-bottom: 1px solid var(--en-border);
  flex-shrink: 0;
}

.en-panel-topbar-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--en-text);
}

.en-panel-topbar-actions {
  display: flex;
  gap: 4px;
}

.en-panel-icon-btn {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  border: none;
  background: transparent;
  color: var(--en-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 0.14s, color 0.14s;
}

.en-panel-icon-btn:hover {
  background: var(--en-soft);
  color: var(--en-text);
}

.en-panel-icn {
  width: 16px;
  height: 16px;
}

.en-panel-scroll {
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

.en-panel-scroll::-webkit-scrollbar {
  width: 6px;
}

.en-panel-scroll::-webkit-scrollbar-thumb {
  background: var(--en-border);
  border-radius: 999px;
}

.en-panel-section {
  border-bottom: 1px solid var(--en-border);
}

.en-panel-section:last-child {
  border-bottom: none;
}

.en-section-header {
  width: 100%;
  height: 48px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 18px;
  border: none;
  background: transparent;
  color: var(--en-text);
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: background 0.14s;
  text-align: left;
}

.en-section-header:hover {
  background: color-mix(in srgb, var(--en-soft) 50%, transparent);
}

.en-section-chevron {
  width: 17px;
  height: 17px;
  color: var(--en-muted);
  transition: transform 0.22s ease;
  transform: rotate(-90deg);
  flex-shrink: 0;
}

.en-section-header.open .en-section-chevron {
  transform: rotate(0deg);
}

.en-section-title {
  flex: 1;
}

.en-section-body {
  padding: 6px 18px 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.en-filter-search {
  position: relative;
  margin-bottom: 2px;
}

.en-filter-search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: var(--en-muted);
  pointer-events: none;
}

.en-filter-input {
  width: 100%;
  height: 40px;
  border-radius: 9px;
  background: var(--en-soft);
  border: 1px solid var(--en-border);
  color: var(--en-text);
  font-size: 13px;
  padding: 0 12px 0 36px;
  outline: none;
  transition: border-color 0.14s;
}

.en-filter-input::placeholder {
  color: var(--en-muted);
}

.en-filter-input:focus {
  border-color: color-mix(in srgb, var(--en-primary) 60%, var(--en-border));
}

.en-filter-row {
  min-height: 42px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 13px;
  font-weight: 600;
  color: var(--en-text);
}

.en-switch {
  width: 42px;
  height: 24px;
  border-radius: 999px;
  border: none;
  background: var(--en-border);
  padding: 3px;
  cursor: pointer;
  transition: background 0.16s ease;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.en-switch-thumb {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--en-text);
  transition: transform 0.16s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
}

.en-switch.active {
  background: var(--en-primary);
}

.en-switch.active .en-switch-thumb {
  transform: translateX(18px);
}

.en-slider-control {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.en-slider-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--en-text);
}

.en-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 5px;
  border-radius: 999px;
  background: var(--en-border);
  outline: none;
  cursor: pointer;
}

.en-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 15px;
  border-radius: 999px;
  background: var(--en-text);
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35);
  transition: transform 0.12s;
}

.en-slider::-webkit-slider-thumb:hover {
  transform: scale(1.08);
}

.en-slider::-moz-range-thumb {
  width: 20px;
  height: 15px;
  border-radius: 999px;
  background: var(--en-text);
  border: none;
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35);
}

.en-primary-button {
  width: 100%;
  height: 44px;
  border-radius: 9px;
  border: none;
  background: var(--en-primary);
  color: #ffffff;
  font-size: 14px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  cursor: pointer;
  transition: opacity 0.16s, transform 0.12s;
}

.en-primary-button:hover:not(:disabled) {
  opacity: 0.9;
}

.en-primary-button:active:not(:disabled) {
  transform: translateY(1px);
}

.en-primary-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.en-btn-icn {
  width: 17px;
  height: 17px;
}

.en-spin {
  animation: en-spin 0.8s linear infinite;
}

@keyframes en-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.en-group-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}

.en-group-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 11px;
  border-radius: 9px;
  background: var(--en-soft);
  font-size: 13px;
}

.en-group-dot {
  width: 11px;
  height: 11px;
  border-radius: 50%;
  flex-shrink: 0;
}

.en-group-name {
  flex: 1;
  color: var(--en-text);
}

.en-group-count {
  color: var(--en-muted);
  font-size: 12px;
}

.en-timeline-panel {
  position: absolute;
  left: 50%;
  bottom: 20px;
  transform: translateX(-50%);
  height: 44px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 12px;
  background: color-mix(in srgb, var(--en-surface) 94%, transparent);
  border: 1px solid var(--en-border);
  border-radius: 12px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(12px);
  z-index: 22;
}

.en-timeline-play,
.en-timeline-close {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  border: none;
  background: var(--en-primary);
  color: #ffffff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  transition: opacity 0.14s;
}

.en-timeline-play:hover {
  opacity: 0.88;
}

.en-timeline-close {
  background: var(--en-soft);
  color: var(--en-muted);
}

.en-timeline-close:hover {
  color: var(--en-text);
}

.en-tl-icn {
  width: 15px;
  height: 15px;
}

.en-timeline-slider {
  flex: 1;
  min-width: 200px;
  height: 5px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--en-border);
  border-radius: 999px;
  cursor: pointer;
}

.en-timeline-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 15px;
  height: 15px;
  border-radius: 50%;
  background: var(--en-primary);
  cursor: pointer;
  box-shadow: 0 0 8px color-mix(in srgb, var(--en-primary) 50%, transparent);
}

.en-timeline-slider::-moz-range-thumb {
  width: 15px;
  height: 15px;
  border-radius: 50%;
  background: var(--en-primary);
  border: none;
  cursor: pointer;
}

.en-timeline-date {
  color: var(--en-text);
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  min-width: 100px;
}

.en-timeline-speed {
  padding: 4px 10px;
  border-radius: 7px;
  border: 1px solid var(--en-border);
  background: var(--en-soft);
  color: var(--en-text);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: background 0.14s;
}

.en-timeline-speed:hover {
  background: var(--en-bg);
}

.en-note-preview-card {
  position: absolute;
  width: 360px;
  max-height: 440px;
  display: flex;
  flex-direction: column;
  background: color-mix(in srgb, var(--en-surface) 97%, transparent);
  border: 1px solid var(--en-border);
  border-radius: 12px;
  box-shadow: 0 18px 46px rgba(0, 0, 0, 0.4);
  color: var(--en-text);
  overflow: hidden;
  z-index: 30;
  transition: opacity 0.18s ease, transform 0.18s ease, max-height 0.22s ease;
}

.en-note-preview-card.collapsed {
  max-height: 52px;
}

.en-note-preview-card.dragging {
  cursor: grabbing;
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.55);
}

.en-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 14px;
  cursor: grab;
  flex-shrink: 0;
}

.en-card-header:active {
  cursor: grabbing;
}

.en-card-title {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--en-text);
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.en-card-header-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.en-card-mini-btn {
  width: 26px;
  height: 26px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--en-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 0.14s, color 0.14s;
}

.en-card-mini-btn:hover {
  background: var(--en-soft);
  color: var(--en-text);
}

.en-card-icn {
  width: 14px;
  height: 14px;
}

.en-card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 0 14px;
  color: var(--en-muted);
  font-size: 12px;
}

.en-card-summary {
  margin: 10px 0 0;
  padding: 0 14px;
  font-size: 13px;
  line-height: 1.5;
  color: color-mix(in srgb, var(--en-text) 80%, transparent);
  max-height: 130px;
  overflow-y: auto;
}

.en-card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 10px 14px 0;
}

.en-card-tag {
  padding: 3px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--en-primary) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--en-primary) 30%, transparent);
  color: var(--en-primary);
  font-size: 11px;
  font-weight: 600;
}

.en-card-open {
  margin: 12px 0 0;
  height: 46px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 0 14px;
  border: none;
  border-top: 1px solid var(--en-border);
  background: transparent;
  color: var(--en-primary);
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: background 0.14s, color 0.14s;
  flex-shrink: 0;
}

.en-card-open:hover {
  background: color-mix(in srgb, var(--en-primary) 10%, transparent);
}

.en-card-open-icn {
  width: 15px;
  height: 15px;
}

.en-panel-enter-active,
.en-panel-leave-active {
  transition: opacity 0.22s ease, transform 0.22s ease;
}

.en-panel-enter-from,
.en-panel-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.en-accordion-enter-active,
.en-accordion-leave-active {
  transition: opacity 0.18s ease;
  overflow: hidden;
}

.en-accordion-enter-from,
.en-accordion-leave-to {
  opacity: 0;
}

.en-timeline-enter-active,
.en-timeline-leave-active {
  transition: opacity 0.24s ease, transform 0.24s ease;
}

.en-timeline-enter-from,
.en-timeline-leave-to {
  opacity: 0;
  transform: translate(-50%, 20px);
}

.en-card-enter-active,
.en-card-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.en-card-enter-from,
.en-card-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.96);
}
</style>
