class WebGLRenderingContextStub {}
class WebGL2RenderingContextStub extends WebGLRenderingContextStub {}

if (typeof globalThis.WebGLRenderingContext === 'undefined') {
  globalThis.WebGLRenderingContext = WebGLRenderingContextStub
}

if (typeof globalThis.WebGL2RenderingContext === 'undefined') {
  globalThis.WebGL2RenderingContext = WebGL2RenderingContextStub
}

if (typeof window !== 'undefined') {
  window.WebGLRenderingContext = globalThis.WebGLRenderingContext
  window.WebGL2RenderingContext = globalThis.WebGL2RenderingContext
  if (window.HTMLCanvasElement?.prototype && !window.HTMLCanvasElement.prototype.getContext.__elephantWebglPatched) {
    const originalGetContext = window.HTMLCanvasElement.prototype.getContext
    const getContext = function getContext(type, ...args) {
      if (type === 'webgl' || type === 'experimental-webgl' || type === 'webgl2') {
        return {
          canvas: this,
          getExtension: () => null,
          getParameter: () => null,
          createBuffer: () => ({}),
          bindBuffer: () => {},
          bufferData: () => {},
          viewport: () => {},
          clearColor: () => {},
          clear: () => {},
          enable: () => {},
          disable: () => {},
          drawArrays: () => {},
          drawElements: () => {},
          getError: () => 0
        }
      }
      return originalGetContext.call(this, type, ...args)
    }
    getContext.__elephantWebglPatched = true
    window.HTMLCanvasElement.prototype.getContext = getContext
  }
}
