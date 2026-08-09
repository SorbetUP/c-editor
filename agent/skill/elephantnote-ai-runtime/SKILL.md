---
name: elephantnote-ai-runtime
description: >
  Rules for AI providers, local runtimes, model library, embeddings, OCR,
  OpenAI-compatible APIs, Ollama/LM Studio/llama.cpp, and honest capability reporting.
argument-hint: '<ai-runtime-task>'
---

# ElephantNote AI Runtime

Use this skill for chat providers, local model setup, external OpenAI-compatible endpoints, model library, embeddings, OCR, llama runtime, and AI settings UI.

## Invariants

- Provider configuration persists and is actually used by runtime calls.
- Model lists come from the selected real provider/runtime, not hardcoded demo entries.
- Local model downloads, install state, and delete actions reflect disk reality.
- Embedding and OCR providers must report unavailable/missing runtime states honestly.
- Chat/RAG must show when no local notes matched instead of inventing context.
- Secrets and API keys must not be logged or committed.

## Read first

- `Elephant/backend/js/modelRuntime.js`, `modelLibrary.js`, runtime providers, OCR runtime.
- `Elephant/shared/ai*.js`, `aiProviders.js`, `aiSetup.js`.
- Settings/model UI files.
- Tests for model library, providers, runtime, OCR, RAG prompts, and local llama setup.

## Verification

- Configure a provider and confirm the request path uses it.
- Test missing runtime/model cases and visible user errors.
- Verify model list/download/delete updates disk and UI state.
- Verify embeddings/search only claim semantic mode when an index/provider actually ran.

## Anti-slop

No fake local model, fake OCR success, fake Codex connection, or hardcoded model card presented as installed.
