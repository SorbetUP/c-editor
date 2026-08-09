<template>
  <section class="en-addon-workspace">
    <div v-if="!view" class="en-addon-empty">
      <h2>Addon view unavailable</h2>
      <p>The addon may have been disabled or uninstalled.</p>
      <button type="button" @click="emit('close')">Back to notes</button>
    </div>

    <div v-else-if="view.contribution.kind !== 'task-manager-v1'" class="en-addon-empty">
      <h2>Unsupported addon view</h2>
      <p>ElephantNote does not support the declarative view kind <code>{{ view.contribution.kind }}</code>.</p>
      <button type="button" @click="emit('close')">Back to notes</button>
    </div>

    <template v-else>
      <aside class="en-task-nav">
        <div class="en-task-brand">
          <div class="en-task-brand-mark">✓</div>
          <div>
            <strong>{{ state?.title || view.contribution.title }}</strong>
            <span>{{ state?.subtitle || 'Plan what matters' }}</span>
          </div>
        </div>

        <nav class="en-task-nav-groups" aria-label="Task lists">
          <div class="en-task-nav-group">
            <button
              v-for="item in state?.navigation || fallbackNavigation"
              :key="item.id"
              class="en-task-nav-item"
              :class="{ active: activeList === item.id }"
              type="button"
              @click="selectList(item.id)"
            >
              <span class="en-task-nav-icon">{{ navigationIcon(item.id) }}</span>
              <span>{{ item.title }}</span>
              <span v-if="Number.isFinite(item.count)" class="en-task-count">{{ item.count }}</span>
            </button>
          </div>

          <div class="en-task-nav-group">
            <div class="en-task-nav-heading">
              <span>Areas</span>
              <button type="button" title="Create area" @click="showAreaComposer = !showAreaComposer">+</button>
            </div>
            <form v-if="showAreaComposer" class="en-inline-composer" @submit.prevent="createArea">
              <input v-model="newAreaTitle" maxlength="100" placeholder="New area" autofocus>
              <button type="submit" :disabled="!newAreaTitle.trim()">Add</button>
            </form>
            <button
              v-for="area in state?.areas || []"
              :key="area.id"
              class="en-task-nav-item en-task-nav-subitem"
              :class="{ active: activeList === `area:${area.id}` }"
              type="button"
              @click="selectList(`area:${area.id}`)"
            >
              <span class="en-area-dot" :style="{ background: area.color || 'var(--en-primary)' }" />
              <span>{{ area.title }}</span>
              <span class="en-task-count">{{ area.count || 0 }}</span>
            </button>
          </div>

          <div class="en-task-nav-group">
            <div class="en-task-nav-heading">
              <span>Projects</span>
              <button type="button" title="Create project" @click="showProjectComposer = !showProjectComposer">+</button>
            </div>
            <form v-if="showProjectComposer" class="en-inline-composer" @submit.prevent="createProject">
              <input v-model="newProjectTitle" maxlength="160" placeholder="New project" autofocus>
              <select v-model="newProjectAreaId" aria-label="Project area">
                <option value="">No area</option>
                <option v-for="area in state?.areas || []" :key="area.id" :value="area.id">{{ area.title }}</option>
              </select>
              <button type="submit" :disabled="!newProjectTitle.trim()">Add</button>
            </form>
            <button
              v-for="project in state?.projects || []"
              :key="project.id"
              class="en-task-nav-item en-task-nav-subitem"
              :class="{ active: activeList === `project:${project.id}` }"
              type="button"
              @click="selectList(`project:${project.id}`)"
            >
              <span class="en-project-progress" :style="{ '--progress': `${project.progress || 0}%` }" />
              <span>{{ project.title }}</span>
              <span class="en-task-count">{{ project.openCount || 0 }}</span>
            </button>
          </div>
        </nav>
      </aside>

      <main class="en-task-main">
        <header class="en-task-header">
          <div>
            <p class="en-task-eyebrow">{{ state?.activeList?.eyebrow || 'Tasks' }}</p>
            <h1>{{ state?.activeList?.title || 'Inbox' }}</h1>
            <p>{{ state?.activeList?.description || 'Capture now, decide later.' }}</p>
          </div>
          <div class="en-task-header-actions">
            <button type="button" title="Refresh" @click="load">↻</button>
            <button type="button" title="Close task view" @click="emit('close')">×</button>
          </div>
        </header>

        <form class="en-quick-task" @submit.prevent="quickAdd">
          <button type="submit" aria-label="Add task" :disabled="!quickTitle.trim()">+</button>
          <input
            v-model="quickTitle"
            maxlength="500"
            placeholder="Add a task to the Inbox…"
            aria-label="New task title"
          >
          <button class="en-quick-schedule" type="button" @click="quickToday = !quickToday">
            {{ quickToday ? 'Today' : 'Inbox' }}
          </button>
        </form>

        <div class="en-task-toolbar">
          <label class="en-task-search">
            <span>⌕</span>
            <input v-model="query" placeholder="Search tasks" @input="scheduleLoad">
          </label>
          <select v-model="tagFilter" @change="load">
            <option value="">All tags</option>
            <option v-for="tag in state?.tags || []" :key="tag.name" :value="tag.name">#{{ tag.name }} ({{ tag.count }})</option>
          </select>
          <span class="en-task-summary">{{ state?.summary?.open || 0 }} open</span>
        </div>

        <div class="en-task-scroll">
          <div v-if="loading" class="en-task-message">Loading tasks…</div>
          <div v-else-if="error" class="en-task-message error">
            <strong>Unable to load this addon view</strong>
            <span>{{ error }}</span>
            <button type="button" @click="load">Retry</button>
          </div>
          <div v-else-if="!state?.sections?.some(section => section.tasks?.length)" class="en-task-message">
            <strong>{{ emptyTitle }}</strong>
            <span>{{ emptyDescription }}</span>
          </div>

          <section v-for="section in state?.sections || []" :key="section.id" class="en-task-section">
            <header v-if="section.title" class="en-task-section-title">
              <h2>{{ section.title }}</h2>
              <span>{{ section.tasks?.length || 0 }}</span>
            </header>
            <article
              v-for="task in section.tasks || []"
              :key="task.id"
              class="en-task-row"
              :class="{ selected: selectedTaskId === task.id, completed: task.status === 'completed' }"
              @click="selectTask(task)"
            >
              <button
                class="en-task-checkbox"
                type="button"
                :aria-label="task.status === 'completed' ? 'Reopen task' : 'Complete task'"
                @click.stop="toggleTask(task)"
              >
                {{ task.status === 'completed' ? '✓' : '' }}
              </button>
              <div class="en-task-row-body">
                <div class="en-task-row-title">
                  <span>{{ task.title }}</span>
                  <span v-if="task.evening" class="en-task-badge evening">Evening</span>
                  <span v-if="task.repeating" class="en-task-badge">↻</span>
                </div>
                <div class="en-task-meta">
                  <span v-if="task.projectTitle">▣ {{ task.projectTitle }}</span>
                  <span v-else-if="task.areaTitle">● {{ task.areaTitle }}</span>
                  <span v-if="task.startDate">Starts {{ formatDate(task.startDate) }}</span>
                  <span v-if="task.deadline" :class="{ overdue: task.deadlineOverdue }">Deadline {{ formatDate(task.deadline) }}</span>
                  <span v-for="tag in task.tags || []" :key="tag">#{{ tag }}</span>
                </div>
              </div>
              <button
                v-if="task.status === 'open'"
                class="en-task-today"
                :class="{ active: task.today }"
                type="button"
                :title="task.today ? 'Remove from Today' : 'Add to Today'"
                @click.stop="dispatch('toggleToday', { id: task.id })"
              >
                ★
              </button>
            </article>
          </section>
        </div>
      </main>

      <aside v-if="selectedTask" class="en-task-detail">
        <header>
          <span>Task details</span>
          <button type="button" @click="selectedTaskId = ''; draft = null">×</button>
        </header>
        <div class="en-task-detail-scroll">
          <label>
            <span>Title</span>
            <textarea v-model="draft.title" rows="2" maxlength="500" />
          </label>
          <label>
            <span>Notes</span>
            <textarea v-model="draft.notes" rows="6" maxlength="20000" placeholder="Add context, links or decisions…" />
          </label>
          <div class="en-task-detail-grid">
            <label>
              <span>Start date</span>
              <input v-model="draft.startDate" type="date">
            </label>
            <label>
              <span>Deadline</span>
              <input v-model="draft.deadline" type="date">
            </label>
          </div>
          <div class="en-task-detail-grid">
            <label>
              <span>Area</span>
              <select v-model="draft.areaId">
                <option value="">None</option>
                <option v-for="area in state?.areas || []" :key="area.id" :value="area.id">{{ area.title }}</option>
              </select>
            </label>
            <label>
              <span>Project</span>
              <select v-model="draft.projectId">
                <option value="">None</option>
                <option v-for="project in state?.projects || []" :key="project.id" :value="project.id">{{ project.title }}</option>
              </select>
            </label>
          </div>
          <label>
            <span>When</span>
            <select v-model="draft.bucket">
              <option value="inbox">Inbox</option>
              <option value="anytime">Anytime</option>
              <option value="someday">Someday</option>
            </select>
          </label>
          <label class="en-check-line">
            <input v-model="draft.today" type="checkbox">
            <span>Show in Today</span>
          </label>
          <label class="en-check-line">
            <input v-model="draft.evening" type="checkbox">
            <span>Move to Evening in Today</span>
          </label>
          <label>
            <span>Tags</span>
            <input v-model="draft.tagsText" placeholder="work, errands, deep-work">
          </label>

          <fieldset class="en-repeat-editor">
            <legend>Repeat</legend>
            <label class="en-check-line">
              <input v-model="draft.repeatEnabled" type="checkbox">
              <span>Repeat this task</span>
            </label>
            <template v-if="draft.repeatEnabled">
              <div class="en-task-detail-grid">
                <label>
                  <span>Schedule</span>
                  <select v-model="draft.repeatMode">
                    <option value="fixed">On a fixed schedule</option>
                    <option value="after-completion">After completion</option>
                  </select>
                </label>
                <label>
                  <span>Frequency</span>
                  <select v-model="draft.repeatFrequency">
                    <option value="daily">Daily</option>
                    <option value="weekly">Weekly</option>
                    <option value="monthly">Monthly</option>
                    <option value="yearly">Yearly</option>
                  </select>
                </label>
              </div>
              <label>
                <span>Every</span>
                <input v-model.number="draft.repeatInterval" type="number" min="1" max="365">
              </label>
            </template>
          </fieldset>

          <div class="en-task-detail-actions">
            <button class="primary" type="button" :disabled="saving || !draft.title.trim()" @click="saveTask">
              {{ saving ? 'Saving…' : 'Save changes' }}
            </button>
            <button type="button" @click="dispatch('cancelTask', { id: draft.id })">Cancel task</button>
            <button class="danger" type="button" @click="deleteArmed ? deleteTask() : deleteArmed = true">
              {{ deleteArmed ? 'Confirm delete' : 'Delete' }}
            </button>
          </div>
        </div>
      </aside>
    </template>
  </section>
</template>

<script setup>
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useAddonsStore } from '@/store/addons'

const props = defineProps({
  viewId: {
    type: String,
    required: true
  }
})
const emit = defineEmits(['close'])
const addonsStore = useAddonsStore()
const state = ref(null)
const loading = ref(false)
const saving = ref(false)
const error = ref('')
const activeList = ref('inbox')
const query = ref('')
const tagFilter = ref('')
const quickTitle = ref('')
const quickToday = ref(false)
const selectedTaskId = ref('')
const draft = ref(null)
const deleteArmed = ref(false)
const showAreaComposer = ref(false)
const showProjectComposer = ref(false)
const newAreaTitle = ref('')
const newProjectTitle = ref('')
const newProjectAreaId = ref('')
let loadTimer = null
let loadGeneration = 0

const fallbackNavigation = [
  { id: 'inbox', title: 'Inbox', count: 0 },
  { id: 'today', title: 'Today', count: 0 },
  { id: 'upcoming', title: 'Upcoming', count: 0 },
  { id: 'anytime', title: 'Anytime', count: 0 },
  { id: 'someday', title: 'Someday', count: 0 },
  { id: 'deadlines', title: 'Deadlines', count: 0 },
  { id: 'repeating', title: 'Repeating', count: 0 },
  { id: 'logbook', title: 'Logbook', count: 0 }
]

const view = computed(() => addonsStore.getContributions('views')
  .find((entry) => entry.contribution?.id === props.viewId) || null)
const selectedTask = computed(() => state.value?.selectedTask || null)
const emptyTitle = computed(() => activeList.value === 'inbox' ? 'Inbox cleared' : 'Nothing here')
const emptyDescription = computed(() => activeList.value === 'inbox'
  ? 'Capture something new, or enjoy the empty inbox.'
  : 'Tasks will appear here when they match this list.')

const navigationIcon = (id) => ({
  inbox: '↓',
  today: '★',
  upcoming: '▤',
  anytime: '○',
  someday: '☁',
  deadlines: '!',
  repeating: '↻',
  logbook: '✓'
}[id] || '•')

const formatDate = (value) => {
  if (!value) return ''
  const date = new Date(`${value}T12:00:00`)
  if (Number.isNaN(date.getTime())) return value
  return new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric', year: date.getFullYear() !== new Date().getFullYear() ? 'numeric' : undefined }).format(date)
}

const makeDraft = (task) => task ? {
  ...task,
  tagsText: (task.tags || []).join(', '),
  repeatEnabled: Boolean(task.recurrence),
  repeatMode: task.recurrence?.mode || 'fixed',
  repeatFrequency: task.recurrence?.frequency || 'weekly',
  repeatInterval: task.recurrence?.interval || 1
} : null

const applyState = (nextState) => {
  state.value = nextState || null
  const selected = nextState?.selectedTask
  if (selected) {
    selectedTaskId.value = selected.id
    draft.value = makeDraft(selected)
  } else if (selectedTaskId.value) {
    selectedTaskId.value = ''
    draft.value = null
  }
}

const load = async () => {
  const contribution = view.value?.contribution
  if (!contribution?.getState) return
  const generation = ++loadGeneration
  loading.value = true
  error.value = ''
  try {
    const nextState = await contribution.getState({
      list: activeList.value,
      query: query.value,
      tag: tagFilter.value,
      selectedTaskId: selectedTaskId.value
    })
    if (generation === loadGeneration) applyState(nextState)
  } catch (cause) {
    if (generation === loadGeneration) error.value = cause?.message || String(cause)
  } finally {
    if (generation === loadGeneration) loading.value = false
  }
}

const scheduleLoad = () => {
  clearTimeout(loadTimer)
  loadTimer = setTimeout(load, 180)
}

const dispatch = async (action, params = {}) => {
  const contribution = view.value?.contribution
  if (!contribution?.dispatch) throw new Error('Addon view action dispatcher is unavailable')
  error.value = ''
  try {
    const result = await contribution.dispatch(action, params)
    if (result?.state) applyState(result.state)
    else await load()
    return result
  } catch (cause) {
    error.value = cause?.message || String(cause)
    throw cause
  }
}

const selectList = (id) => {
  activeList.value = id
  selectedTaskId.value = ''
  draft.value = null
  deleteArmed.value = false
  void load()
}

const selectTask = (task) => {
  selectedTaskId.value = task.id
  draft.value = makeDraft(task)
  deleteArmed.value = false
  void load()
}

const quickAdd = async () => {
  const title = quickTitle.value.trim()
  if (!title) return
  await dispatch('createTask', {
    title,
    bucket: quickToday.value ? 'anytime' : 'inbox',
    today: quickToday.value
  })
  quickTitle.value = ''
}

const toggleTask = (task) => dispatch(task.status === 'completed' ? 'reopenTask' : 'completeTask', { id: task.id })

const saveTask = async () => {
  if (!draft.value?.title?.trim()) return
  saving.value = true
  try {
    await dispatch('updateTask', {
      id: draft.value.id,
      title: draft.value.title.trim(),
      notes: draft.value.notes || '',
      bucket: draft.value.bucket,
      startDate: draft.value.startDate || '',
      deadline: draft.value.deadline || '',
      today: Boolean(draft.value.today),
      evening: Boolean(draft.value.evening),
      areaId: draft.value.areaId || '',
      projectId: draft.value.projectId || '',
      tags: draft.value.tagsText.split(',').map(value => value.trim()).filter(Boolean),
      recurrence: draft.value.repeatEnabled ? {
        mode: draft.value.repeatMode,
        frequency: draft.value.repeatFrequency,
        interval: Math.max(1, Number(draft.value.repeatInterval) || 1)
      } : null
    })
  } finally {
    saving.value = false
  }
}

const deleteTask = async () => {
  await dispatch('deleteTask', { id: draft.value.id })
  selectedTaskId.value = ''
  draft.value = null
  deleteArmed.value = false
}

const createArea = async () => {
  const title = newAreaTitle.value.trim()
  if (!title) return
  await dispatch('createArea', { title })
  newAreaTitle.value = ''
  showAreaComposer.value = false
}

const createProject = async () => {
  const title = newProjectTitle.value.trim()
  if (!title) return
  await dispatch('createProject', { title, areaId: newProjectAreaId.value })
  newProjectTitle.value = ''
  newProjectAreaId.value = ''
  showProjectComposer.value = false
}

watch(() => props.viewId, () => {
  activeList.value = 'inbox'
  selectedTaskId.value = ''
  void load()
}, { immediate: true })
watch(view, (nextView) => {
  if (nextView) void load()
})

onBeforeUnmount(() => clearTimeout(loadTimer))
</script>

<style scoped>
.en-addon-workspace {
  min-width: 0;
  min-height: 0;
  flex: 1;
  display: grid;
  grid-template-columns: 218px minmax(420px, 1fr) minmax(280px, 340px);
  background: var(--en-bg);
  color: var(--en-text);
  overflow: hidden;
}
.en-addon-empty { margin: auto; max-width: 460px; padding: 32px; text-align: center; }
.en-addon-empty p { color: var(--en-muted); }
.en-addon-empty button,
.en-task-header-actions button,
.en-task-nav-heading button { border: 0; border-radius: 8px; background: var(--en-soft); color: var(--en-text); cursor: pointer; }
.en-task-nav { min-height: 0; border-right: 1px solid var(--en-border); background: var(--en-sidebar-bg, var(--en-bg)); display: flex; flex-direction: column; overflow: hidden; }
.en-task-brand { display: flex; gap: 10px; align-items: center; padding: 18px 14px 12px; }
.en-task-brand-mark { width: 34px; height: 34px; display: grid; place-items: center; border-radius: 11px; background: var(--en-primary); color: white; font-weight: 800; }
.en-task-brand strong,.en-task-brand span { display: block; }
.en-task-brand span { margin-top: 2px; color: var(--en-muted); font-size: 11px; }
.en-task-nav-groups { min-height: 0; padding: 4px 8px 16px; overflow-y: auto; }
.en-task-nav-group { display: flex; flex-direction: column; gap: 2px; margin-bottom: 14px; }
.en-task-nav-item { width: 100%; min-height: 34px; display: grid; grid-template-columns: 20px minmax(0,1fr) auto; gap: 8px; align-items: center; border: 0; border-radius: 8px; padding: 0 9px; color: var(--en-muted); background: transparent; text-align: left; cursor: pointer; }
.en-task-nav-item:hover { background: var(--en-soft); color: var(--en-text); }
.en-task-nav-item.active { background: color-mix(in srgb,var(--en-primary) 18%,var(--en-soft)); color: var(--en-text); }
.en-task-nav-icon { text-align: center; font-size: 14px; }
.en-task-count { font-size: 11px; font-variant-numeric: tabular-nums; }
.en-task-nav-heading { min-height: 28px; display: flex; justify-content: space-between; align-items: center; padding: 0 7px; color: var(--en-muted); font-size: 10px; font-weight: 800; letter-spacing: .08em; text-transform: uppercase; }
.en-task-nav-heading button { width: 22px; height: 22px; }
.en-task-nav-subitem { grid-template-columns: 14px minmax(0,1fr) auto; }
.en-area-dot { width: 8px; height: 8px; border-radius: 50%; }
.en-project-progress { width: 12px; height: 12px; border: 1px solid var(--en-muted); border-radius: 50%; background: conic-gradient(var(--en-primary) var(--progress), transparent 0); }
.en-inline-composer { display: grid; gap: 5px; padding: 4px 5px 8px; }
.en-inline-composer input,.en-inline-composer select { min-width: 0; height: 30px; border: 1px solid var(--en-border); border-radius: 7px; padding: 0 8px; color: var(--en-text); background: var(--en-surface); }
.en-inline-composer button { height: 28px; border: 0; border-radius: 7px; background: var(--en-primary); color: white; cursor: pointer; }
.en-task-main { min-width: 0; min-height: 0; display: flex; flex-direction: column; overflow: hidden; }
.en-task-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 16px; padding: 25px 30px 14px; }
.en-task-header h1 { margin: 1px 0 4px; font-size: 28px; letter-spacing: -.025em; }
.en-task-header p { margin: 0; color: var(--en-muted); font-size: 13px; }
.en-task-eyebrow { color: var(--en-primary)!important; font-size: 10px!important; font-weight: 800; letter-spacing: .1em; text-transform: uppercase; }
.en-task-header-actions { display: flex; gap: 6px; }
.en-task-header-actions button { width: 30px; height: 30px; }
.en-quick-task { margin: 0 30px 12px; min-height: 45px; display: grid; grid-template-columns: 30px minmax(0,1fr) auto; align-items: center; gap: 8px; border: 1px solid var(--en-border); border-radius: 12px; padding: 0 10px; background: var(--en-surface); }
.en-quick-task>button:first-child { width: 25px; height: 25px; border: 1px solid var(--en-border-strong); border-radius: 50%; background: transparent; color: var(--en-primary); font-size: 20px; cursor: pointer; }
.en-quick-task input { min-width: 0; border: 0; outline: 0; color: var(--en-text); background: transparent; font: inherit; }
.en-quick-schedule { border: 0; border-radius: 7px; padding: 5px 8px; background: var(--en-soft); color: var(--en-muted); cursor: pointer; }
.en-task-toolbar { display: flex; align-items: center; gap: 8px; padding: 0 30px 12px; }
.en-task-search { min-width: 140px; flex: 1; display: flex; gap: 7px; align-items: center; border: 1px solid var(--en-border); border-radius: 8px; padding: 0 9px; background: var(--en-surface); }
.en-task-search input { width: 100%; height: 31px; border: 0; outline: 0; color: var(--en-text); background: transparent; }
.en-task-toolbar select { max-width: 180px; height: 33px; border: 1px solid var(--en-border); border-radius: 8px; color: var(--en-text); background: var(--en-surface); }
.en-task-summary { color: var(--en-muted); font-size: 12px; white-space: nowrap; }
.en-task-scroll { min-height: 0; flex: 1; padding: 0 30px 40px; overflow-y: auto; }
.en-task-section { margin-bottom: 22px; }
.en-task-section-title { height: 31px; display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--en-border); color: var(--en-muted); }
.en-task-section-title h2 { margin: 0; font-size: 12px; font-weight: 700; }
.en-task-section-title span { font-size: 11px; }
.en-task-row { min-height: 54px; display: grid; grid-template-columns: 24px minmax(0,1fr) 28px; gap: 10px; align-items: center; border-bottom: 1px solid color-mix(in srgb,var(--en-border) 65%,transparent); padding: 7px 5px; cursor: pointer; }
.en-task-row:hover,.en-task-row.selected { background: color-mix(in srgb,var(--en-soft) 75%,transparent); }
.en-task-row.completed { opacity: .64; }
.en-task-row.completed .en-task-row-title>span:first-child { text-decoration: line-through; }
.en-task-checkbox { width: 21px; height: 21px; border: 1.5px solid var(--en-border-strong); border-radius: 50%; background: transparent; color: white; cursor: pointer; }
.en-task-row.completed .en-task-checkbox { border-color: var(--en-primary); background: var(--en-primary); }
.en-task-row-body { min-width: 0; }
.en-task-row-title { display: flex; gap: 7px; align-items: center; font-size: 14px; }
.en-task-row-title>span:first-child { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.en-task-meta { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 4px; color: var(--en-muted); font-size: 10px; }
.en-task-meta .overdue { color: var(--en-danger,#ef4444); }
.en-task-badge { border-radius: 5px; padding: 2px 5px; background: var(--en-soft); color: var(--en-muted); font-size: 9px; }
.en-task-badge.evening { color: #a78bfa; }
.en-task-today { border: 0; background: transparent; color: var(--en-border-strong); cursor: pointer; }
.en-task-today.active { color: #f5b942; }
.en-task-message { min-height: 180px; display: flex; flex-direction: column; gap: 8px; align-items: center; justify-content: center; color: var(--en-muted); text-align: center; }
.en-task-message strong { color: var(--en-text); }
.en-task-message button { border: 0; border-radius: 8px; padding: 7px 12px; color: white; background: var(--en-primary); cursor: pointer; }
.en-task-message.error span { color: var(--en-danger,#ef4444); }
.en-task-detail { min-width: 0; min-height: 0; border-left: 1px solid var(--en-border); background: var(--en-surface); overflow: hidden; }
.en-task-detail>header { height: 50px; display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--en-border); padding: 0 16px; font-size: 12px; font-weight: 700; }
.en-task-detail>header button { border: 0; background: transparent; color: var(--en-muted); font-size: 20px; cursor: pointer; }
.en-task-detail-scroll { height: calc(100% - 50px); display: flex; flex-direction: column; gap: 14px; padding: 17px; overflow-y: auto; }
.en-task-detail label { display: flex; flex-direction: column; gap: 5px; color: var(--en-muted); font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: .04em; }
.en-task-detail input,.en-task-detail textarea,.en-task-detail select { width: 100%; border: 1px solid var(--en-border); border-radius: 8px; padding: 8px; color: var(--en-text); background: var(--en-bg); font: inherit; font-size: 12px; text-transform: none; letter-spacing: normal; }
.en-task-detail textarea { resize: vertical; }
.en-task-detail-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 9px; }
.en-check-line { flex-direction: row!important; align-items: center; gap: 8px!important; text-transform: none!important; letter-spacing: 0!important; }
.en-check-line input { width: auto; }
.en-repeat-editor { border: 1px solid var(--en-border); border-radius: 9px; padding: 10px; }
.en-repeat-editor legend { padding: 0 5px; color: var(--en-muted); font-size: 10px; font-weight: 700; }
.en-task-detail-actions { display: flex; flex-direction: column; gap: 7px; padding-top: 5px; }
.en-task-detail-actions button { min-height: 34px; border: 1px solid var(--en-border); border-radius: 8px; color: var(--en-text); background: var(--en-bg); cursor: pointer; }
.en-task-detail-actions .primary { border-color: var(--en-primary); background: var(--en-primary); color: white; }
.en-task-detail-actions .danger { color: var(--en-danger,#ef4444); }
@media (max-width: 1050px) {
  .en-addon-workspace { grid-template-columns: 190px minmax(360px,1fr); }
  .en-task-detail { position: absolute; z-index: 5; top: 0; right: 0; bottom: 0; width: min(360px,90vw); box-shadow: -18px 0 40px rgba(0,0,0,.25); }
}
@media (max-width: 720px) {
  .en-addon-workspace { grid-template-columns: 1fr; }
  .en-task-nav { display: none; }
  .en-task-header { padding: 18px 16px 10px; }
  .en-quick-task { margin: 0 16px 10px; }
  .en-task-toolbar { padding: 0 16px 10px; flex-wrap: wrap; }
  .en-task-scroll { padding: 0 16px 32px; }
}
</style>
