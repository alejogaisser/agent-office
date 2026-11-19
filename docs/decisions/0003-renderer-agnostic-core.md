# 0003. Renderer-agnostic core crate and a JSON Snapshot/Delta contract
- Date: 2026-11-18
- Status: accepted (user-approved 2026-11-18)
- Decision: Discovery, tail reading, parsing, activity classification, roles and colors, and session lifecycle all live in the `office-core` crate. It has no GUI or watcher dependencies, takes an injected clock and process probe, and exposes serde `Snapshot`/`Delta` values. `office-dump --follow` streams the same contract as NDJSON.
- Reason: The parser stays unit-testable against fixtures without a GUI. Another renderer (Bevy, Godot, Unity) could replace the WebView by consuming the same contract, so the stack choice in 0001 can be reversed.
