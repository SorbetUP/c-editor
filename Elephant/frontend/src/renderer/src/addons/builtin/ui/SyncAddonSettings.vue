<template>
  <SyncSettingsPanel
    :vaults="store.vaults"
    :active-vault-path="store.activeVault?.path || ''"
    initial-page="overview"
  />
</template>

<script setup>
import { useVaultStore } from 'elephant-front/stores/vaultStore'
import SyncSettingsPanel from 'elephant-front/components/settings/SyncSettingsPanel.vue'

const store = useVaultStore()
</script>
