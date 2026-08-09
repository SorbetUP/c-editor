#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT_DIR"

cargo check --manifest-path Elephant/backend/tauri/Cargo.toml --lib
cargo test --manifest-path Elephant/backend/tauri/Cargo.toml --lib

if command -v cargo-llvm-cov >/dev/null 2>&1; then
  bash Elephant/backend/tauri/coverage.sh
else
  echo "cargo-llvm-cov is not installed; skipping local build/coverage."
  echo "Install it with: cargo install cargo-llvm-cov"
fi
