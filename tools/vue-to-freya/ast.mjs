import { parse as parseSfc } from '../../Elephant/node_modules/@vue/compiler-sfc/dist/compiler-sfc.cjs.js'
import { baseParse, NodeTypes } from '../../Elephant/node_modules/@vue/compiler-dom/index.js'
import { parse as parseJavaScript } from '../../Elephant/node_modules/@babel/parser/lib/index.js'
import postcss from '../../Elephant/node_modules/postcss/lib/postcss.js'

const ELEMENT_MAP = new Map([
  ['div', 'rect'],
  ['section', 'rect'],
  ['main', 'rect'],
  ['aside', 'rect'],
  ['header', 'rect'],
  ['nav', 'rect'],
  ['article', 'rect'],
  ['span', 'label'],
  ['strong', 'label'],
  ['small', 'label'],
  ['p', 'label'],
  ['h1', 'label'],
  ['h2', 'label'],
  ['h3', 'label'],
  ['button', 'button'],
  ['input', 'input'],
  ['img', 'image'],
  ['template', 'fragment']
])

const STYLE_MAP = new Map([
  ['width', 'width'],
  ['height', 'height'],
  ['min-width', 'min_width'],
  ['max-width', 'max_width'],
  ['min-height', 'min_height'],
  ['max-height', 'max_height'],
  ['padding', 'padding'],
  ['margin', 'margin'],
  ['gap', 'spacing'],
  ['flex-direction', 'direction'],
  ['align-items', 'cross_align'],
  ['justify-content', 'main_align'],
  ['color', 'color'],
  ['background', 'background'],
  ['background-color', 'background'],
  ['border', 'border'],
  ['border-radius', 'with_corner_radius'],
  ['opacity', 'opacity'],
  ['font-size', 'font_size'],
  ['font-weight', 'font_weight'],
  ['overflow', 'overflow']
])

function lineOf(source, offset) {
  return source.slice(0, offset).split('\n').length
}

function attr(node, name) {
  return node.props?.find((prop) => prop.type === NodeTypes.ATTRIBUTE && prop.name === name)
}

function directive(node, name) {
  return node.props?.find((prop) => prop.type === NodeTypes.DIRECTIVE && prop.name === name)
}

function directiveNames(node) {
  return (node.props || [])
    .filter((prop) => prop.type === NodeTypes.DIRECTIVE)
    .map((prop) => {
      const argument = prop.arg?.content || ''
      return argument ? `@${prop.name}:${argument}` : `@${prop.name}`
    })
}

function expressionContent(prop) {
  return prop?.exp?.content || null
}

function convertNode(node, source, unsupported) {
  if (node.type === NodeTypes.TEXT) {
    return { kind: 'text', value: node.content, line: lineOf(source, node.loc.start.offset) }
  }
  if (node.type === NodeTypes.INTERPOLATION) {
    return {
      kind: 'interpolation',
      expression: node.content.content,
      line: lineOf(source, node.loc.start.offset)
    }
  }
  if (node.type !== NodeTypes.ELEMENT && node.type !== NodeTypes.TEMPLATE) {
    unsupported.push({
      kind: `template-node:${node.type}`,
      line: lineOf(source, node.loc.start.offset),
      reason: 'Vue compiler node has no deterministic Freya mapping.'
    })
    return { kind: 'unsupported', nodeType: node.type }
  }

  const name = node.tag
  const mapped = ELEMENT_MAP.get(name) || (name[0] === name[0]?.toUpperCase() ? 'component' : null)
  if (!mapped) {
    unsupported.push({
      kind: `element:${name}`,
      line: lineOf(source, node.loc.start.offset),
      reason: 'Custom/unknown HTML element requires a named Freya component mapping.'
    })
  }

  const ifDirective = directive(node, 'if')
  const forDirective = directive(node, 'for')
  const bindAttributes = (node.props || [])
    .filter((prop) => prop.type === NodeTypes.ATTRIBUTE || (prop.type === NodeTypes.DIRECTIVE && prop.name === 'bind'))
    .map((prop) => {
      if (prop.type === NodeTypes.ATTRIBUTE) return { name: prop.name, value: prop.value?.content ?? true }
      return { name: `:${prop.arg?.content || '*'}`, value: expressionContent(prop) }
    })
  const events = directiveNames(node).filter((name) => name.startsWith('@on') || name.startsWith('@click') || name.startsWith('@input') || name.startsWith('@keydown') || name.startsWith('@pointer') || name.startsWith('@drag'))
  const children = node.children?.map((child) => convertNode(child, source, unsupported)) || []

  if (directiveNames(node).some((name) => name.startsWith('@slot') || name.includes('v-slot'))) {
    unsupported.push({
      kind: 'slot',
      line: lineOf(source, node.loc.start.offset),
      reason: 'Slots need explicit Freya child composition.'
    })
  }

  return {
    kind: 'element',
    sourceTag: name,
    freya: mapped || 'manual-component',
    line: lineOf(source, node.loc.start.offset),
    attributes: bindAttributes,
    directives: directiveNames(node),
    events,
    conditional: expressionContent(ifDirective),
    iteration: expressionContent(forDirective),
    children
  }
}

function parseScript(script, source) {
  if (!script?.content.trim()) return { imports: [], state: [], calls: [], unsupported: [] }
  const unsupported = []
  let ast
  try {
    ast = parseJavaScript(script.content, {
      sourceType: 'module',
      plugins: ['typescript', 'jsx', 'topLevelAwait', 'decorators-legacy']
    })
  } catch (error) {
    return {
      imports: [],
      state: [],
      calls: [],
      unsupported: [{ kind: 'script-parse', line: lineOf(source, script.loc.start.offset), reason: error.message }]
    }
  }

  const imports = []
  const state = []
  const calls = []
  for (const statement of ast.program.body) {
    if (statement.type === 'ImportDeclaration') {
      imports.push({ source: statement.source.value, line: lineOf(source, script.loc.start.offset + statement.start) })
      continue
    }
    const text = script.content.slice(statement.start, statement.end)
    if (/\b(ref|reactive|computed|watch|provide|inject)\s*\(/.test(text)) {
      state.push({ code: text.trim(), line: lineOf(source, script.loc.start.offset + statement.start) })
    }
  }

  const sourceText = script.content
  for (const match of sourceText.matchAll(/\b(document|window|MutationObserver|URL|Blob|FileReader|Range|navigator|contentEditable|canvas|querySelector(?:All)?)\b/g)) {
    const token = match[1]
    unsupported.push({
      kind: `web-api:${token}`,
      line: lineOf(source, script.loc.start.offset + match.index),
      reason: 'Requires an explicit native Freya/platform adapter or isolated web island.'
    })
  }
  for (const match of sourceText.matchAll(/\b([A-Za-z_$][\w$]*)\s*\(/g)) {
    const name = match[1]
    if (!['ref', 'reactive', 'computed', 'watch', 'provide', 'inject'].includes(name)) {
      calls.push({ name, line: lineOf(source, script.loc.start.offset + match.index) })
    }
  }
  return { imports, state, calls, unsupported }
}

function parseStyles(styles, source) {
  const mappings = []
  const unsupported = []
  for (const style of styles) {
    let root
    try {
      root = postcss.parse(style.content)
    } catch (error) {
      unsupported.push({ kind: 'style-parse', line: lineOf(source, style.loc.start.offset), reason: error.message })
      continue
    }
    root.walkRules((rule) => {
      if (/[>:]/.test(rule.selector) || /::v-deep|:global|:deep/.test(rule.selector)) {
        unsupported.push({
          kind: 'css-selector',
          selector: rule.selector,
          line: lineOf(source, style.loc.start.offset + rule.source.start.offset),
          reason: 'Pseudo/relational selector needs an explicit native component boundary.'
        })
      }
      const declarations = []
      rule.walkDecls((declaration) => {
        const freya = STYLE_MAP.get(declaration.prop)
        if (!freya) {
          unsupported.push({
            kind: `css-property:${declaration.prop}`,
            selector: rule.selector,
            line: lineOf(source, style.loc.start.offset + declaration.source.start.offset),
            reason: 'No deterministic Freya style mapping is registered.'
          })
          return
        }
        declarations.push({ css: declaration.prop, freya, value: declaration.value })
      })
      mappings.push({ selector: rule.selector, line: lineOf(source, style.loc.start.offset + rule.source.start.offset), declarations })
    })
  }
  return { mappings, unsupported }
}

function rustComponentName(filename) {
  const stem = filename.replace(/\.vue$/i, '').replace(/[^A-Za-z0-9]+(.)?/g, (_, char) => char ? char.toUpperCase() : '')
  return stem || 'ConvertedComponent'
}

function rustText(node) {
  if (node.kind === 'text') return node.value.trim() ? `label().text(${JSON.stringify(node.value.trim())})` : 'rect()'
  if (node.kind === 'interpolation') return `label().text(${JSON.stringify(`{{ ${node.expression} }`)})`
  if (node.kind === 'unsupported') return 'rect().a11y_alt("unsupported Vue node")'
  const target = node.freya === 'label' ? 'label()' : node.freya === 'button' ? 'button()' : 'rect()'
  const attributes = node.attributes.filter(({ name }) => !name.startsWith(':') && name !== 'class' && name !== 'style')
    .map(({ name, value }) => name === 'aria-label' ? `.a11y_alt(${JSON.stringify(String(value))})` : '')
    .filter(Boolean)
  const events = node.events.map((event) => `.on_press(/* convert ${event} */)`)
  const children = node.children.map(rustText).filter(Boolean)
  const body = [target, ...attributes, ...events, ...children.map((child) => `.child(${child})`)].join('\n    ')
  return node.conditional ? `if ${node.conditional} { ${body} } else { rect() }` : body
}

export function analyzeVueFile(filename, source) {
  const parsed = parseSfc(source, { filename })
  const unsupported = []
  const template = parsed.descriptor.template
  const templateAst = template
    ? baseParse(template.content, {
      isVoidTag: (tag) => ['area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'param', 'source', 'track', 'wbr'].includes(tag)
    })
    : null
  const templateNodes = templateAst?.children.map((node) => convertNode(node, source, unsupported)) || []
  const script = parseScript(parsed.descriptor.scriptSetup || parsed.descriptor.script, source)
  const styles = parseStyles(parsed.descriptor.styles, source)
  return {
    source: filename,
    target: `Elephant/freya/src/components/${rustComponentName(filename)}.rs`,
    sourceBlocks: {
      template: Boolean(template),
      script: Boolean(parsed.descriptor.script || parsed.descriptor.scriptSetup),
      styles: parsed.descriptor.styles.length
    },
    template: templateNodes,
    script,
    styles,
    unsupported: [...unsupported, ...script.unsupported, ...styles.unsupported],
    skeleton: `// Generated from ${filename}; review unsupported constructs before wiring.\npub fn ${rustComponentName(filename)}() -> Element {\n    ${templateNodes.map(rustText).join('\n    ')}\n}`
  }
}
