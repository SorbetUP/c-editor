# Model Library

The model library lives in `Elephant/back/app/modelLibrary.js` and is exposed through the existing `ModelRuntime` facade.

## What it does

- Lists local GGUF models from the configured model directory.
- Downloads models through `node-llama-cpp`, including Hugging Face repo support.
- Streams download progress and supports cancel by `downloadId` through the Electron IPC bridge.
- Activates a model by loading it through the local runtime.
- Deactivates a model and releases the underlying loaded model when possible.
- Deletes a local model file and its sidecar manifest.
- Searches Hugging Face model metadata and fetches detailed model information.

## Notes

- Hugging Face search uses the public Hub API.
- Downloaded models store a JSON sidecar next to the model file so the library can preserve source metadata and active state.
- Local model listings are cached in `model-index.json` next to the model directory to avoid rescanning on every refresh.
- Hugging Face search results are cached in the same local index file with a short TTL.
- The library is intended for the Electron/Node runtime, not the browser WASM runtime.
