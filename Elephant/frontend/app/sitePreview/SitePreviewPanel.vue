<template>
  <div
    v-if="isVisible"
    class="en-site-preview-panel"
  >
    <site-preview-toolbar
      :title="statusText"
      :url="previewStore.previewUrl"
      @open-external="previewStore.openPreviewExternal"
      @close="closePreview"
    />
    <div
      v-if="previewStore.error || previewStore.lastBuild?.outputDir"
      class="en-site-preview-body"
    >
      <p
        v-if="previewStore.error"
        class="en-site-preview-error"
      >
        {{ previewStore.error }}
      </p>
      <p
        v-if="previewStore.lastBuild?.outputDir"
        class="en-site-preview-build"
      >
        Static website built at {{ previewStore.lastBuild.outputDir }}
        <button
          type="button"
          @click="previewStore.openBuildExternal"
        >
          Open folder
        </button>
      </p>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useSitePreviewStore } from './sitePreviewStore'
import SitePreviewToolbar from './SitePreviewToolbar.vue'

const previewStore = useSitePreviewStore()
const isVisible = computed(() => previewStore.status !== 'idle' || !!previewStore.info || !!previewStore.lastBuild)
const statusText = computed(() => {
  if (previewStore.status === 'preparing') return 'Preparing website preview...'
  if (previewStore.status === 'building') return 'Building website...'
  if (previewStore.status === 'error') return 'Website preview failed.'
  return 'Website preview'
})

const closePreview = async () => {
  await previewStore.stopPreview()
  previewStore.clear()
}
</script>

<style scoped>
.en-site-preview-panel {
  position: absolute;
  left: 18px;
  right: 18px;
  bottom: 18px;
  z-index: 20;
  display: grid;
  gap: 10px;
  border: 1px solid color-mix(in srgb, var(--en-border-strong) 70%, transparent);
  border-radius: 8px;
  padding: 10px;
  color: var(--en-text);
  background: color-mix(in srgb, var(--en-surface) 94%, transparent);
  box-shadow: 0 18px 44px rgba(0, 0, 0, 0.22);
}

.en-site-preview-body {
  display: grid;
  gap: 8px;
}

.en-site-preview-error,
.en-site-preview-build {
  margin: 0;
  color: var(--en-muted);
  font-size: 13px;
  line-height: 1.4;
}

.en-site-preview-error {
  color: var(--en-danger, #ff6b7a);
}

.en-site-preview-build {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  overflow-wrap: anywhere;
}

.en-site-preview-build button {
  min-height: 30px;
  flex: 0 0 auto;
  border: 1px solid var(--en-border);
  border-radius: 8px;
  padding: 0 10px;
  color: var(--en-text);
  background: transparent;
  font: inherit;
  font-size: 13px;
}
</style>
