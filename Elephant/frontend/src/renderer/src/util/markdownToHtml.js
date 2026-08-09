import { ElephantRustBridge } from '../editor-rust/bridge'
import { ElephantRustDomRenderer } from '../editor-rust/domRenderer'
import { createBundledElephantRustEngine } from '../editor-rust/wasmFactory'

const markdownToHtml = async (markdown) => {
  const container = document.createElement('div')
  const renderer = new ElephantRustDomRenderer(container)
  const engine = await createBundledElephantRustEngine(String(markdown || ''))
  const bridge = new ElephantRustBridge(engine, {
    applySnapshot: (snapshot) => renderer.applySnapshot(snapshot)
  })
  await bridge.snapshot()
  return `<article class="markdown-body">${container.innerHTML}</article>`
}

export default markdownToHtml
