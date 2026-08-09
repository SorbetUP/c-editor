const normalizeKey = (value) => String(value || '').trim()

const modifierLabelForEvent = (event) => {
  const parts = []
  if (event.metaKey) parts.push('Cmd')
  if (event.ctrlKey) parts.push('Ctrl')
  if (event.altKey) parts.push('Alt')
  if (event.shiftKey) parts.push('Shift')
  return parts
}

const normalizeEventKey = (event) => {
  const key = normalizeKey(event?.key)
  if (!key) return ''

  const lower = key.toLowerCase()
  const aliases = {
    control: 'Ctrl',
    shift: 'Shift',
    alt: 'Alt',
    meta: 'Cmd',
    os: 'Cmd',
    cmd: 'Cmd',
    command: 'Cmd',
    esc: 'Esc',
    escape: 'Esc',
    enter: 'Enter',
    return: 'Enter',
    spacebar: 'Space',
    ' ': 'Space',
    arrowup: 'Up',
    arrowdown: 'Down',
    arrowleft: 'Left',
    arrowright: 'Right'
  }

  if (aliases[lower]) return aliases[lower]
  if (key.length === 1) return key.toUpperCase()
  return key[0].toUpperCase() + key.slice(1)
}

export const isCompositionEvent = (event) => {
  return !!event?.isComposing || event?.key === 'Process' || event?.keyCode === 229
}

export const isValidAccelerator = (accelerator) => {
  const value = String(accelerator || '').trim()
  if (!value) return false

  const parts = value.split('+').map((part) => part.trim()).filter(Boolean)
  if (!parts.length) return false

  const modifiers = new Set(['Cmd', 'Ctrl', 'Alt', 'Shift', 'Option', 'Super'])
  const last = parts[parts.length - 1]
  if (!last) return false

  const modifierOnly = parts.every((part) => modifiers.has(part))
  if (modifierOnly) return false

  return parts.every((part, index) => {
    if (index === parts.length - 1) return part.length > 0
    return modifiers.has(part)
  })
}

export const getAcceleratorFromKeyboardEvent = (event) => {
  const modifiers = modifierLabelForEvent(event)
  const key = normalizeEventKey(event)
  const isValid = !!key && !['Cmd', 'Ctrl', 'Alt', 'Shift'].includes(key)

  return {
    accelerator: [...modifiers, key].filter(Boolean).join('+'),
    isValid
  }
}

export const setKeyboardLayout = () => {}
