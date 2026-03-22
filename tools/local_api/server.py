#!/usr/bin/env python3
"""ElephantNote local-first HTTP server.

Serves the web UI and exposes a small JSON API around a vault folder.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from datetime import datetime, timezone
from http import HTTPStatus
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urlparse, unquote


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def default_config_dir() -> Path:
    home = Path.home()
    if sys.platform == "darwin":
        return home / "Library" / "Application Support" / "ElephantNote"
    return home / ".config" / "elephantnote"


class LocalConfig:
    def __init__(self, path: Path):
        self.path = path
        self.data = {
            "vault_path": "",
            "serve_port": 8421,
            "sync_mode": "local",
            "peers": [],
        }
        self.load()

    def load(self) -> None:
        if not self.path.exists():
            return
        with self.path.open("r", encoding="utf-8") as handle:
            loaded = json.load(handle)
        if isinstance(loaded, dict):
            self.data.update(loaded)

    def save(self) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        with self.path.open("w", encoding="utf-8") as handle:
            json.dump(self.data, handle, indent=2, ensure_ascii=False)

    @property
    def vault_path(self) -> str:
        return str(self.data.get("vault_path", "") or "")

    @vault_path.setter
    def vault_path(self, value: str) -> None:
        self.data["vault_path"] = value

    def to_dict(self) -> dict[str, Any]:
        return dict(self.data)


class VaultStore:
    NOTE_EXTENSIONS = {".md", ".markdown"}

    def __init__(self, config: LocalConfig):
        self.config = config

    @property
    def root(self) -> Path | None:
        path = self.config.vault_path.strip()
        if not path:
            return None
        return Path(path).expanduser().resolve()

    @property
    def notes_dir(self) -> Path | None:
        root = self.root
        if root is None:
            return None
        notes_dir = root / "Notes"
        return notes_dir if notes_dir.exists() else root

    def ensure_vault(self, selected_path: str) -> dict[str, Any]:
        root = Path(selected_path).expanduser().resolve()
        root.mkdir(parents=True, exist_ok=True)
        (root / "Notes").mkdir(exist_ok=True)
        (root / "Assets").mkdir(exist_ok=True)
        marker_dir = root / ".elephantnote"
        marker_dir.mkdir(exist_ok=True)
        marker = marker_dir / "vault.json"
        if not marker.exists():
            marker.write_text(
                json.dumps(
                    {
                        "created_at": datetime.now(timezone.utc).isoformat(),
                        "version": 1,
                    },
                    indent=2,
                ),
                encoding="utf-8",
            )
        self.config.vault_path = str(root)
        self.config.save()
        return self.describe_vault()

    def describe_vault(self) -> dict[str, Any]:
        root = self.root
        if root is None:
            return {"configured": False, "path": "", "note_count": 0}
        notes = self.list_notes()
        return {
            "configured": True,
            "path": str(root),
            "note_count": len(notes),
            "notes_dir": str(self.notes_dir or root),
        }

    def list_notes(self) -> list[dict[str, Any]]:
        notes_dir = self.notes_dir
        if notes_dir is None or not notes_dir.exists():
            return []

        notes: list[dict[str, Any]] = []
        for path in sorted(notes_dir.rglob("*")):
            if not path.is_file() or path.suffix.lower() not in self.NOTE_EXTENSIONS:
                continue
            content = path.read_text(encoding="utf-8")
            notes.append(self._serialize_note(path, content))
        notes.sort(key=lambda item: item["updatedAt"], reverse=True)
        return notes

    def read_note(self, relative_path: str) -> dict[str, Any]:
        path = self._resolve_note_path(relative_path)
        content = path.read_text(encoding="utf-8")
        return self._serialize_note(path, content)

    def write_note(
        self, relative_path: str | None, title: str | None, content: str
    ) -> dict[str, Any]:
        root_notes = self.notes_dir
        if root_notes is None:
            raise ValueError("No vault configured")

        rel = (relative_path or "").strip()
        if rel:
            path = self._resolve_note_path(rel)
        else:
            filename = slugify(title or extract_title(content) or "nouvelle-note") + ".md"
            path = root_notes / filename

        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return self._serialize_note(path, content)

    def metadata_for_content(self, content: str) -> dict[str, Any]:
        return {
            "sensitive": is_sensitive_markdown(content),
            "links": extract_links(content),
            "title": extract_title(content),
            "tags": extract_tags(content),
        }

    def _resolve_note_path(self, relative_path: str) -> Path:
        notes_dir = self.notes_dir
        if notes_dir is None:
            raise ValueError("No vault configured")
        raw = unquote(relative_path).lstrip("/")
        candidate = (notes_dir / raw).resolve()
        if notes_dir.resolve() not in candidate.parents and candidate != notes_dir.resolve():
            raise ValueError("Invalid note path")
        if candidate.suffix.lower() not in self.NOTE_EXTENSIONS:
            candidate = candidate.with_suffix(".md")
        return candidate

    def _serialize_note(self, path: Path, content: str) -> dict[str, Any]:
        stats = path.stat()
        notes_dir = self.notes_dir or path.parent
        relative_path = str(path.relative_to(notes_dir)).replace(os.sep, "/")
        metadata = self.metadata_for_content(content)
        return {
            "id": slugify(relative_path),
            "path": relative_path,
            "title": metadata["title"] or path.stem,
            "content": content,
            "updatedAt": datetime.fromtimestamp(
                stats.st_mtime, tz=timezone.utc
            ).isoformat(),
            "tags": metadata["tags"],
            "links": metadata["links"],
            "sensitive": metadata["sensitive"],
            "excerpt": mask_sensitive_preview(content),
        }


def slugify(value: str) -> str:
    lowered = re.sub(r"[^a-zA-Z0-9]+", "-", value.strip().lower()).strip("-")
    return lowered or "note"


def extract_title(content: str) -> str:
    for line in content.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("#"):
            return stripped.lstrip("#").strip()
        return stripped[:80]
    return ""


def extract_tags(content: str) -> list[str]:
    found: list[str] = []
    seen: set[str] = set()
    for match in re.finditer(r"(?<!\w)#([a-zA-Z0-9_-]+)", content):
        tag = match.group(1).lower()
        if tag not in seen:
            seen.add(tag)
            found.append(tag)
    return found


def extract_links(content: str) -> list[str]:
    found: list[str] = []
    seen: set[str] = set()
    for pattern in (r"\[\[([^\]]+)\]\]", r"\[([^\]]+)\]\(([^)]+)\)"):
        for match in re.finditer(pattern, content):
            value = match.group(1).strip()
            if value and value.lower() not in seen:
                seen.add(value.lower())
                found.append(value)
    return found


def is_sensitive_markdown(content: str) -> bool:
    return bool(re.search(r"(?<!\w)#credentials\b", content, flags=re.IGNORECASE))


def mask_sensitive_preview(content: str) -> str:
    lines = content.splitlines()[:4]
    preview = " ".join(line.strip() for line in lines if line.strip())
    if is_sensitive_markdown(content):
        return "Contenu sensible masqué"
    preview = re.sub(r"\s+", " ", preview)
    return preview[:160]


class ElephantHandler(SimpleHTTPRequestHandler):
    server_version = "ElephantNoteLocalAPI/0.1"

    def __init__(self, *args: Any, directory: str | None = None, **kwargs: Any):
        super().__init__(*args, directory=directory, **kwargs)

    @property
    def app(self) -> "ElephantServer":
        return self.server  # type: ignore[return-value]

    def log_message(self, format: str, *args: Any) -> None:
        return

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path.startswith("/api/"):
            self.handle_api_get(parsed)
            return
        super().do_GET()

    def do_PUT(self) -> None:
        parsed = urlparse(self.path)
        if not parsed.path.startswith("/api/"):
            self.send_error(HTTPStatus.METHOD_NOT_ALLOWED)
            return
        payload = self.read_json_body()
        self.handle_api_put(parsed, payload)

    def do_OPTIONS(self) -> None:
        self.send_response(HTTPStatus.NO_CONTENT)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, PUT, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def end_headers(self) -> None:
        self.send_header("Access-Control-Allow-Origin", "*")
        super().end_headers()

    def handle_api_get(self, parsed: Any) -> None:
        query = parse_qs(parsed.query)
        try:
            if parsed.path == "/api/health":
                self.send_json(
                    {
                        "ok": True,
                        "vault": self.app.vault_store.describe_vault(),
                        "config": self.app.config.to_dict(),
                    }
                )
                return
            if parsed.path == "/api/config":
                self.send_json(self.app.config.to_dict())
                return
            if parsed.path == "/api/vault":
                self.send_json(self.app.vault_store.describe_vault())
                return
            if parsed.path == "/api/notes":
                self.send_json({"notes": self.app.vault_store.list_notes()})
                return
            if parsed.path == "/api/note":
                rel = first_query_value(query, "path")
                if not rel:
                    self.send_json({"error": "Missing path"}, status=HTTPStatus.BAD_REQUEST)
                    return
                self.send_json(self.app.vault_store.read_note(rel))
                return
            if parsed.path == "/api/metadata":
                text = first_query_value(query, "text") or ""
                self.send_json(self.app.vault_store.metadata_for_content(text))
                return
            self.send_json({"error": "Not found"}, status=HTTPStatus.NOT_FOUND)
        except FileNotFoundError:
            self.send_json({"error": "Note not found"}, status=HTTPStatus.NOT_FOUND)
        except ValueError as exc:
            self.send_json({"error": str(exc)}, status=HTTPStatus.BAD_REQUEST)

    def handle_api_put(self, parsed: Any, payload: dict[str, Any]) -> None:
        try:
            if parsed.path == "/api/config":
                self.app.config.data.update(payload)
                self.app.config.save()
                self.send_json(self.app.config.to_dict())
                return
            if parsed.path == "/api/vault":
                path = str(payload.get("path", "")).strip()
                if not path:
                    self.send_json({"error": "Missing path"}, status=HTTPStatus.BAD_REQUEST)
                    return
                self.send_json(self.app.vault_store.ensure_vault(path))
                return
            if parsed.path == "/api/note":
                content = str(payload.get("content", ""))
                note = self.app.vault_store.write_note(
                    payload.get("path"),
                    payload.get("title"),
                    content,
                )
                self.send_json(note)
                return
            self.send_json({"error": "Not found"}, status=HTTPStatus.NOT_FOUND)
        except ValueError as exc:
            self.send_json({"error": str(exc)}, status=HTTPStatus.BAD_REQUEST)

    def read_json_body(self) -> dict[str, Any]:
        try:
            length = int(self.headers.get("Content-Length", "0"))
        except ValueError:
            length = 0
        raw = self.rfile.read(length) if length > 0 else b"{}"
        if not raw:
            return {}
        return json.loads(raw.decode("utf-8"))

    def send_json(self, payload: Any, status: HTTPStatus = HTTPStatus.OK) -> None:
        data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def first_query_value(query: dict[str, list[str]], key: str) -> str:
    values = query.get(key) or [""]
    return values[0]


class ElephantServer(ThreadingHTTPServer):
    def __init__(
        self,
        server_address: tuple[str, int],
        handler_factory: type[ElephantHandler],
        repo_root: Path,
        config: LocalConfig,
    ):
        self.repo_root = repo_root
        self.config = config
        self.vault_store = VaultStore(config)
        self.site_root = repo_root / "web" / "site"
        super().__init__(server_address, self._wrap_handler(handler_factory))

    def _wrap_handler(self, handler_factory: type[ElephantHandler]):
        site_root = str(self.site_root)

        def factory(*args: Any, **kwargs: Any) -> ElephantHandler:
            return handler_factory(*args, directory=site_root, **kwargs)

        return factory


def create_server(
    host: str,
    port: int,
    repo_root: Path,
    config_path: Path | None = None,
) -> ElephantServer:
    config = LocalConfig(config_path or (default_config_dir() / "local_api.json"))
    return ElephantServer((host, port), ElephantHandler, repo_root, config)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="ElephantNote local API server")
    parser.add_argument("--host", default="127.0.0.1", help="Bind host")
    parser.add_argument("--port", type=int, default=8421, help="Bind port")
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=default_repo_root(),
        help="Repository root used to serve web/site",
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=default_config_dir() / "local_api.json",
        help="Path to config JSON",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    server = create_server(args.host, args.port, args.repo_root, args.config)
    print(f"ElephantNote local API listening on http://{args.host}:{args.port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
