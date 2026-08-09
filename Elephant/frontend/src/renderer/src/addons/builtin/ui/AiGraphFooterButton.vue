<template>
  <button type="button" title="Open graph" @click="openGraph">
    Graph
  </button>
</template>

<script setup>
const openGraph = () => {
  globalThis.dispatchEvent?.(new CustomEvent('elephantnote:open-addon-view', {
    detail: { viewId: 'elephant.ai.graph' }
  }))
}
</script>
