import { createVNode, render } from 'vue'

export const mountSettingsComponent = (ctx, component, props = {}) => (container) => {
  if (!container) throw new Error('Settings contribution container is required')
  const host = document.createElement('div')
  host.className = 'en-system-addon-settings-host'
  container.append(host)

  const vnode = createVNode(component, props)
  if (ctx?.vueApp?._context) vnode.appContext = ctx.vueApp._context
  render(vnode, host)

  return () => {
    render(null, host)
    host.remove()
  }
}
