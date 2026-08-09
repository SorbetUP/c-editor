<template>
  <div class="en-nav-bar">
    <button
      type="button"
      class="en-nav-btn"
      :aria-disabled="!nav.canGoBack"
      :class="{ disabled: !nav.canGoBack }"
      title="Retour"
      @mousedown.stop.prevent
      @pointerdown.stop
      @mouseup.stop
      @click.stop="goBack"
    >
      <ChevronLeft class="en-nav-icon" />
    </button>
    <button
      type="button"
      class="en-nav-btn"
      :aria-disabled="!nav.canGoForward"
      :class="{ disabled: !nav.canGoForward }"
      title="Avancer"
      @mousedown.stop.prevent
      @pointerdown.stop
      @mouseup.stop
      @click.stop="goForward"
    >
      <ChevronRight class="en-nav-icon" />
    </button>
    <component
      :is="entry.contribution.component"
      v-for="entry in topBarItems"
      :key="entry.contribution.id"
    />
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { ChevronLeft, ChevronRight } from '@lucide/vue'
import { useAddonsStore } from '@/store/addons'
import { useNavigationStore } from '../../stores/navigationStore'
import { useVaultStore } from '../../stores/vaultStore'

const nav = useNavigationStore()
const vaultStore = useVaultStore()
const addonsStore = useAddonsStore()
const topBarItems = computed(() => addonsStore.getContributions('top-bar.items')
  .filter((entry) => entry?.contribution?.id && entry?.contribution?.component)
  .sort((left, right) => Number(left.contribution.order || 0) - Number(right.contribution.order || 0)))

const goBack = () => {
  if (!nav.canGoBack) return
  const entry = nav.back()
  if (entry) vaultStore.navigateTo(entry)
}

const goForward = () => {
  if (!nav.canGoForward) return
  const entry = nav.forward()
  if (entry) vaultStore.navigateTo(entry)
}
</script>

<style scoped>
.en-nav-bar {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 24px;
  padding: 0;
  -webkit-app-region: no-drag !important;
  pointer-events: auto;
}

.en-nav-btn {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  color: var(--en-muted);
  background: transparent;
  cursor: pointer;
  transition: background 0.12s ease, color 0.12s ease, opacity 0.12s ease;
  -webkit-app-region: no-drag !important;
  pointer-events: auto;
}

.en-nav-btn:hover:not(:disabled) {
  background: var(--en-soft);
  color: var(--en-text);
}

.en-nav-btn.disabled,
.en-nav-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.en-nav-icon {
  width: 18px;
  height: 18px;
}
</style>
