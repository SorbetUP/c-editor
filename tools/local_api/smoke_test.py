#!/usr/bin/env python3
"""Small smoke test for the local API server."""

from __future__ import annotations

import json
import tempfile
import threading
import time
import urllib.request
from pathlib import Path

from server import create_server, default_repo_root


def request_json(url: str, method: str = "GET", payload: dict | None = None) -> dict:
    data = None
    headers = {}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=data, method=method, headers=headers)
    with urllib.request.urlopen(req, timeout=5) as response:
        return json.loads(response.read().decode("utf-8"))


def main() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        config_path = tmp_path / "local_api.json"
        server = create_server("127.0.0.1", 8765, default_repo_root(), config_path)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        time.sleep(0.2)

        health = request_json("http://127.0.0.1:8765/api/health")
        assert health["ok"] is True

        vault_path = str(tmp_path / "Vault")
        vault = request_json(
            "http://127.0.0.1:8765/api/vault", "PUT", {"path": vault_path}
        )
        assert vault["configured"] is True

        note = request_json(
            "http://127.0.0.1:8765/api/note",
            "PUT",
            {"title": "Smoke", "content": "# Smoke\n\nLien vers [[Bienvenue]]"},
        )
        assert note["title"] == "Smoke"

        notes = request_json("http://127.0.0.1:8765/api/notes")
        assert len(notes["notes"]) == 1
        assert notes["notes"][0]["links"] == ["Bienvenue"]
        server.shutdown()
        server.server_close()
    print("local api smoke test passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
