<template>
  <main
    class="en-main"
    :class="{ 'has-editor-open': hasOpenNote }"
  >
    <addon-workspace-router
      v-if="!hasOpenNote && activeAddonViewId"
      :view-id="activeAddonViewId"
      @close="emit('close-addon-view')"
    />
    <section
      v-else-if="!hasOpenNote && showLibrary"
      class="en-library"
    >
      <library-toolbar />
      <library-grid />
    </section>
    <template v-if="!hasOpenNote && !activeAddonViewId && store.activeWorkspaceView === 'notes'">
      <template v-for="entry in workspacePanels" :key="entry.contribution.id">
        <component
          :is="entry.contribution.component"
          v-if="isPanelVisible(entry)"
        />
      </template>
    </template>
    <note-editor-host
      v-if="hasOpenNote"
      class="en-main-editor"
    />
  </main>
</template>

<script setup>
import { computed, watch } from 'vue'
import log from '@/platform/runtimeLogShim'
import { useEditorStore } from '@/store/editor'
import { useVaultStore } from '../../stores/vaultStore'
import { useAddonsStore } from '@/store/addons'
import LibraryToolbar from '../library/LibraryToolbar.vue'
import LibraryGrid from '../library/LibraryGrid.vue'
import NoteEditorHost from '../editor/NoteEditorHost.vue'
import AddonWorkspaceRouter from '../views/AddonWorkspaceRouter.vue'

const props = defineProps({
  activeAddonViewId: {
    type: String,
    default: ''
  }
})
const emit = defineEmits(['close-addon-view'])
const store = useVaultStore()
const editorStore = useEditorStore()
const addonsStore = useAddonsStore()
const openedNoteAbsolutePath = computed(() => {
  if (!store.activeVault?.path || !store.openedNotePath) return ''
  return window.path.join(store.activeVault.path, store.openedNotePath)
})
const pathsMatch = (left, right) => {
  if (!left || !right) return false
  if (typeof window.fileUtils?.isSamePathSync === 'function') {
    return window.fileUtils.isSamePathSync(left, right)
  }
  return String(left).replace(/\\/g, '/') === String(right).replace(/\\/g, '/')
}
const hasOpenNote = computed(() => {
  const pathname = openedNoteAbsolutePath.value
  if (!pathname) return false
  return [editorStore.currentFile, ...(editorStore.tabs || [])]
    .some((file) => pathsMatch(file?.pathname, pathname) &&
      file?.id && typeof file.markdown === 'string')
})
watch(
  [hasOpenNote, openedNoteAbsolutePath, () => store.openedNotePath, () => editorStore.currentFile?.pathname, () => editorStore.tabs.length],
  ([nextOpen, absolutePath, relativePath, currentPath, tabCount], previous) => {
    log.info('[main-content] note-visibility', {
      nextOpen,
      previousOpen: previous?.[0] ?? null,
      absolutePath: absolutePath || null,
      relativePath: relativePath || null,
      currentPath: currentPath || null,
      tabCount,
      activeVaultId: store.activeVaultId || null,
      workspaceView: store.activeWorkspaceView
    })
  },
  { immediate: true }
)
const activeAddonViewId = computed(() => props.activeAddonViewId)
const showLibrary = computed(() => store.activeWorkspaceView === 'notes')
const workspacePanels = computed(() => addonsStore.getContributions('layout.zones')
  .filter((entry) => entry?.contribution?.zone === 'workspace.notes' && entry?.contribution?.component)
  .sort((left, right) => Number(left.contribution.order || 0) - Number(right.contribution.order || 0)))

const isPanelVisible = (entry) => {
  const predicate = entry?.contribution?.when
  if (typeof predicate !== 'function') return true
  try {
    return predicate() === true
  } catch (error) {
    console.warn('[addons] workspace panel visibility predicate failed', {
      id: entry?.contribution?.id || '',
      error
    })
    return false
  }
}
</script>

<style scoped>
.en-main {
  position: relative;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--en-bg);
  overflow: hidden;
}

.en-library {
  min-height: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.en-main-editor {
  min-height: 0;
  flex: 1;
}
</style>
