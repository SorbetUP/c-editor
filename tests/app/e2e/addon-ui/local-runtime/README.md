# Real local addon runtime tests

Run the three production-sidecar checks explicitly with:

```bash
NODE_PATH=Elephant/node_modules pnpm exec playwright test tests/app/e2e/addon-ui/local-runtime/real-local-runtime.spec.js \
  --config tests/app/e2e/playwright.config.js --workers=1 --reporter=line
```

The first successful run downloads the 105 MB `SmolLM2-135M-Instruct-Q4_K_M.gguf`
through the real `elephant.open-models` service. The file and a SHA-256 manifest
are kept in `ELEPHANT_E2E_LOCAL_RUNTIME_CACHE` (or the platform cache directory)
and later runs assert that the service discovers and reuses the same file without
calling `models.download` again.

The tests start the actual Open Models and Knowledge Rust services, the bundled
`llama-server`, and the OCR process sidecar. OCR deliberately uses Tesseract
because that is the production path declared by its manifest; it does not claim
that the GGUF model is used by OCR.
