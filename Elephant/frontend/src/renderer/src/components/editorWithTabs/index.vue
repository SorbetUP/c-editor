<template>
  <div
    class="editor-with-tabs"
    :style="{ 'max-width': showSideBar ? `calc(100vw - ${sideBarWidth}px` : '100vw' }"
  >
    <tabs v-show="showTabBar" />
    <div class="container">
      <runtime-editor
        :markdown="markdown"
        :cursor="cursor"
        :text-direction="textDirection"
        :platform="platform"
        :to-editor-markdown="toEditorMarkdown"
        :from-editor-markdown="fromEditorMarkdown"
        :rust-runtime-factory="rustRuntimeFactory"
      />
    </div>
    <tab-notifications />
  </div>
</template>

<script setup>
import { useLayoutStore } from '@/store/layout'
import { storeToRefs } from 'pinia'
import Tabs from './tabs.vue'
import RuntimeEditor from './runtimeEditor.vue'
import TabNotifications from './notifications.vue'

defineProps({
  markdown: {
    type: String,
    required: true
  },
  cursor: {
    validator (value) {
      return typeof value === 'object'
    },
    required: true
  },
  muyaIndexCursor: {
    type: Object
  },
  showTabBar: {
    type: Boolean,
    required: true
  },
  textDirection: {
    type: String,
    required: true
  },
  platform: {
    type: String,
    required: true
  },
  toEditorMarkdown: {
    type: Function,
    default: (markdown) => markdown
  },
  fromEditorMarkdown: {
    type: Function,
    default: (markdown) => markdown
  },
  rustRuntimeFactory: {
    type: Function,
    default: null
  }
})

const layoutStore = useLayoutStore()
const { showSideBar, sideBarWidth } = storeToRefs(layoutStore)
</script>

<style scoped>
.editor-with-tabs {
  position: relative;
  height: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;

  overflow: hidden;
  background: var(--editorBgColor);
  & > .container {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
}
</style>
