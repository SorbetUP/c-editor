#!/usr/bin/env node

import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { parse as parseSfc } from 'vue/compiler-sfc'

// The SFC parser already exposes the compiler AST. Keeping these stable Vue
// compiler node tags local avoids adding a second direct runtime dependency
// solely for parsing templates.
const NodeTypes = { ELEMENT: 1, SIMPLE_EXPRESSION: 4, DIRECTIVE: 7 }

const ELEMENT_MAP = new Map([
  ['div', 'rect()'],
  ['section', 'rect()'],
  ['main', 'rect()'],
  ['aside', 'rect()'],
  ['header', 'rect()'],
  ['footer', 'rect()'],
  ['span', 'label()'],
  ['p', 'paragraph()'],
  ['label', 'label()'],
  ['button', 'Button'],
  ['input', 'Input'],
  ['textarea', 'Input'],
  ['img', 'ImageViewer'],
  ['ul', 'rect().children(iterator)'],
  ['ol', 'rect().children(iterator)'],
  ['li', 'rect()'],
])

const EVENT_MAP = new Map([
  ['click', 'on_mouse_up'],
  ['mousedown', 'on_mouse_down'],
  ['mouseup', 'on_mouse_up'],
  ['mousemove', 'on_mouse_move'],
  ['keydown', 'on_key_down'],
  ['keyup', 'on_key_up'],
  ['input', 'on_input'],
  ['focus', 'on_focus'],
  ['blur', 'on_blur'],
])

const CSS_MAP = new Map([
  ['width', 'width'],
  ['height', 'height'],
  ['min-width', 'min_width'],
  ['max-width', 'max_width'],
  ['min-height', 'min_height'],
  ['max-height', 'max_height'],
  ['padding', 'padding'],
  ['margin', 'margin'],
  ['gap', 'spacing'],
  ['flex-direction', 'horizontal/vertical'],
  ['align-items', 'cross_align'],
  ['justify-content', 'main_align'],
  ['color', 'color'],
  ['background', 'background'],
  ['background-color', 'background'],
  ['border', 'border'],
  ['border-radius', 'corner_radius'],
  ['opacity', 'opacity'],
  ['font-family', 'font_family'],
  ['font-size', 'font_size'],
  ['font-weight', 'font_weight'],
  ['overflow', 'overflow'],
  ['visibility', 'visibility'],
])

const UNSUPPORTED_PATTERNS = [
  ['document.', 'DOM document access', 'replace with a native Freya event or service'],
  ['window.', 'browser window access', 'replace with a typed platform service'],
  ['querySelector', 'DOM selector lookup', 'replace with component state or accessibility IDs'],
  ['MutationObserver', 'DOM mutation observer', 'replace with explicit state subscriptions'],
  ['contentEditable', 'contentEditable editing', 'replace with freya-edit or a native editor component'],
  ['clipboard', 'browser clipboard logic', 'use Freya clipboard or the host platform service'],
  ['canvas', 'direct browser canvas', 'use Freya Canvas/Skia or an isolated web island'],
  ['localStorage', 'browser storage', 'use the Rust persistence service'],
  ['teleport', 'Vue teleport/portal', 'use Freya layers or explicit component composition'],
]

function lineAt(source, offset) {
  return source.slice(0, offset).split('\n').length
}

function componentName(filePath) {
  const base = path.basename(filePath, '.vue').replace(/[^A-Za-z0-9]+/g, ' ')
  return base
    .split(/\s+/)
    .filter(Boolean)
    .map(part => part[0].toUpperCase() + part.slice(1))
    .join('') || 'MigratedComponent'
}

function rustName(name) {
  return name
    .replace(/([a-z0-9])([A-Z])/g, '$1_$2')
    .toLowerCase()
}

function walkTemplate(node, file, result) {
  if (node.type === NodeTypes.ELEMENT) {
    const tag = node.tag.toLowerCase()
    result.elements.push({
      tag,
      line: lineAt(file.template, node.loc.start.offset),
      mapping: ELEMENT_MAP.get(tag) || null,
      unsupported: ELEMENT_MAP.has(tag) ? [] : ['custom component or unsupported HTML element'],
    })
    for (const prop of node.props) {
      if (prop.type === NodeTypes.DIRECTIVE) {
        const name = prop.name
        const argument = prop.arg?.type === NodeTypes.SIMPLE_EXPRESSION ? prop.arg.content : null
        const mapping = name === 'on' && argument ? EVENT_MAP.get(argument) : null
        result.directives.push({
          name,
          argument,
          line: lineAt(file.template, prop.loc.start.offset),
          mapping,
          status: mapping || ['if', 'for', 'bind', 'key'].includes(name) ? 'MAPPED' : 'MANUAL_PORT_REQUIRED',
        })
      }
    }
    for (const child of node.children || []) walkTemplate(child, file, result)
    return
  }
  for (const child of node.children || []) walkTemplate(child, file, result)
}

function analyzeStyles(styleContent) {
  const properties = []
  const unsupported = []
  for (const [index, rawLine] of styleContent.split('\n').entries()) {
    const line = rawLine.replace(/\/\*.*?\*\//g, '').trim()
    const match = line.match(/^([\w-]+)\s*:\s*([^;]+);?$/)
    if (!match) continue
    const [, property, value] = match
    const mapping = CSS_MAP.get(property) || null
    properties.push({ property, value: value.trim(), line: index + 1, mapping })
    if (!mapping) unsupported.push({ property, value: value.trim(), line: index + 1, reason: 'no deterministic Freya style mapping' })
  }
  return { properties, unsupported }
}

function analyzeScript(scriptContent, source) {
  const unsupported = []
  for (const [token, construction, strategy] of UNSUPPORTED_PATTERNS) {
    let offset = scriptContent.indexOf(token)
    while (offset !== -1) {
      unsupported.push({ token, construction, strategy, line: lineAt(source, offset) })
      offset = scriptContent.indexOf(token, offset + token.length)
    }
  }
  return {
    signals: {
      props: /defineProps\s*\(|props\s*[:=]/.test(scriptContent),
      emits: /defineEmits\s*\(|emit\s*\(/.test(scriptContent),
      refs: /\bref\s*\(/.test(scriptContent),
      reactive: /\breactive\s*\(/.test(scriptContent),
      computed: /\bcomputed\s*\(/.test(scriptContent),
      watchers: /\bwatch(?:Effect)?\s*\(/.test(scriptContent),
    },
    unsupported,
  }
}

export function analyzeSfc(source, filePath = 'Component.vue') {
  const parsed = parseSfc(source, { filename: filePath })
  const errors = parsed.errors.map(error => String(error))
  const descriptor = parsed.descriptor
  const template = descriptor.template?.content || ''
  const script = [descriptor.script?.content, descriptor.scriptSetup?.content].filter(Boolean).join('\n')
  const templateAnalysis = { elements: [], directives: [] }
  if (descriptor.template?.ast) walkTemplate(descriptor.template.ast, { template }, templateAnalysis)
  const styles = descriptor.styles.flatMap(style => analyzeStyles(style.content).properties)
  const unsupportedStyles = descriptor.styles.flatMap(style => analyzeStyles(style.content).unsupported)
  const scriptAnalysis = analyzeScript(script, source)
  const unsupported = [
    ...scriptAnalysis.unsupported,
    ...unsupportedStyles.map(item => ({ ...item, construction: 'CSS declaration', strategy: 'replace with a ThemeTokens/style builder' })),
    ...templateAnalysis.elements.flatMap(element => element.unsupported.map(reason => ({
      construction: `template <${element.tag}>`,
      strategy: 'provide an explicit component mapping',
      reason,
      line: element.line,
    }))),
    ...templateAnalysis.directives.filter(item => item.status === 'MANUAL_PORT_REQUIRED'),
  ]
  return {
    source: filePath,
    component: componentName(filePath),
    rustModule: rustName(componentName(filePath)),
    errors,
    template: templateAnalysis,
    script: scriptAnalysis,
    styles,
    unsupported,
    status: errors.length ? 'UNKNOWN' : unsupported.length ? 'MANUAL_PORT_REQUIRED' : 'MOSTLY_AUTO',
  }
}

export function skeletonFor(analysis) {
  const structName = analysis.component
  const children = analysis.template.elements
    .map(element => `        // <${element.tag}> -> ${element.mapping || 'manual port required'}`)
    .join('\n')
  return `use freya::prelude::*;\n\n#[derive(PartialEq)]\npub struct ${structName};\n\nimpl Component for ${structName} {\n    fn render(&self) -> impl IntoElement {\n        rect()\n${children}\n            .child("Port behavior explicitly; unsupported constructs are not dropped.")\n    }\n}\n`
}

async function main() {
  const [filePath, outputPath] = process.argv.slice(2)
  if (!filePath || filePath.startsWith('-')) {
    console.error('Usage: node tools/vue-to-freya/index.mjs <component.vue> [output.json]')
    process.exitCode = 2
    return
  }
  const source = await fs.readFile(filePath, 'utf8')
  const analysis = analyzeSfc(source, filePath)
  const result = { ...analysis, skeleton: skeletonFor(analysis) }
  const json = JSON.stringify(result, null, 2)
  if (outputPath) await fs.writeFile(outputPath, `${json}\n`)
  else console.log(json)
}

if (import.meta.url === `file://${process.argv[1]}`) await main()
