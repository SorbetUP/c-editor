<template>
  <section v-if="ready">
    <button @click="save">Save</button>
    <div v-for="item in items" :key="item.id">{{ item.title }}</div>
  </section>
</template>

<script setup>
import { ref } from 'vue'
const ready = ref(true)
document.querySelector('.legacy')
</script>

<style scoped>
.root {
  display: flex;
  gap: 8px;
  filter: blur(2px);
}
</style>
