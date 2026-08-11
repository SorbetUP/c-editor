# Freya capability matrix

This matrix records what has been exercised locally, not what merely exists in a dependency.

| Capability | Local evidence | State |
| --- | --- | --- |
| Native rect/label layout | `cargo check`, `freya-testing` render | PROVEN |
| State/hooks | `use_state` in shell and headless render | PROVEN |
| Semantic accessibility labels | `TestingRunner.find` in shell acceptance | PROVEN |
| Pointer event dispatch | `TestingRunner.click_cursor` on semantic node | PROVEN |
| Keyboard/text editing primitives | not wired into converted editor surface | NOT PROVEN |
| Virtual scrolling | typed 120/72/720 contract only | NOT PROVEN |
| Rich text/Markdown editing | muya-core adapter only; no Freya input surface | NOT PROVEN |
| Images/SVG | no native card image renderer yet | NOT PROVEN |
| Isolated WebView/Excalidraw | no Freya island target | NOT PROVEN |
| Desktop packaged Freya app | no package target | NOT PROVEN |
| Android Freya app | no target/build/run | NOT PROVEN |

The current implementation intentionally refuses to pretend that drawing creation or rich editing is native until the existing Excalidraw and Muya behavior have been converted through real production paths.
