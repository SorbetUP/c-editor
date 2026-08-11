# Performance baseline (partial)

This is a measured phase artifact, not a claim that Freya is faster.

| Runtime | Scenario | Measurement | Evidence |
|---|---|---:|---|
| Freya | macOS, debug binary already built, fixture vault, idle after 35 s | RSS `96 144 KiB` | direct `ps -axo pid=,rss=,etime=,command=` snapshot at 2026-08-11 |
| Freya | cold build + launch observable smoke | `22.94 s` wall time, including Cargo build | `2026-08-11T13-18-04-183Z-freya-dev-60456.log` / `.ndjson` |
| Tauri/Vue | real launch | started and rendered before controlled stop | `2026-08-11T12-59-46-807Z-tauri-dev-51175.log` |

Cold/warm startup, RSS, CPU, keystroke latency, large-library stress and equivalent Tauri/Freya measurements are still missing. They must be collected with the same profile, vault, window size and build mode before a default-runtime switch.
