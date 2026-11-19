# 0002. Data source: local transcripts, with the session registry as an optional hint, and no hooks
- Date: 2026-11-18
- Status: accepted (user-approved 2026-11-18)
- Decision: Derive all agent state from `~/.claude/projects/**.jsonl` and the subagent `.meta.json` files, using incremental tail reads. Use `~/.claude/sessions/<pid>.json` (undocumented) only to detect open sessions and waiting state, behind `useSessionRegistry`. Never open `*.key` files. Do not use hooks.
- Reason: This costs zero tokens and needs no changes to the user's Claude settings. Hooks would spawn a process per tool call inside the user's sessions. The registry is undocumented and has a known stuck-`busy` bug, so the app falls back to an mtime window (default 30 min).
