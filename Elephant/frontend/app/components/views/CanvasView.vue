<template>
  <section class="en-canvas-view">
    <header class="en-canvas-header">
      <div>
        <h1>Semantic Canvas</h1>
        <p>
          {{ graphData.nodes.length }} nodes, {{ graphData.edges.length }} links
          <span v-if="graphData.clusters.length">, {{ graphData.clusters.length }} clusters</span>
        </p>
      </div>
      <div class="en-canvas-controls">
        <button
          type="button"
          @click="scale = Math.max(0.5, scale - 0.1)"
        >
          -
        </button>
        <span>{{ Math.round(scale * 100) }}%</span>
        <button
          type="button"
          @click="scale = Math.min(1.8, scale + 0.1)"
        >
          +
        </button>
      </div>
    </header>
    <div class="en-canvas-stage">
      <svg class="en-canvas-edges">
        <line
          v-for="edge in positionedEdges"
          :key="edge.id"
          :x1="edge.x1"
          :y1="edge.y1"
          :x2="edge.x2"
          :y2="edge.y2"
          :data-type="edge.type"
        />
      </svg>
      <div
        class="en-canvas-plane"
        :style="{ transform: `scale(${scale})` }"
      >
        <button
          v-for="node in positionedNodes"
          :key="node.id"
          class="en-canvas-node"
          :class="node.kind"
          type="button"
          :style="{ left: `${node.x}px`, top: `${node.y}px` }"
          @pointerdown="startDrag($event, node)"
          @dblclick="openNode(node)"
        >
          <strong>{{ node.title }}</strong>
          <span>{{ node.summary || semanticLabel(node) }}</span>
        </button>
      </div>
    </div>
  </section>
</template>

<script setup>
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { useVaultStore } from '../../stores/vaultStore'
import { useSearchStore } from '../../stores/searchStore'
import { buildSemanticViewModel, selectSemanticGraphSource } from './semanticGraphViewHelpers'

const store = useVaultStore()
const searchStore = useSearchStore()
const scale = ref(1)
const positions = ref({})
const dragging = ref(null)

const storageKey = computed(() => `elephantnote:canvas:${store.activeVaultId || 'default'}`)
const graphData = computed(() => buildSemanticViewModel({
  graph: selectSemanticGraphSource({
    inspectionGraph: searchStore.indexInspection?.graph
  }),
  savedPositions: positions.value,
  width: 1800,
  height: 1200
}))
const positionedNodes = computed(() => graphData.value.nodes)
const positionedEdges = computed(() => {
  const byId = new Map(positionedNodes.value.map((node) => [node.id, node]))
  return graphData.value.edges
    .map((edge) => {
      const source = byId.get(edge.source)
      const target = byId.get(edge.target)
      if (!source || !target) return null
      return {
        id: `${edge.source}->${edge.target}`,
        x1: source.x + 88,
        y1: source.y + 40,
        x2: target.x + 88,
        y2: target.y + 40,
        type: edge.type || 'semantic'
      }
    })
    .filter(Boolean)
})

const semanticLabel = (node) => {
  const tags = Array.isArray(node.tags) ? node.tags : []
  if (tags.length) return tags.slice(0, 3).map((tag) => `#${tag}`).join(' ')
  if ((node.kind || node.type) === 'folder') return 'Cluster node'
  return 'Semantic note'
}

const persist = () => {
  window.localStorage.setItem(storageKey.value, JSON.stringify(positions.value))
}

const loadPositions = () => {
  try {
    positions.value = JSON.parse(window.localStorage.getItem(storageKey.value) || '{}')
  } catch {
    positions.value = {}
  }
}

const startDrag = (event, node) => {
  event.currentTarget.setPointerCapture?.(event.pointerId)
  dragging.value = {
    id: node.id,
    startX: event.clientX,
    startY: event.clientY,
    x: node.x,
    y: node.y
  }
}

const moveDrag = (event) => {
  if (!dragging.value) return
  const next = {
    x: dragging.value.x + (event.clientX - dragging.value.startX) / scale.value,
    y: dragging.value.y + (event.clientY - dragging.value.startY) / scale.value
  }
  positions.value = {
    ...positions.value,
    [dragging.value.id]: next
  }
}

const stopDrag = () => {
  if (!dragging.value) return
  dragging.value = null
  persist()
}

const openNode = (node) => {
  if ((node.kind || node.type) !== 'note') return
  const note = [...store.rootEntries, ...store.entries, ...store.openedNotes].find((entry) => entry.path === node.id)
  if (note) store.openNote(note)
}

watch(storageKey, loadPositions)
watch(() => store.activeVault?.path, () => {
  searchStore.inspect().catch(() => {})
})

onMounted(() => {
  searchStore.inspect().catch(() => {})
  loadPositions()
  window.addEventListener('pointermove', moveDrag)
  window.addEventListener('pointerup', stopDrag)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', moveDrag)
  window.removeEventListener('pointerup', stopDrag)
})
</script>

<style scoped>
.en-canvas-view {
  min-height: 0;
  flex: 1;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
}

.en-canvas-header {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 16px;
  padding: 6px 28px 12px;
}

.en-canvas-header h1 {
  margin: 0;
  font-size: 28px;
}

.en-canvas-header p {
  margin: 4px 0 0;
  color: var(--en-muted);
  font-size: 13px;
}

.en-canvas-controls {
  display: inline-flex;
  gap: 8px;
  align-items: center;
}

.en-canvas-controls button {
  width: 34px;
  height: 34px;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  color: var(--en-text);
  background: var(--en-bg);
}

.en-canvas-stage {
  position: relative;
  min-height: 0;
  margin: 0 28px 28px;
  border: 1px solid var(--en-border);
  border-radius: 14px;
  background:
    radial-gradient(circle at 20% 20%, color-mix(in srgb, var(--en-primary) 10%, transparent), transparent 34%),
    linear-gradient(var(--en-border) 1px, transparent 1px),
    linear-gradient(90deg, var(--en-border) 1px, transparent 1px);
  background-size: auto, 32px 32px, 32px 32px;
  overflow: auto;
}

.en-canvas-plane,
.en-canvas-edges {
  position: absolute;
  inset: 0;
  width: 1800px;
  height: 1200px;
  transform-origin: 0 0;
}

.en-canvas-edges {
  pointer-events: none;
}

.en-canvas-edges line {
  stroke: color-mix(in srgb, var(--en-muted) 45%, transparent);
  stroke-width: 2;
  stroke-linecap: round;
}

.en-canvas-edges line[data-type="folder"] {
  stroke-width: 1.5;
  stroke-dasharray: 4 7;
}

.en-canvas-edges line[data-type="semantic"] {
  stroke-width: 2.5;
  stroke-opacity: 0.85;
}

.en-canvas-node {
  position: absolute;
  width: 200px;
  min-height: 88px;
  display: grid;
  gap: 8px;
  border: 1px solid var(--en-border);
  border-radius: 14px;
  padding: 14px 15px;
  color: var(--en-text);
  background: color-mix(in srgb, var(--en-bg) 88%, transparent);
  text-align: left;
  box-shadow: 0 16px 36px color-mix(in srgb, #020617 16%, transparent);
  backdrop-filter: blur(14px);
  touch-action: none;
}

.en-canvas-node.folder {
  background: color-mix(in srgb, var(--en-primary) 10%, var(--en-bg));
}

.en-canvas-node strong {
  font-size: 14px;
}

.en-canvas-node span {
  color: var(--en-muted);
  font-size: 12px;
  line-height: 1.4;
}
</style>
