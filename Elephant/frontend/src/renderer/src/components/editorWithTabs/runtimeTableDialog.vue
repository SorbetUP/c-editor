<template>
  <el-dialog
    v-model="visible"
    :show-close="true"
    :modal="true"
    custom-class="ag-dialog-table"
    width="454px"
    center
    dir="ltr"
  >
    <template #header>
      <div class="dialog-title">
        {{ t('editor.insertTable.title') }}
      </div>
    </template>
    <el-form
      :model="table"
      :inline="true"
    >
      <el-form-item :label="t('editor.insertTable.rows')">
        <el-input-number
          ref="rowInput"
          v-model="table.rows"
          size="mini"
          controls-position="right"
          :min="1"
          :max="30"
        />
      </el-form-item>
      <el-form-item :label="t('editor.insertTable.columns')">
        <el-input-number
          v-model="table.columns"
          size="mini"
          controls-position="right"
          :min="1"
          :max="20"
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="visible = false">
          {{ t('common.cancel') }}
        </el-button>
        <el-button
          type="primary"
          @click="confirm"
        >
          {{ t('common.ok') }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup>
import { computed, nextTick, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps({
  modelValue: { type: Boolean, required: true }
})

const emit = defineEmits(['update:modelValue', 'confirm'])
const { t } = useI18n()
const rowInput = ref(null)
const table = reactive({ rows: 4, columns: 3 })

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

watch(
  () => props.modelValue,
  async (value) => {
    if (!value) return
    table.rows = 4
    table.columns = 3
    await nextTick()
    rowInput.value?.focus?.()
  }
)

const confirm = () => {
  emit('confirm', { rows: table.rows, columns: table.columns })
  visible.value = false
}
</script>
