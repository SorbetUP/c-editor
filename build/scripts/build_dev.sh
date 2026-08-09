#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

kill_port_1420() {
  local pids
  if ! command -v lsof >/dev/null 2>&1; then
    return 0
  fi
  pids="$(lsof -ti tcp:1420 2>/dev/null || true)"

  if [ -n "$pids" ]; then
    kill $pids 2>/dev/null || true
  fi
}

pkill -f 'target/debug/Elephant' 2>/dev/null || true
pkill -f 'target/debug/elephantnote-tauri' 2>/dev/null || true
pkill -f 'vite --config vite.tauri.config.js' 2>/dev/null || true
kill_port_1420
rm -rf "$ROOT_DIR/Elephant/backend/tauri/target/debug/bundle/macos/Elephant.app" 2>/dev/null || true

cd "$ROOT_DIR/Elephant/backend/tauri"
export TAURI_FRONTEND_PATH="$ROOT_DIR"

# Optional native runtimes are installed and launched from physical addon
# packages. The core development shell must not download or bundle them.
if [ "$(uname -s 2>/dev/null || echo unknown)" = "Linux" ]; then
  cargo tauri dev "$@" --no-watch --config tauri.linux.conf.json
else
  cargo tauri dev "$@" --no-watch
fi
