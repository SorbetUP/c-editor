import {
  computed,
  h,
  nextTick,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  shallowRef,
  watch
} from 'vue'

const createDomComponent = ({ name = 'ElephantPhysicalAddonView', className = '', mount }) => ({
  name,
  setup() {
    const root = ref(null)
    let dispose = null

    onMounted(async () => {
      const result = await mount?.(root.value)
      if (typeof result === 'function') dispose = result
      else if (typeof result?.dispose === 'function') dispose = () => result.dispose()
    })

    onBeforeUnmount(() => {
      try { dispose?.() } finally { dispose = null }
    })

    return () => h('div', {
      ref: root,
      class: className,
      'data-elephant-physical-addon-view': name
    })
  }
})

const getStore = (pinia, id) => pinia?._s?.get?.(id) || null

Object.defineProperty(globalThis, '__ELEPHANT_ADDON_VUE__', {
  configurable: true,
  enumerable: false,
  writable: false,
  value: Object.freeze({
    computed,
    createDomComponent,
    getStore,
    h,
    nextTick,
    onBeforeUnmount,
    onMounted,
    reactive,
    ref,
    shallowRef,
    watch
  })
})
