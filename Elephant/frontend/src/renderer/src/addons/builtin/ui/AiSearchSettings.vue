<template>
  <AiProviderSettingsPanel :page="SETTINGS_PAGE.routePage" />
</template>

<script setup>
import AiProviderSettingsPanel from 'elephant-front/components/settings/AiProviderSettingsPanel.vue'
import { AI_SETTINGS_PAGE_BY_ID } from '../aiSettingsRegistry'

const SETTINGS_PAGE = AI_SETTINGS_PAGE_BY_ID.search
</script>
