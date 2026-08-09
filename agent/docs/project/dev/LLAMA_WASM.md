# WASM Llama Runtime

This repository now contains a small WASM-oriented Llama runtime under `Elephant/shared/ai/`.

The shared runtime is browser-safe. Callers provide the WASM config paths, so the app can point them at bundled assets, a local static server, or a CDN.

The public entrypoint is `Elephant/shared/ai/index.js`, which exports the shared backend helpers, the WASM runtime, and the auto runtime.

## Backend order

The runtime resolves backends in this order:

1. `cpu`
2. `gpu`
3. `mpu`
4. `npu`
5. `openvino`

## Current behavior

- In the WASM runtime, only `cpu` and `gpu` are executable today.
- `gpu` is exposed through WebGPU when the browser runtime has `navigator.gpu`.
- `mpu` is normalized as a Metal/MPS-style alias, but remains unsupported in the WASM runtime.
- `npu` and `openvino` are reported as unsupported in the WASM runtime and are skipped by the selector.
- `tpu` is excluded from the active backend order for now; if it is requested explicitly, the auto runtime falls back to `cpu` and exposes a fallback notice in the returned session.

## Node runtime

- The Electron/Node runtime reuses `node-llama-cpp`.
- Its backend report is derived from the installed `llama.cpp` build and `getLlamaGpuTypes()`.
- `GGML_OPENVINO=ON` in `cmakeOptions` enables the OpenVINO path when the local build supports it.
- The runtime still preserves the same backend preference order: `cpu`, `gpu`, `mpu`, `npu`, `openvino`.

## Auto runtime

- `AutoLlamaRuntime` selects the browser WASM path when running in a browser-like environment.
- It selects the Node `node-llama-cpp` path outside the browser.
- The returned session exposes the same `chat()`, `complete()`, and `embed()` methods regardless of which runtime is selected.
- The returned session can include a `fallbackNotice` when a requested backend is not supported and a CPU fallback is used.

## Local example

Use `scripts/llama-wasm-demo.mjs` with a local `.gguf` model file:

```bash
node scripts/llama-wasm-demo.mjs --model /path/to/model.gguf --prompt "Say hello"
```

The example resolves the local `@wllama/wllama` WASM assets from the installed package, loads the model from disk, and prints the generated text.
