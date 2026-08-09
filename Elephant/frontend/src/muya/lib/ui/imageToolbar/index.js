import BaseFloat from '../baseFloat'
import { patch, h } from '../../parser/render/snabbdom'
import getIcons from './config'
import { URL_REG } from '../../config'
import {
  getEditorImageToolbarItems,
  runEditorImageToolbarItem
} from '../../parser/render/renderBlock/editorExtensionRenderBridge'

import './index.css'

const defaultOptions = {
  placement: 'top',
  modifiers: {
    offset: {
      offset: '0, 10'
    }
  },
  showArrow: false
}

class ImageToolbar extends BaseFloat {
  static pluginName = 'imageToolbar'

  constructor(muya, options = {}) {
    const name = 'ag-image-toolbar'
    const opts = Object.assign({}, defaultOptions, options)
    super(muya, name, opts)
    this.oldVnode = null
    this.imageInfo = null
    this.options = opts
    this.icons = getIcons(muya?.options?.t)
    this.reference = null
    const toolbarContainer = (this.toolbarContainer = document.createElement('div'))
    this.container.appendChild(toolbarContainer)
    this.floatBox.classList.add('ag-image-toolbar-container')
    this.listen()
  }

  listen() {
    const { eventCenter } = this.muya
    super.listen()
    eventCenter.subscribe('muya-image-toolbar', ({ reference, imageInfo }) => {
      this.reference = reference
      if (reference) {
        this.imageInfo = imageInfo
        setTimeout(() => {
          this.show(reference)
          this.render()
        }, 0)
      } else {
        this.hide()
      }
    })
  }

  render() {
    const { muya, oldVnode, toolbarContainer, imageInfo } = this
    const isLocalImage = this.isLocalFile(imageInfo)
    const icons = [
      ...getIcons(muya?.options?.t),
      ...getEditorImageToolbarItems({ imageInfo, muya, isLocalImage })
    ]
    const { attrs } = imageInfo.token
    const dataAlign = attrs['data-align']
    const children = icons
      .filter((item) => !item.localOnly || isLocalImage)
      .map((item) => {
        let icon
        let iconWrapperSelector = 'div.icon-wrapper'
        if (item.icon) {
          icon = h(
            'i.icon',
            h(
              'i.icon-inner',
              {
                style: {
                  background: `url(${item.icon}) no-repeat`,
                  'background-size': '100%'
                }
              },
              ''
            )
          )
        }
        const iconWrapper = h(iconWrapperSelector, icon)
        let itemSelector = `li.item.${item.type}`

        if (item.type === 'open') {
          itemSelector += isLocalImage ? '.enable' : '.disable'
        }
        if (item.type === dataAlign || (!dataAlign && item.type === 'inline')) {
          itemSelector += '.active'
        }
        return h(
          itemSelector,
          {
            dataset: {
              tip: item.tooltip
            },
            on: {
              click: (event) => {
                this.selectItem(event, item)
              }
            }
          },
          [h('div.tooltip', item.tooltip), iconWrapper]
        )
      })

    const vnode = h('ul', children)

    if (oldVnode) {
      patch(oldVnode, vnode)
    } else {
      patch(toolbarContainer, vnode)
    }
    this.oldVnode = vnode
  }

  selectItem(event, item) {
    event.preventDefault()
    event.stopPropagation()

    const { imageInfo, muya } = this
    const isLocalImage = this.isLocalFile(imageInfo)

    if (item.addonToolbarItem) {
      runEditorImageToolbarItem(item, {
        imageInfo,
        muya,
        isLocalImage,
        reference: this.reference,
        hide: () => this.hide()
      })
      return this.hide()
    }

    switch (item.type) {
      // Delete image.
      case 'delete':
        this.muya.contentState.deleteImage(imageInfo)
        // Hide image transformer
        this.muya.eventCenter.dispatch('muya-transformer', {
          reference: null
        })
        return this.hide()
      // Edit image, for example: editor alt and title, replace image.
      case 'edit': {
        const rect = this.reference.getBoundingClientRect()
        const reference = {
          getBoundingClientRect() {
            rect.height = 0
            return rect
          }
        }
        // Hide image transformer
        this.muya.eventCenter.dispatch('muya-transformer', {
          reference: null
        })
        this.muya.eventCenter.dispatch('muya-image-selector', {
          reference,
          imageInfo,
          cb: () => {}
        })
        return this.hide()
      }
      case 'inline':
      case 'left':
      case 'center':
      case 'right': {
        this.muya.contentState.updateImage(this.imageInfo, 'data-align', item.type)
        return this.hide()
      }
      case 'open': {
        if (isLocalImage) {
          this.muya.contentState.openImage(this.imageInfo)
          this.hide()
        }
        break
      }
    }
  }

  isLocalFile(imageInfo) {
    const { attrs } = imageInfo.token
    if (URL_REG.test(imageInfo.token.src) || URL_REG.test(attrs.src)) {
      return false
    }
    return true
  }
}

export default ImageToolbar
