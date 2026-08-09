import { reactive } from 'vue'

export const useRuntimeImageToolbar = (handlers) => {
  const state = reactive({ active: null })

  const close = () => {
    state.active = null
  }

  const open = (image) => {
    state.active = image ? { ...image } : null
  }

  const apply = async (image) => {
    await handlers.replace(image)
    close()
  }

  const remove = async (image) => {
    await handlers.remove(image)
    close()
  }

  const chooseFile = async (image) => {
    if (await handlers.chooseReplacement(image)) close()
  }

  return { state, open, close, apply, remove, chooseFile }
}
