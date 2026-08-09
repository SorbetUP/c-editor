import { installExecutableCodeNativeRuntime } from './executableCodeNativeRuntime'
import { installExecutableCodeNativeLifecycle } from './executableCodeNativeLifecycle'

export const installExecutableCodeBlocks = (target = globalThis) => {
  const legacy = target.__ELEPHANT_CODE_RUNTIME__
  if (legacy?.version?.startsWith?.('v')) {
    legacy.dispose?.()
    delete target.__ELEPHANT_CODE_RUNTIME__
  }

  return installExecutableCodeNativeLifecycle(installExecutableCodeNativeRuntime(target), target)
}

export const resetExecutableCodeNativeRuntimeForTests = (target = globalThis) => {
  target.__ELEPHANT_NATIVE_CODE_RUNTIME__?.dispose?.()
  delete target.__ELEPHANT_NATIVE_CODE_RUNTIME__
  delete target.__ELEPHANT_CODE_RUNTIME__
}
