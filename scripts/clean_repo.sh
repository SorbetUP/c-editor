#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Cleaning generated artifacts from ${ROOT_DIR}"

find "${ROOT_DIR}" \
    \( -name '.DS_Store' -o -name '*.pyc' -o -name '*.pyo' \) \
    -type f -delete

find "${ROOT_DIR}" \
    \( -name '__pycache__' -o -name '.pytest_cache' \) \
    -type d -prune -exec rm -rf {} +

find "${ROOT_DIR}/engines" \
    \( -name '*.o' -o -name '*.a' -o -name '*.wasm' -o -name '*.js.mem' \) \
    -type f -delete

find "${ROOT_DIR}/engines" \
    \( -name 'test_*' -o -name 'debug_*' -o -name 'markdown_test' -o -name 'test_render' -o -name 'test_search' \) \
    -type f -delete

rm -rf \
    "${ROOT_DIR}/dist" \
    "${ROOT_DIR}/output" \
    "${ROOT_DIR}/Logs" \
    "${ROOT_DIR}/test-results"

echo "Repository cleanup complete"
