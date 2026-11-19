# 0004. RAM budget raised to 250 MB total private
- Date: 2026-11-18
- Status: accepted (user decision 2026-11-18)
- Decision: The idle RAM criterion is `privateMB <= 250` for the whole process tree (host plus WebView2), and the host budget stays at 40 MB or less. This replaces the 200 MB limit in v1-AC16. The stack stays Tauri 2 + Three.js (0001).
- Reason: v1 measured 203-220 MB, and nearly all of it is WebView2 (the Rust host is about 6 MB). The v2 visual revision needs headroom, and the user accepted this cost instead of switching renderers.

## Amendment (2026-11-18)
After v2 (building, team floors and people), the measured total was 249-270 MB depending on window size at 150% DPI, almost all of it the WebView2 GPU process; the Rust host stays around 6 MB. The user raised the total private budget to 500 MB ("or whatever is needed") and said not to spend more effort trimming it. The host budget stays at 40 MB.
