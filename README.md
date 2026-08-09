<p align="center">
  <img src="Elephant/assets/static/icon.png" alt="ElephantNote icon" width="96" height="96">
</p>

<h1 align="center">ElephantNote</h1>

<p align="center">
  <strong>Local-first Markdown notes, knowledge search, graph navigation, sync, and optional AI assistance.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-PolyForm%20Noncommercial%201.0.0-orange" alt="License: PolyForm Noncommercial 1.0.0"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows%20%7C%20Android-lightgrey" alt="Platforms">
  <img src="https://img.shields.io/badge/runtime-Tauri-blue" alt="Runtime: Tauri">
</p>

## Overview

ElephantNote is a local-first Markdown notes application. It is designed for personal knowledge bases that need fast editing, file-based storage, search, graph navigation, sync, and optional local AI workflows.

The project is under active development. Tauri is the desktop and mobile runtime.

## Goals

- Keep notes as normal Markdown files.
- Make large vaults searchable even when they are not perfectly organized.
- Support exact search, semantic search, graph navigation, and wiki-style views.
- Prefer local execution and local sync for personal workflows.
- Keep external services optional.
- Prepare a clean addon system inspired by Obsidian.

## Features

- Markdown editor and local vaults.
- Folder navigation, note list previews, and wiki/root views.
- Image, attachment, and Excalidraw-style asset handling.
- Exact text search over Markdown files.
- Semantic search with knowledge chunks and metadata.
- Smart search with lexical and semantic routing.
- Graph data with note, folder, tag, and semantic edges.
- Optional local model support for chat, embeddings, OCR, and retrieval.
- Tauri desktop runtime for macOS, Linux, and Windows.
- Tauri Android development scripts.
- Docker/web mode for local experiments.

## Requirements

- Node.js >= 20.19.0
- pnpm >= 10
- Rust toolchain for Tauri builds
- Platform dependencies required by Tauri
- Optional: Docker for web and sync smoke tests

## Quick start

```bash
git clone https://github.com/SorbetUP/ElephantNote.git
cd ElephantNote
pnpm install
```

Run Tauri:

```bash
pnpm tauri:dev
```

## Build

```bash
pnpm tauri:build
pnpm build
pnpm build:mac
pnpm build:linux
pnpm build:win
pnpm tauri:linux:build
```

Android:

```bash
pnpm tauri:android:init
pnpm tauri:android:dev
pnpm tauri:android:build
```

Docker/web:

```bash
pnpm web:docker:build
pnpm web:docker:run
```

## Tests and checks

```bash
pnpm test:unit
pnpm test
pnpm build/coverage
pnpm test:e2e
pnpm security:guard
pnpm tauri:check
pnpm tauri:platform:check
pnpm tauri:mac:smoke
pnpm test:sync:docker
pnpm test:sync:docker:pair
pnpm prod:check
```

## Status

This branch moves quickly. The backend is Rust-only (Tauri); the Electron main process has been retired and `Elephant/backend/js/` now exists only as a compatibility layer for parity tests. Tauri, sync, addon support, local AI, graph features, and search features are being integrated and tested progressively. Do not assume every experimental feature is production-ready.

## Project structure

```text
.
├── Elephant/           # App source: frontend, backend, shared code, and static assets
│   ├── frontend/
│   │   ├── src/        # Current Vue/Muya renderer source
│   │   └── app/        # Elephant UI modules still imported by the app/tests
│   ├── backend/
│   │   ├── tauri/      # Rust/Tauri backend
│   │   └── js/  # Retired JS backend modules kept for parity tests
│   ├── shared/
│   └── assets/static/
├── build/              # Build scripts, platform prototypes, and generated local outputs
│   ├── android/
│   ├── ios/
│   └── scripts/
├── tests/              # Unit, e2e, fixtures, and compatibility parity tests
└── agent/              # Agent playbooks, docs, parity notes, feature intake, patches, security data
├── package.json
└── README.md
```

Generated outputs such as `build/out/`, `build/coverage/`, `build/test-results/`,
`build/ios/.build/`, `build/android/**/build/`, and `Elephant/backend/tauri/target/`
are local build/test artifacts and should not be committed.

## License

ElephantNote project-specific code is distributed under the PolyForm Noncommercial License 1.0.0.

Practical summary:

- personal use is allowed;
- non-commercial study, research, hobby, and experimentation are allowed;
- non-commercial modifications are allowed;
- commercial use, enterprise deployment, resale, SaaS usage, or integration into a paid product or service requires separate written permission.

This is a source-available, non-commercial license. It is not an OSI-approved open-source license.

Parts of this repository are derived from or include third-party work, including MarkText and Muya-era code. Those parts remain subject to their own notices and license terms. See LICENSE and generated third-party license material when applicable.

## Attribution

ElephantNote builds on prior Markdown editor work and is moving toward a separate local-first knowledge application with Tauri, sync, AI, graph, wiki, and addon features.
