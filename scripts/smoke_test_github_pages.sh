#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/github-pages"
PORT="${PORT:-8011}"
SERVER_PID=""

cleanup() {
  if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}

trap cleanup EXIT

if [ ! -d "$DIST_DIR" ]; then
  echo "Missing Pages artifact: $DIST_DIR"
  echo "Run ./scripts/build_github_pages.sh first."
  exit 1
fi

echo "==> Smoke testing GitHub Pages artifact"
python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$DIST_DIR" >/tmp/elephantnote-pages-smoke.log 2>&1 &
SERVER_PID=$!
sleep 2

curl -fsS "http://127.0.0.1:$PORT/" >/dev/null
curl -fsS "http://127.0.0.1:$PORT/docs/index.html" >/dev/null
curl -fsS "http://127.0.0.1:$PORT/docs/editor.js" >/dev/null
curl -fsS "http://127.0.0.1:$PORT/docs/editor.wasm" >/dev/null
curl -fsS "http://127.0.0.1:$PORT/docs/cursor_wasm.js" >/dev/null
curl -fsS "http://127.0.0.1:$PORT/docs/cursor_wasm.wasm" >/dev/null
curl -fsS "http://127.0.0.1:$PORT/docs/cursor_c_interface.js" >/dev/null

echo "==> GitHub Pages smoke test passed"
