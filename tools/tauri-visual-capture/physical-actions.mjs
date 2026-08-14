import { writeFileSync } from 'node:fs'
import path from 'node:path'

import { dispatchNativeAction, inspectAccessibility } from './native-window.mjs'

export class MissingPhysicalTargetError extends Error {
  constructor (actionId, message) {
    super(`${actionId}: ${message}`)
    this.name = 'MissingPhysicalTargetError'
    this.actionId = actionId
  }
}
const roleMap = { button: 'AXButton', menuitem: 'AXMenuItem', textbox: 'AXTextField' }
const textOf = (element) => [element.title, element.description, element.value].filter(Boolean).join(' ')
const area = (rect) => Math.max(0, Number(rect?.width) || 0) * Math.max(0, Number(rect?.height) || 0)
const center = (rect) => ({ x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 })

const elementsFor = (report, target, actionId) => {
  const tauri = target?.tauri || {}
  const candidates = Array.isArray(report?.elements) ? report.elements.filter((element) => element.rect && area(element.rect) > 1) : []
  const expectedRole = roleMap[tauri.role]
  let matches = candidates
  if (expectedRole) matches = matches.filter((element) => element.role === expectedRole)
  const exactName = tauri.name ?? tauri.value
  if (exactName) matches = matches.filter((element) => textOf(element) === exactName)
  if (tauri.placeholder) matches = matches.filter((element) => textOf(element).includes(tauri.placeholder))
  if (tauri.hasText) matches = matches.filter((element) => textOf(element).includes(tauri.hasText))
  if (tauri.selector?.includes('aria-label="Search"')) matches = matches.filter((element) => textOf(element) === 'Search')
  if (tauri.selector?.includes('en-note-editor-shell')) matches = matches.filter((element) => element.role === 'AXWebArea' || element.role === 'AXGroup')
  if (tauri.selector?.includes('muya-runtime-editor')) matches = matches.filter((element) => ['AXTextArea', 'AXTextField', 'AXWebArea'].includes(element.role))
  if (tauri.selector?.includes('en-rail-sidebar-toggle')) matches = matches.filter((element) => /sidebar|hide/i.test(textOf(element)))
  if (tauri.selector?.includes('en-rail-nav')) matches = matches.filter((element) => textOf(element) === 'Search')
  if (matches.length === 1) return matches[0]
  if (matches.length > 1) {
    const ranked = [...matches].sort((left, right) => area(right.rect) - area(left.rect))
    if (area(ranked[0].rect) > area(ranked[1].rect) * 1.2) return ranked[0]
  }
  throw new MissingPhysicalTargetError(actionId, `no unambiguous macOS Accessibility target for ${JSON.stringify(tauri)}`)
}

const cssTarget = (target) => {
  const tauri = target?.tauri || target || {}
  if (tauri.strategy === 'placeholder') return { selector: `[placeholder="${tauri.value}"]` }
  if (tauri.strategy === 'testid') return { selector: `[data-testid="${tauri.value}"]` }
  if (tauri.strategy === 'role') {
    if (tauri.role === 'button' && tauri.name === 'All notes') return { selector: 'button.en-all-notes' }
    return { selector: `${tauri.role === 'button' ? 'button' : `[role="${tauri.role}"]`}[aria-label="${tauri.name}"]` }
  }
  if (tauri.strategy === 'locator-filter') return { selector: tauri.selector, text: tauri.hasText }
  if (tauri.strategy === 'css') return { selector: tauri.selector }
  return null
}

const translated = (rect, windowBounds, scale = { x: 1, y: 1 }) => ({
  x: Number(windowBounds?.x || 0) + Number(rect.x || 0) * scale.x,
  y: Number(windowBounds?.y || 0) + Number(rect.y || 0) * scale.y,
  width: Number(rect.width || 0) * scale.x,
  height: Number(rect.height || 0) * scale.y
})

export const translateObservedRect = ({ rect, windowBounds, viewportRect }) => {
  // The acceptance DOM is rooted at the native window origin. contentBounds is
  // only the capture crop and must never be used to position input events.
  const scale = {
    x: viewportRect?.width > 0 && windowBounds?.width > 0 ? Number(windowBounds.width) / Number(viewportRect.width) : 1,
    y: viewportRect?.height > 0 && windowBounds?.height > 0 ? Number(windowBounds.height) / Number(viewportRect.height) : 1
  }
  return { rect: translated(rect, windowBounds, scale), scale }
}

const observedElement = async (client, target, actionId, windowBounds) => {
  const query = cssTarget(target)
  if (!query || !client) return null
  const observation = await client.command('readDom', query.selector, query.text ?? null)
  if (!observation?.exists || !observation.visible || !observation.rect || area(observation.rect) <= 1) {
    throw new MissingPhysicalTargetError(actionId, `no visible observation target for ${JSON.stringify(query)}`)
  }
  const viewport = await client.command('readDom', '.en-shell')
  const translatedObservation = translateObservedRect({ rect: observation.rect, windowBounds, viewportRect: viewport?.rect })
  return { ...translatedObservation, cssRect: observation.rect }
}

const resolveElement = async ({ report, client, target, actionId, windowBounds }) => {
  if (client && cssTarget(target)) return observedElement(client, target, actionId, windowBounds)
  return elementsFor(report, target, actionId)
}

const pointPath = (rect, shape = []) => {
  const point = center(rect)
  return shape.map((name) => {
    if (name === 'center-plus-x-24') return { x: point.x + 24, y: point.y }
    if (name === 'center-plus-y-120') return { x: point.x, y: point.y + 120 }
    if (name === 'center-plus-y-240') return { x: point.x, y: point.y + 240 }
    if (name === 'left') return { x: rect.x + rect.width * 0.2, y: point.y }
    if (name === 'right') return { x: rect.x + rect.width * 0.8, y: point.y }
    return point
  })
}

const dragPoints = (source, drop) => {
  const start = center(source)
  const end = center(drop)
  return [start, { x: (start.x + end.x) / 2, y: (start.y + end.y) / 2 }, end]
}

const dispatch = (request, requestDir, events) => {
  const filename = path.join(requestDir, `${events.actionId || 'action'}-${String(events.length).padStart(3, '0')}-${request.operation}.json`)
  const nativePoints = (request.points || []).map((point) => [Number(point.x), Number(point.y)])
  const nativeRequest = request.points
    ? { ...request, processId: events.processId, points: nativePoints }
    : { ...request, processId: events.processId }
  writeFileSync(filename, `${JSON.stringify(nativeRequest, null, 2)}\n`, 'utf8')
  const startedAt = Date.now()
  const result = dispatchNativeAction({ requestFile: filename })
  const endedAt = Date.now()
  events.push({ operation: request.operation, request: { ...request, requestFile: filename, nativePoints }, startedAt, endedAt, result })
}

export const observeAccessibility = (pid) => inspectAccessibility({ pid })

export const resolvePhysicalEvent = (action) => action.event || (action.target ? 'move-pointer' : null)

export const executePhysicalAction = async ({ action, pid, requestDir, client, windowBounds }) => {
  const report = inspectAccessibility({ pid })
  if (!report.accessibilityTrusted) throw new MissingPhysicalTargetError(action.id, 'macOS Accessibility trust is unavailable; bridge control is forbidden')
  const events = []
  events.processId = pid
  events.actionId = action.id
  const target = action.target
  const event = resolvePhysicalEvent(action)
  if (event === 'press-key') {
    dispatch({ operation: 'press-key', key: action.key, repeatCount: action.repeat || 1 }, requestDir, events)
  } else if (event === 'write-text' || event === 'focus-write-text') {
    const element = await resolveElement({ report, client, target, actionId: action.id, windowBounds })
    dispatch({ operation: 'click', points: [center(element.rect)] }, requestDir, events)
    for (const key of action.keysBeforeText || []) {
      dispatch({ operation: 'press-key', key, control: key.startsWith('Control+') }, requestDir, events)
    }
    dispatch({ operation: 'write-text', text: action.input || action.text || '' }, requestDir, events)
  } else if (event === 'drag') {
    const source = await resolveElement({ report, client, target: target?.source, actionId: action.id, windowBounds })
    const dropTarget = await resolveElement({ report, client, target: target?.dropTarget, actionId: action.id, windowBounds })
    dispatch({ operation: 'drag', points: dragPoints(source.rect, dropTarget.rect) }, requestDir, events)
  } else if (event === 'scroll') {
    const element = await resolveElement({ report, client, target, actionId: action.id, windowBounds })
    const points = pointPath(element.rect, action.pointerPath || ['center'])
    dispatch({ operation: 'move-pointer', points }, requestDir, events)
    dispatch({ operation: 'scroll', points: [points.at(-1)], deltaY: action.delta?.y || 0 }, requestDir, events)
  } else if (event === 'move-pointer') {
    const element = await resolveElement({ report, client, target, actionId: action.id, windowBounds })
    dispatch({ operation: 'move-pointer', points: pointPath(element.rect, action.pointerPath || ['center']) }, requestDir, events)
  } else if (event === 'click') {
    const element = await resolveElement({ report, client, target, actionId: action.id, windowBounds })
    dispatch({ operation: 'click', points: [center(element.rect)] }, requestDir, events)
  } else if (event) {
    throw new MissingPhysicalTargetError(action.id, `unsupported shared physical event ${event}`)
  }
  return {
    accessibilityTrusted: report.accessibilityTrusted,
    events,
    controlPlane: 'native-cg-event',
    bridgeFallback: false,
    observedTargets: events.flatMap((event) => event.request.points || [])
  }
}
