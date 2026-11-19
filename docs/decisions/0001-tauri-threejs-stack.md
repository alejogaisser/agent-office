# 0001. Desktop stack: Tauri 2 + Three.js with a Rust core
- Date: 2026-11-18
- Status: accepted (user-approved 2026-11-18)
- Decision: Build Agent Office as a Tauri 2 app, with a Rust host and core plus a Vite/TypeScript/Three.js frontend. Use no UI framework and no Tauri plugins.
- Reason: Its installer is the smallest (3-10 MB, system WebView2) and its idle CPU is near zero with on-demand rendering. HTML/CSS handles the sidebar and tooltips easily, and dev effort is moderate. RAM is moderate (80-150 MB, the WebView2 floor). Electron is heavier for no gain. Unity's always-on player loop and 60+ MB builds are overkill. Godot is not installed. Bevy or egui would save RAM but cost much more UI effort. They are the fallback if the user requires less than about 100 MB.
