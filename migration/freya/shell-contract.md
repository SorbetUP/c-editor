# Shell parity contract

The current Tauri shell is the visual oracle. The first native shell deliberately uses the measured baseline geometry captured by the audit:

| Contract | Tauri/Vue baseline | Freya slice | Proof state |
|---|---:|---:|---|
| desktop window | 1280 × 840 config | Freya default window | not pixel-compared |
| navigation rail | approximately 48–72 px depending on active shell path | 72 px | screenshot comparison pending |
| top bar | approximately 28–32 px | represented by shell header | screenshot comparison pending |
| sidebar | approximately 232 px, bounded 184–320 px | 246 px when open | screenshot comparison pending |
| tree row | approximately 36 px | 36 px row | screenshot comparison pending |

Stable selectors, labels, keyboard behavior, mobile breakpoints, empty/error/loading states and persistence are part of the contract. The native shell currently proves only a subset: dark shell layout, visible vault entries, navigation state and visible loader errors. No Vue/Freya pixel parity claim is made.
