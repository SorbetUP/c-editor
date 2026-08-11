const ACTION_TIMEOUT = 15000

const fail = (action, target) => {
  throw new Error(`source-playwright action ${action.id} target is missing: ${target}`)
}

async function requireLocator (locator, action, target) {
  const count = await locator.count()
  if (count === 0) fail(action, target)
  const candidate = locator.first()
  if (!(await candidate.isVisible({ timeout: ACTION_TIMEOUT }).catch(() => false))) fail(action, target)
  return candidate
}

async function waitForVisible (locator, action, target) {
  const candidate = locator.first()
  try {
    await candidate.waitFor({ state: 'visible', timeout: ACTION_TIMEOUT })
  } catch {
    fail(action, target)
  }
  return candidate
}

async function point (locator, action, target, token = 'center') {
  const box = await locator.boundingBox()
  if (!box) fail(action, target)
  const center = { x: box.x + box.width / 2, y: box.y + box.height / 2 }
  if (token === 'center-plus-x-24') return { x: center.x + 24, y: center.y }
  if (token === 'center-plus-y-120') return { x: center.x, y: center.y + 120 }
  if (token === 'center-plus-y-240') return { x: center.x, y: center.y + 240 }
  if (token === 'left') return { x: box.x + 2, y: center.y }
  if (token === 'right') return { x: box.x + box.width - 2, y: center.y }
  return center
}

async function movePath (page, locator, action, target, tokens, durationMs) {
  const segmentMs = Math.max(1, durationMs / Math.max(1, tokens.length - 1))
  for (const token of tokens) {
    await page.mouse.move((await point(locator, action, target, token)).x, (await point(locator, action, target, token)).y)
    await new Promise((resolve) => setTimeout(resolve, segmentMs))
  }
}

async function clickRole (page, action, role, name) {
  const locator = await requireLocator(page.getByRole(role, { name, exact: true }), action, `${role}:${name}`)
  await locator.click()
}

async function card (page, action) {
  return requireLocator(page.locator('.en-note-card').filter({ hasText: 'Alpha note' }), action, '.en-note-card hasText=Alpha note')
}

export async function dispatchAction (page, action) {
  switch (action.id) {
    case 'move-to-alpha-card': {
      const target = await card(page, action)
      await movePath(page, target, action, 'Alpha note card', ['center', 'center-plus-x-24', 'center'], action.pointer?.durationMs || 400)
      return
    }
    case 'open-search':
      await clickRole(page, action, 'button', 'Search')
      await waitForVisible(page.getByPlaceholder('Search notes, paths, tags, or ideas…'), action, 'search placeholder')
      return
    case 'search-alpha': {
      const input = await requireLocator(page.getByPlaceholder('Search notes, paths, tags, or ideas…'), action, 'search placeholder')
      await input.click()
      await input.pressSequentially(action.input, { delay: 0 })
      return
    }
    case 'close-search': {
      const input = await requireLocator(page.getByPlaceholder('Search notes, paths, tags, or ideas…'), action, 'search placeholder')
      for (let index = 0; index < (action.repeat || 1); index += 1) await input.press('Escape')
      return
    }
    case 'navigate-all-notes':
      await clickRole(page, action, 'button', 'All notes')
      return
    case 'open-alpha-note':
      await (await card(page, action)).click()
      await waitForVisible(page.getByTestId('muya-runtime-editor'), action, 'testid:muya-runtime-editor')
      return
    case 'edit-alpha-note': {
      const editor = await requireLocator(page.getByTestId('muya-runtime-editor'), action, 'testid:muya-runtime-editor')
      await editor.click()
      for (const key of action.keysBeforeText || []) await page.keyboard.press(key)
      await editor.pressSequentially(action.text, { delay: 0 })
      return
    }
    case 'scroll-alpha-note': {
      const target = await requireLocator(page.locator('.en-editor-host .editor-component'), action, '.en-editor-host .editor-component')
      const tokens = action.pointerPath || ['center']
      const first = await point(target, action, '.en-editor-host .editor-component', tokens[0])
      await page.mouse.move(first.x, first.y)
      await page.mouse.wheel(action.delta.x, action.delta.y)
      const segmentMs = Math.max(1, (action.durationMs || 400) / Math.max(1, tokens.length - 1))
      for (const token of tokens.slice(1)) {
        await new Promise((resolve) => setTimeout(resolve, segmentMs))
        const next = await point(target, action, '.en-editor-host .editor-component', token)
        await page.mouse.move(next.x, next.y)
      }
      return
    }
    case 'close-alpha-note':
      await clickRole(page, action, 'button', 'Close note')
      return
    case 'open-create-menu':
      await clickRole(page, action, 'button', 'Create')
      return
    case 'move-through-create-menu': {
      const target = await requireLocator(page.getByRole('menuitem', { name: /^Note\b/ }), action, 'menuitem:Note')
      await movePath(page, target, action, 'menuitem:Note', action.pointerPath || ['left', 'center', 'right', 'center'], action.durationMs || 400)
      return
    }
    case 'close-create-menu':
      await page.keyboard.press(action.key || 'Escape')
      return
    case 'drag-search-rail-item': {
      const source = await requireLocator(page.locator('.en-rail-nav .en-rail-icon[aria-label="Search"]'), action, 'rail Search')
      const target = await requireLocator(page.locator('.en-rail-nav .en-rail-sidebar-toggle'), action, 'rail sidebar toggle')
      const start = await point(source, action, 'rail Search')
      const middle = { x: (start.x + (await point(target, action, 'rail sidebar toggle')).x) / 2, y: (start.y + (await point(target, action, 'rail sidebar toggle')).y) / 2 }
      const end = await point(target, action, 'rail sidebar toggle')
      await page.mouse.move(start.x, start.y)
      await page.mouse.down()
      await new Promise((resolve) => setTimeout(resolve, 50))
      await page.mouse.move(middle.x, middle.y, { steps: 8 })
      await new Promise((resolve) => setTimeout(resolve, 100))
      await page.mouse.move(end.x, end.y, { steps: 8 })
      await target.drop({ data: { 'text/plain': 'search' } })
      await page.mouse.up()
      await new Promise((resolve) => setTimeout(resolve, 150))
      return
    }
    default:
      throw new Error(`source-playwright does not implement shared action ${action.id}`)
  }
}
