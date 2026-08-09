<template>
  <div v-if="!view" class="en-addon-router-empty">
    <h2>Addon view unavailable</h2>
    <p>The addon may have been disabled, removed or uninstalled.</p>
    <button type="button" @click="emit('close')">Back to notes</button>
  </div>
  <component
    :is="view.contribution.component"
    v-else-if="view.contribution.component"
    :view="view"
    @close="emit('close')"
  />
  <addon-workspace-host
    v-else-if="view.contribution.kind === 'task-manager-v1'"
    :view-id="viewId"
    @close="emit('close')"
  />
  <div v-else class="en-addon-router-empty">
    <h2>Unsupported addon view</h2>
    <p>The enabled addon did not provide a workspace component for <code>{{ view.contribution.kind || view.contribution.id }}</code>.</p>
    <button type="button" @click="emit('close')">Back to notes</button>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useAddonsStore } from '@/store/addons'
import AddonWorkspaceHost from './AddonWorkspaceHost.vue'

const props = defineProps({ viewId: { type: String, required: true } })
const emit = defineEmits(['close'])
const addonsStore = useAddonsStore()
const view = computed(() => addonsStore.getContributions('views')
  .find((entry) => entry?.contribution?.id === props.viewId) || null)
</script>

<style scoped>
.en-addon-router-empty { min-height: 0; flex: 1; display: grid; place-content: center; justify-items: center; gap: 8px; padding: 32px; color: var(--en-muted); text-align: center; }
.en-addon-router-empty h2, .en-addon-router-empty p { margin: 0; }
.en-addon-router-empty h2 { color: var(--en-text); font-size: 18px; }
.en-addon-router-empty p { max-width: 480px; font-size: 12px; }
.en-addon-router-empty button { margin-top: 8px; min-height: 32px; padding: 0 12px; border: 1px solid var(--en-border); border-radius: 8px; color: var(--en-text); background: var(--en-surface); cursor: pointer; }
</style>
