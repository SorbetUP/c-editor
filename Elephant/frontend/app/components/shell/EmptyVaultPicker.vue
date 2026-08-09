<template>
  <main class="en-empty">
    <section class="en-empty-card">
      <img class="en-logo-image" src="../../../../assets/static/icon.png" alt="Elephant">
      <h1>Choose your first vault</h1>
      <p class="en-empty-copy">Keep notes in private app storage or choose an existing folder.</p>
      <div class="en-empty-actions">
        <button class="en-primary-button" type="button" @click="emit('create-local')">
          <strong>Private storage</strong>
          <span>Create a vault managed by Elephant</span>
        </button>
        <button
          class="en-secondary-button"
          type="button"
          aria-label="Dossier Android — Choose a folder and authorize access"
          @click="emit('choose')"
        >
          <strong>Choose folder</strong>
          <span>Open an existing notes folder</span>
        </button>
      </div>
    </section>
  </main>
</template>

<script setup>
const emit = defineEmits(['create-local', 'choose'])
</script>

<style scoped>
.en-empty { min-height: 100dvh; display: grid; place-items: center; padding: 32px; background: var(--en-bg, #0f141d); color: var(--en-text, #eef3fb); }
.en-empty-card { width: min(440px, 100%); display: grid; gap: 18px; text-align: center; }
.en-logo-image { width: 96px; height: 96px; object-fit: contain; margin: 0 auto; }
.en-empty-card h1, .en-empty-card p { margin: 0; }
.en-empty-copy { color: var(--en-muted, #98a3b6); }
.en-empty-actions { display: grid; gap: 12px; }
.en-primary-button, .en-secondary-button { min-height: 64px; display: grid; gap: 4px; padding: 12px 16px; border-radius: 14px; border: 1px solid var(--en-border, #283244); color: inherit; }
.en-primary-button { background: var(--en-primary, #2563eb); color: white; }
.en-secondary-button { background: var(--en-surface, #141a24); }
.en-primary-button span, .en-secondary-button span { font-size: 12px; opacity: .8; }
</style>
