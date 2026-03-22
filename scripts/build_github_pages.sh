#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
EDITOR_DIR="$ROOT_DIR/engines/editor"
CURSOR_DIR="$ROOT_DIR/engines/cursor"
SITE_DIR="$ROOT_DIR/web/site"
DOCS_DIR="$SITE_DIR/docs"
DIST_DIR="$ROOT_DIR/dist/github-pages"
DIST_DOCS_DIR="$DIST_DIR/docs"

echo "==> Building GitHub Pages artifact"
echo "Root: $ROOT_DIR"

make -C "$EDITOR_DIR" clean wasm
make -C "$CURSOR_DIR" clean wasm

mkdir -p "$DOCS_DIR"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DOCS_DIR"

cp -f "$EDITOR_DIR/editor.js" "$DOCS_DIR/editor.js"
cp -f "$EDITOR_DIR/editor.wasm" "$DOCS_DIR/editor.wasm"
cp -f "$CURSOR_DIR/cursor_wasm.js" "$DOCS_DIR/cursor_wasm.js"
cp -f "$CURSOR_DIR/cursor_wasm.wasm" "$DOCS_DIR/cursor_wasm.wasm"
cp -f "$ROOT_DIR/web/wasm/cursor_c_interface.js" "$DOCS_DIR/cursor_c_interface.js"

cp -f "$SITE_DIR/index.html" "$DIST_DIR/index.html"
cp -f "$DOCS_DIR/index.html" "$DIST_DOCS_DIR/index.html"
cp -f "$DOCS_DIR/editor.js" "$DIST_DOCS_DIR/editor.js"
cp -f "$DOCS_DIR/editor.wasm" "$DIST_DOCS_DIR/editor.wasm"
cp -f "$DOCS_DIR/cursor_wasm.js" "$DIST_DOCS_DIR/cursor_wasm.js"
cp -f "$DOCS_DIR/cursor_wasm.wasm" "$DIST_DOCS_DIR/cursor_wasm.wasm"
cp -f "$DOCS_DIR/cursor_c_interface.js" "$DIST_DOCS_DIR/cursor_c_interface.js"

touch "$DIST_DIR/.nojekyll"

test -f "$DIST_DIR/index.html"
test -f "$DIST_DOCS_DIR/index.html"
test -f "$DIST_DOCS_DIR/editor.js"
test -f "$DIST_DOCS_DIR/editor.wasm"
test -f "$DIST_DOCS_DIR/cursor_wasm.js"
test -f "$DIST_DOCS_DIR/cursor_wasm.wasm"
test -f "$DIST_DOCS_DIR/cursor_c_interface.js"
test -f "$DIST_DIR/.nojekyll"

echo "==> GitHub Pages artifact ready"
ls -lh \
  "$DIST_DOCS_DIR/editor.js" \
  "$DIST_DOCS_DIR/editor.wasm" \
  "$DIST_DOCS_DIR/cursor_wasm.js" \
  "$DIST_DOCS_DIR/cursor_wasm.wasm"
