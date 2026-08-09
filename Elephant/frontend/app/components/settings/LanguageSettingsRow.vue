<template>
  <div class="en-settings-row">
    <div class="en-settings-row-copy">
      <strong>{{ t('settings.language') }}</strong>
      <span>{{ t('settings.languageDescription') }}</span>
    </div>
    <select
      class="en-compact-select en-language-select"
      :value="preference"
      :aria-label="t('settings.language')"
      @change="updateLanguage($event.target.value)"
    >
      <option
        v-for="option in languageOptions"
        :key="option.code"
        :value="option.code"
      >
        {{ option.code === 'system'
          ? `${t('settings.systemLanguage')} · ${option.displayName}`
          : `${option.nativeName} · ${option.displayName}` }}
      </option>
    </select>
  </div>
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import {
  getCurrentLanguage,
  getLanguageOptions,
  getLanguagePreference,
  setLanguage,
  t
} from '@/i18n'

const preference = ref(getLanguagePreference())
const localeRevision = ref(0)

const languageOptions = computed(() => {
  void localeRevision.value
  return getLanguageOptions()
})

const syncLanguageState = () => {
  preference.value = getLanguagePreference()
  localeRevision.value += 1
}

const updateLanguage = (value) => {
  preference.value = String(value || 'system')
  setLanguage(preference.value)
  localeRevision.value += 1
}

onMounted(() => globalThis.addEventListener?.('elephantnote:language-changed', syncLanguageState))
onBeforeUnmount(() => globalThis.removeEventListener?.('elephantnote:language-changed', syncLanguageState))

void getCurrentLanguage()
</script>
