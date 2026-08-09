#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

cargo llvm-cov clean --workspace

cargo llvm-cov \
  --lib \
  --fail-under-lines 90 \
  --ignore-filename-regex '(^|/)(lib_min|main|vault_min|vault_backend|vault_backend2|tauri_extra_commands|vault/commands|vault/config|vault/entries|vault/metadata|markdown/commands|markdown/parser\.rs|model_library|chat_runtime|.*_runtime|lib)\.rs$'
