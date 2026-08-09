<template>
  <ModelsView class="en-open-models-only" />
</template>

<script setup>
import ModelsView from 'elephant-front/components/views/ModelsView.vue'
</script>

<style scoped>
/* Role assignment is configured by the feature that consumes a model, not by the model catalogue. */
.en-open-models-only :deep(.en-role-grid) {
  display: none;
}
</style>
