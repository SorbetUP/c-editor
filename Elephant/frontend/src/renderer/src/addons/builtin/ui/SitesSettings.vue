<template>
  <section class="en-settings-group">
    <div class="en-settings-row">
      <div class="en-settings-row-copy">
        <strong>Site preview</strong>
        <span>{{ siteStatusLabel }}</span>
      </div>
      <button
        class="en-switch"
        type="button"
        role="switch"
        aria-label="Enable site preview"
        :aria-checked="featureEnabled"
        :class="{ active: featureEnabled }"
        :disabled="busy"
        @click="toggleFeature"
      ><span /></button>
    </div>

    <div class="en-settings-inline-actions">
      <button type="button" :disabled="!sitePreviewStore.previewUrl || busy" @click="sitePreviewStore.openPreviewExternal">
        <Globe2 aria-hidden="true" />Open preview
      </button>
      <button type="button" :disabled="!sitePreviewStore.info || busy" @click="stopSitePreview">Stop preview</button>
      <span v-if="message">{{ message }}</span>
    </div>
  </section>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { Globe2 } from '@lucide/vue'
import { elephantnoteClient } from 'elephant-front/services/elephantnoteClient'
import { useSitePreviewStore } from 'elephant-front/sitePreview/sitePreviewStore'

const sitePreviewStore = useSitePreviewStore()
const featureEnabled = ref(false)
const busy = ref(false)
const message = ref('')
const siteStatusLabel = computed(() => sitePreviewStore.previewUrl
  ? 'Preview running'
  : sitePreviewStore.lastBuild?.outputDir
    ? 'Static build ready'
    : 'No generated site active')

const loadFeature = async () => {
  const flags = await elephantnoteClient.features.get()
  featureEnabled.value = flags?.sitePreview === true
}

const toggleFeature = async () => {
  busy.value = true
  message.value = ''
  try {
    const flags = await elephantnoteClient.features.set('sitePreview', !featureEnabled.value)
    featureEnabled.value = flags?.sitePreview === true
  } catch (error) {
    message.value = error instanceof Error ? error.message : 'Unable to update site preview.'
  } finally {
    busy.value = false
  }
}

const stopSitePreview = async () => {
  busy.value = true
  message.value = ''
  try {
    await sitePreviewStore.stopPreview()
    sitePreviewStore.clear()
  } catch (error) {
    message.value = error instanceof Error ? error.message : 'Unable to stop site preview.'
  } finally {
    busy.value = false
  }
}

onMounted(async () => {
  await Promise.allSettled([loadFeature(), sitePreviewStore.refresh?.()])
})
</script>
