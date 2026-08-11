# Code composition report

This is a baseline/phase report, not a completion claim.

| Layer | Before migration | Current phase |
|---|---:|---:|
| Rust | existing Tauri backend and `muya-core` | Freya shell crate added; domain reuse beyond visibility is pending |
| Vue | 109 audited SFC files | unchanged; retained as oracle |
| JavaScript | 434 audited JS files | unchanged; no domain logic was replaced by a fake bridge |
| CSS | 106 external CSS files plus component styles | unchanged; token mapping starts only in the Freya shell |
| Web islands | Excalidraw is still embedded in the global renderer | isolation contract documented; extraction pending |

The current phase must not be counted as “Vue removed”, “JavaScript translated”, or “WebView eliminated”.
