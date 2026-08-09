import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import { tokenizer } from '../../../../../Elephant/frontend/src/muya/lib/parser'
import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip

const editableBlocks = (muya) => {
  const output = []
  const visit = (block) => {
    if (block?.functionType === 'paragraphContent') output.push(block)
    for (const child of block?.children || []) visit(child)
  }
  for (const block of muya.contentState.getBlocks()) visit(block)
  return output
}

const findImageToken = (tokens) => {
  for (const token of tokens) {
    if (token.type === 'image') return token
    const nested = findImageToken(token.children || [])
    if (nested) return nested
  }
  return null
}

const imageInfo = (muya) => {
  const block = editableBlocks(muya).find((candidate) => candidate.text.includes('!['))
  if (!block) throw new Error('Muya image paragraph was not found.')
  const token = findImageToken(tokenizer(block.text))
  if (!token) throw new Error('Muya image token was not found.')
  return { key: block.key, token }
}

const rustImageId = (rust) => {
  const node = rust.snapshot().document.nodes.find(
    (candidate) => candidate.kind?.layer === 'inline' && candidate.kind?.value?.type === 'image'
  )
  if (!node) throw new Error('Muya Rust image node was not found.')
  return node.id
}

const cases = [
  {
    name: 'replace a Markdown image in surrounding text',
    initial: 'before ![old](old.png "Old") after',
    expected: 'before ![new alt](new%20image.png "New title") after\n',
    runJs: (muya) => muya.contentState.replaceImage(imageInfo(muya), {
      src: 'new image.png',
      alt: 'new alt',
      title: 'New title'
    }),
    runRust: (rust) => rust.request({
      type: 'replace_image',
      image: rustImageId(rust),
      source: 'new image.png',
      alt: 'new alt',
      title: 'New title'
    })
  },
  {
    name: 'delete a Markdown image from surrounding text',
    initial: 'before ![old](old.png) after',
    expected: 'before  after\n',
    runJs: (muya) => muya.contentState.deleteImage(imageInfo(muya)),
    runRust: (rust) => rust.request({
      type: 'delete_image',
      image: rustImageId(rust)
    })
  },
  {
    name: 'delete the only image in a paragraph',
    initial: '![old](old.png)',
    expected: '\n',
    runJs: (muya) => muya.contentState.deleteImage(imageInfo(muya)),
    runRust: (rust) => rust.request({
      type: 'delete_image',
      image: rustImageId(rust)
    })
  }
]

describeBundled('Muya image mutation differential traces', () => {
  let jsEditor = null

  beforeAll(initializeRustWasm)

  afterEach(() => {
    jsEditor?.destroy?.()
    jsEditor = null
    document.body.innerHTML = ''
  })

  for (const testCase of cases) {
    it(testCase.name, async () => {
      const result = await runDifferentialTrace({
        initial: testCase.initial,
        runJs: testCase.runJs,
        runRust: testCase.runRust
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(testCase.expected)
    })
  }

  it('undoes and redoes image replacement and deletion atomically', async () => {
    const initial = 'before ![old](old.png) after\n'
    const replaced = 'before ![new](new.png) after\n'
    const deleted = 'before  after\n'
    const result = await runDifferentialTrace({
      initial: 'before ![old](old.png) after',
      runJs: (muya) => {
        muya.contentState.replaceImage(imageInfo(muya), {
          src: 'new.png',
          alt: 'new',
          title: ''
        })
        const afterReplace = muya.getMarkdown()
        muya.undo()
        const replaceUndo = muya.getMarkdown()
        muya.redo()
        const replaceRedo = muya.getMarkdown()
        muya.contentState.deleteImage(imageInfo(muya))
        const afterDelete = muya.getMarkdown()
        muya.undo()
        const deleteUndo = muya.getMarkdown()
        muya.redo()
        return [
          afterReplace,
          replaceUndo,
          replaceRedo,
          afterDelete,
          deleteUndo,
          muya.getMarkdown()
        ]
      },
      runRust: (rust) => {
        const image = rustImageId(rust)
        rust.request({
          type: 'replace_image',
          image,
          source: 'new.png',
          alt: 'new',
          title: null
        })
        const afterReplace = rust.markdown()
        rust.request({ type: 'undo' })
        const replaceUndo = rust.markdown()
        rust.request({ type: 'redo' })
        const replaceRedo = rust.markdown()
        rust.request({ type: 'delete_image', image })
        const afterDelete = rust.markdown()
        rust.request({ type: 'undo' })
        const deleteUndo = rust.markdown()
        rust.request({ type: 'redo' })
        return [
          afterReplace,
          replaceUndo,
          replaceRedo,
          afterDelete,
          deleteUndo,
          rust.markdown()
        ]
      }
    })
    jsEditor = result.jsEditor
    expect(result.jsResult).toEqual(result.rustResult)
    expect(result.rustResult).toEqual([
      replaced,
      initial,
      replaced,
      deleted,
      replaced,
      deleted
    ])
  })
})
