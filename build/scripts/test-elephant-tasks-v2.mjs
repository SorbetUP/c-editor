import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import vm from 'node:vm'

const root = path.resolve(process.argv[2] || '')
if (!process.argv[2]) {
  console.error('Usage: node build/scripts/test-elephant-tasks-v2.mjs <catalogue-directory>')
  process.exit(2)
}

const pad = value => String(value).padStart(2, '0')
const dateString = date => `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`
const offsetDate = days => {
  const date = new Date()
  date.setHours(12, 0, 0, 0)
  date.setDate(date.getDate() + days)
  return dateString(date)
}

const manifest = JSON.parse(await fs.readFile(path.join(root, 'addons/elephant-tasks/manifest.json'), 'utf8'))
const source = await fs.readFile(path.join(root, 'addons/elephant-tasks', manifest.runtime.entry), 'utf8')
const sandbox = { self: {}, Intl, Date, Math, console }
vm.runInNewContext(source, sandbox, { filename: `elephant-tasks/${manifest.runtime.entry}`, timeout: 2_000 })
const definition = sandbox.self.elephantAddon
assert.equal(typeof definition?.activate, 'function')

const values = new Map()
const commands = new Map()
const views = new Map()
const api = {
    app: { info: async () => ({ name: 'ElephantNote', version: '0.1.0', addonApiVersion: 1 }) },
  notes: {
    list: async () => [],
    read: async () => { throw new Error('notes.read not expected') },
    write: async () => { throw new Error('notes.write not expected') }
  },
  http: { request: async () => { throw new Error('network not expected') } },
  storage: {
    get: async key => values.get(key) ?? null,
    set: async (key, value) => { values.set(key, structuredClone(value)); return { ok: true } },
    remove: async key => values.delete(key),
    entries: async () => Object.fromEntries(values)
  },
  commands: {
    register(command) {
      commands.set(command.id, command)
      return () => commands.delete(command.id)
    }
  },
  views: {
    register(view) {
      views.set(view.id, view)
      return () => views.delete(view.id)
    }
  }
}

const dispose = await definition.activate(api)
assert.equal(commands.size, 1)
assert.equal(views.size, 1)
const command = commands.get('com.elephantnote.elephant-tasks.capture')
const view = views.get('com.elephantnote.elephant-tasks.workspace')
assert.equal(view.kind, 'task-manager-v1')

let state = await view.getState({ list: 'inbox' })
assert.equal(state.schema, 'task-manager-v1')
assert.equal(state.summary.open, 0)

const capture = await command.run({ title: `Prepare release !today #work due:${offsetDate(2)}` })
assert.ok(capture.id)
assert.equal(capture.openView, 'com.elephantnote.elephant-tasks.workspace')
state = await view.getState({ list: 'today', selectedTaskId: capture.id })
assert.equal(state.sections.flatMap(section => section.tasks).length, 1)
assert.equal(state.selectedTask.tags.includes('work'), true)
assert.equal(state.selectedTask.deadline, offsetDate(2))

await view.dispatch('createArea', { title: 'Work', tags: ['office'] })
state = await view.getState({ list: 'today', selectedTaskId: capture.id })
const areaId = state.areas.find(area => area.title === 'Work')?.id
assert.ok(areaId)
await view.dispatch('createProject', { title: 'Ship 1.0', areaId, tags: ['release'] })
state = await view.getState({ list: 'today', selectedTaskId: capture.id })
const projectId = state.projects.find(project => project.title === 'Ship 1.0')?.id
assert.ok(projectId)

await view.dispatch('updateTask', {
  id: capture.id,
  title: 'Prepare release',
  notes: 'Coordinate the final release.',
  bucket: 'anytime',
  startDate: offsetDate(3),
  deadline: offsetDate(5),
  today: false,
  evening: false,
  areaId: '',
  projectId,
  heading: 'Launch',
  tags: ['deep-work'],
  checklist: [{ title: 'Run tests' }, { title: 'Publish notes' }],
  recurrence: null
})

state = await view.getState({ list: 'anytime' })
assert.equal(state.sections.flatMap(section => section.tasks).some(task => task.id === capture.id), false, 'future start dates must stay out of Anytime')
state = await view.getState({ list: 'upcoming', selectedTaskId: capture.id })
assert.equal(state.sections.flatMap(section => section.tasks).some(task => task.id === capture.id), true)
assert.deepEqual([...state.selectedTask.tags].sort(), ['deep-work', 'office', 'release'].sort())
assert.equal(state.selectedTask.projectTitle, 'Ship 1.0')
assert.equal(state.selectedTask.areaTitle, 'Work')
assert.equal(state.selectedTask.checklistTotal, 2)

state = await view.getState({ list: `area:${areaId}` })
assert.equal(state.sections.flatMap(section => section.tasks).some(task => task.id === capture.id), true, 'Area must include tasks inherited through its Project')
state = await view.dispatch('toggleToday', { id: capture.id })
assert.equal(state.state.activeList.id, `area:${areaId}`, 'view actions must preserve the current list context')
state = await view.getState({ list: 'today' })
assert.equal(state.sections.flatMap(section => section.tasks).some(task => task.id === capture.id), true, 'manual Today must override a future start date')

const recurring = await command.run({ title: 'Weekly review !today #review' })
await view.dispatch('updateTask', {
  id: recurring.id,
  title: 'Weekly review',
  notes: '',
  bucket: 'anytime',
  startDate: offsetDate(0),
  deadline: '',
  today: true,
  evening: true,
  areaId: '',
  projectId: '',
  heading: '',
  tags: ['review'],
  recurrence: { mode: 'after-completion', frequency: 'weekly', interval: 1 }
})
await view.dispatch('completeTask', { id: recurring.id })
state = await view.getState({ list: 'logbook', selectedTaskId: recurring.id })
assert.equal(state.selectedTask.status, 'completed')
state = await view.getState({ list: 'upcoming' })
const nextReview = state.sections.flatMap(section => section.tasks).find(task => task.title === 'Weekly review')
assert.ok(nextReview, 'after-completion recurrence must create a separate future occurrence')
assert.notEqual(nextReview.id, recurring.id)
assert.equal(nextReview.startDate, offsetDate(7))

const fixed = await command.run({ title: 'Water plants !today' })
await view.dispatch('updateTask', {
  id: fixed.id,
  title: 'Water plants',
  notes: '',
  bucket: 'anytime',
  startDate: offsetDate(0),
  deadline: '',
  today: true,
  evening: false,
  areaId: '',
  projectId: '',
  heading: '',
  tags: [],
  recurrence: { mode: 'fixed', frequency: 'daily', interval: 2 }
})
state = await view.getState({ list: 'upcoming' })
const plantOccurrences = state.sections.flatMap(section => section.tasks).filter(task => task.title === 'Water plants')
assert.equal(plantOccurrences.some(task => task.startDate === offsetDate(2)), true)
assert.ok(plantOccurrences.length <= 45, 'fixed repeats must stay inside the 90-day materialization horizon')
assert.ok(plantOccurrences.every(task => task.startDate <= offsetDate(90)))

await view.getState({ list: 'today', selectedTaskId: capture.id })
await view.dispatch('cancelTask', { id: capture.id })
state = await view.getState({ list: 'logbook' })
assert.equal(state.sections.flatMap(section => section.tasks).some(task => task.id === capture.id && task.status === 'canceled'), true)

const database = values.get('database')
assert.equal(database.version, 1)
assert.ok(database.tasks.length >= 5)
assert.equal(database.areas.length, 1)
assert.equal(database.projects.length, 1)
assert.ok(database.repeatTemplates.length >= 2)

await dispose()
assert.equal(commands.size, 0)
assert.equal(views.size, 0)
console.log(`[elephant-tasks] ok version=${manifest.version} tasks=${database.tasks.length} areas=${database.areas.length} projects=${database.projects.length} repeats=${database.repeatTemplates.length}`)
