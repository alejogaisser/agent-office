<!-- software-team plan -->
# Agent Office: implementation plan (v1)

## Handoff

### Goal
Build a Windows 11 desktop app that shows every open Claude Code session as a floor of a low-poly 3D office, seen from an isometric camera. Each floor shows the main agent and its subagents at desks, with live activity states. The app only reads local files, never calls a model, and treats RAM and CPU use as top requirements.

### Context
- The repo `C:\dev\agent-office` is empty. Stack (see `docs/decisions/0001-tauri-threejs-stack.md`): a Cargo workspace with a pure Rust core crate `office-core` (discovery, tail reader, parser, state model), a Tauri 2 host `src-tauri` (file watcher, IPC), and a Vite + TypeScript + Three.js frontend `ui/` with no UI framework.
- Data source (verified on this machine 2026-11-18, `docs/decisions/0002-transcripts-only-data-source.md`):
  - Transcripts: `<claudeDir>/projects/<slug>/<sessionId>.jsonl`. The slug is cwd with every char not in `[A-Za-z0-9]` replaced by `-` (`C:\Users\ASUS\OneDrive\Desktop\Content creator` -> `C--Users-ASUS-OneDrive-Desktop-Content-creator`).
  - Subagents: `<slug>/<sessionId>/subagents/agent-<agentId>.jsonl` plus `.meta.json` with keys `agentType`, `description`, `toolUseId`, `spawnDepth`. Ignore other subfolders such as `tool-results/`.
  - Parent spawn: an assistant `tool_use` with `name:"Agent"` and `input.subagent_type`, `input.description`, `input.run_in_background`. Its `id` equals the meta `toolUseId`. For foreground agents, the parent later receives a `tool_result` with that `tool_use_id`.
  - One assistant API message is split over several lines with the same `message.id`, and each line repeats `message.usage`. Count tokens once per `message.id`.
  - `message.stop_reason` is `"tool_use"`, `"end_turn"`, or null. A turn ends with `end_turn` and then a `system` line with `subtype:"stop_hook_summary"`. `system` with `subtype:"api_error"` also occurs.
  - `agentType` has both legacy (`architect`, `coder`, `tester`) and prefixed (`gaisser-agents:software-architect`) forms.
  - Undocumented session registry: `<claudeDir>/sessions/<pid>.json` with `pid`, `sessionId`, `cwd`, `name`, `status` (`busy`/`idle`/`waiting`), `startedAt`, `updatedAt`, `procStart` (a Windows FILETIME). The same folder holds `<pid>.<hash>.key` files, which are secrets and must never be opened.
- Tool names seen: Read, Write, Edit, Bash, Grep, Glob, WebFetch, WebSearch, Agent, SendMessage, ListAgents, ReadNotifications, ToolSearch, TodoWrite, SubagentHandback, `mcp__*`.

### Constraints
- Zero LLM tokens and no network: no HTTP client crates, no `fetch`/`WebSocket` in the UI, strict CSP, and no Tauri plugins.
- Read-only access to `<claudeDir>`. Never open files that match `*.key`. Never write anywhere except `%APPDATA%\agent-office\` (config) and `%LOCALAPPDATA%\agent-office\` (optional log).
- Render on demand only. No continuous `setAnimationLoop`. Render nothing while the document is hidden.
- File reads are incremental: offset-based tail reads, 64 KiB chunks. Skip lines longer than 16 MiB. The first read of a file covers at most its last `maxInitialReadBytes`.
- Per-agent memory is bounded: `recent` ring of at most 20 entries, pending tool map of at most 256 entries, strings truncated to 200 chars.
- `office-core` must not depend on Tauri, notify, or any GUI code, and must be testable with `cargo test` only. All time-dependent functions take `now_ms: i64`.
- Fixtures are synthetic. Never copy real transcript content into the repo.
- Frontend: vanilla TypeScript and `three` only (no React/Vue). Cap the pixel ratio at 1.5. No shadows, antialias off, flat shading, shared geometries and materials.
- Windows 11 is the target. Cross-platform code paths may compile but are not tested.

### Files to touch
- `Cargo.toml` (workspace: members `crates/office-core`, `src-tauri`; release profile `opt-level="s"`, `lto=true`, `codegen-units=1`, `strip=true`)
- `.gitignore`
- `README.md`: what the app is, how to run it, config keys, and a `## Data access and privacy` section (see AC23)
- `crates/office-core/Cargo.toml` (deps: serde, serde_json, `windows-sys` under cfg(windows); dev-deps: tempfile, filetime)
- `crates/office-core/src/{lib.rs,model.rs,line.rs,reducer.rs,tail.rs,discovery.rs,office.rs,roles.rs,config.rs,probe.rs}`
- `crates/office-core/src/bin/office-dump.rs`
- `crates/office-core/tests/{parser.rs,tail.rs,discovery.rs,office.rs,roles.rs}`, plus `crates/office-core/tests/fixtures/**` (synthetic)
- `src-tauri/{Cargo.toml,build.rs,tauri.conf.json,capabilities/default.json,icons/*}`
- `src-tauri/src/{main.rs,lib.rs,worker.rs,watcher.rs,commands.rs,logging.rs}`
- `ui/{package.json,tsconfig.json,vite.config.ts,index.html}`
- `ui/src/{main.ts,ipc.ts,store.ts,types.ts,sidebar.ts,tooltip.ts,debug.ts,styles.css}`
- `ui/src/scene/{renderer.ts,office.ts,desk.ts,avatar.ts,tweens.ts,palette.ts}`
- `scripts/simulate.mjs`, `scripts/measure-resources.ps1`

### Out of scope
Hooks integration, system tray, auto-update, installers for macOS/Linux, a settings UI (v1 uses a config file), camera rotation, sound, history replay, editing or controlling sessions, and a Bevy/Unity renderer. See "Nice-to-haves".

### Acceptance criteria
Unless stated otherwise, commands run from `C:\dev\agent-office`. "Measure" means `pwsh scripts/measure-resources.ps1 -Name agent-office -Seconds 60`, which prints JSON `{cpuPercent, privateMB, hostPrivateMB, processCount}`. The script sums the host process and all descendants (WebView2) found through `Win32_Process.ParentProcessId`. CPU% is total processor-time delta / (wall seconds x logical cores) x 100.
- AC1: `npm ci --prefix ui` and `npx --prefix ui tauri build` exit 0. The NSIS installer in `target/release/bundle/nsis/` is at most 15 MB.
- AC2: `cargo test -p office-core` exits 0 with 0 failures and includes every test named in AC3 to AC11 and AC20 to AC22.
- AC3 `tail_partial_line`: two lines are appended to a temp file in 3 writes, splitting line 2 mid-JSON. After each write `TailReader::read_new` returns 1, 0, then 1 complete lines, and returns no error.
- AC4 `tolerates_unknown_and_malformed`: `fixtures/unknown_and_malformed.jsonl` holds 6 lines: 1 unknown type, 1 invalid JSON, 1 missing `message`, and 3 valid lines. Parsing yields events for the 3 valid lines, `stats.malformed == 1`, and no panic.
- AC5 `activity_transitions`: after applying `fixtures/main_basic.jsonl` line by line, the main agent's activity is `writing` with target `src/app.ts` after the Edit tool_use, `thinking` after its tool_result, `running` with target starting `npm test` after the Bash tool_use, and `waiting` after `end_turn` plus `stop_hook_summary`.
- AC6 `tokens_dedup_by_message_id`: in `fixtures/tokens_dup.jsonl`, 3 lines share one `message.id` with `output_tokens:7`, plus 1 line with a different id and `output_tokens:5`. The result is `tokens.output == 12`, and `tokens.context` equals the last message's input + cache_read + cache_creation.
- AC7 `subagent_spawn_and_done`: in `fixtures/root/`, a session spawns foreground agent `a1` (meta `agentType:"gaisser-agents:software-test-engineer"`). The snapshot has agent `a1` with `role.label=="Tester"`, `parentId=="main"`, and the meta description. One `agentSpawned` delta fires. After the parent tool_result for its `toolUseId` is appended, `a1.activity=="done"`. Background agent `a2` (`run_in_background:true`) stays non-done after its tool_result and becomes `done` only after its own `SubagentHandback` tool_use.
- AC8 `role_mapping`: each pair in the table of Step 6 maps as listed (including the legacy `coder` -> `Coder`). `"gaisser-agents:foo-bar"` -> label `"Foo Bar"`, with a palette color that is identical across 2 calls.
- AC9 `discovery_active_window`: a temp root holds 3 transcripts with mtimes now-2 min, now-10 min, and now-2 h, and `activeWindowMinutes=30`. `initial_scan` returns 2 sessions. `is_registry_file("123.json")` is true, and `is_registry_file("123.abc.key")` and `is_registry_file("x.json")` are false.
- AC10 `long_transcript_bounded`: 200,000 synthetic lines (tool_use/tool_result pairs, 10% never resolved) are applied. Afterwards `recent.len() <= 20`, `pending.len() <= 256`, and every stored string is at most 200 chars.
- AC11: `cargo run -p office-core --bin office-dump -- --claude-dir crates/office-core/tests/fixtures/root --once --now-ms 1795000000000` prints one JSON object. `(… | ConvertFrom-Json).sessions.Count` equals 2, and `title` values are `"Fixture A"` (from custom-title) and `"fake-b"` (project fallback).
- AC12 (live latency): run `AGENT_OFFICE_CLAUDE_DIR=<tmp> AGENT_OFFICE_LOG=debug` with `cargo run -p agent-office`, then `node scripts/simulate.mjs --root <tmp> --scenario spawn`. For each simulator `WRITE <ts>` line, the log has an `emit delta` line at most 1000 ms later, and a new avatar with a drop-in animation appears on the floor.
- AC13 (floors): with `--scenario multi` (3 sessions), the sidebar lists 3 floors labeled with the registry `name`. Clicking each floor shows only that session's agents (desk count = 1 + its subagents). The selected floor stays selected across deltas.
- AC14 (tooltip): hovering an avatar shows within 100 ms the role label, the activity, the target (basename for paths, first 60 chars for commands), active time as `mm:ss`, and tokens (`out / ctx`). It hides on pointer leave.
- AC15 (idle CPU): use the `--scenario static` root (5 sessions, no writes), with the window visible and unfocused. Measure gives `cpuPercent <= 1.0` with `ambientFps=2` and at least one working agent. With all agents in waiting, sleeping, or done it gives `cpuPercent <= 0.3`. Minimized, it gives `cpuPercent <= 0.3`.
- AC16 (idle RAM): use the static root with 5 sessions x 4 subagents, after 5 min. Measure gives `privateMB <= 200` and `hostPrivateMB <= 40`.
- AC17 (soak): run `--scenario soak --minutes 10 --rate 20`, which writes 20 lines/s, 5% of them 50 KB tool_results. `hostPrivateMB` at minute 10 minus minute 1 is at most 15, average `cpuPercent` over the soak is at most 5.0, and the app does not crash.
- AC18 (on-demand render): the F3 debug overlay shows `frames`. Over 30 s with no deltas and `ambientFps=0`, `frames` does not change. With `ambientFps=2` and 1 working agent it grows by at most 70. With the window minimized for 30 s it grows by 0.
- AC19 (no network, no tokens): `cargo tree -i reqwest`, `-i hyper`, and `-i ureq` each report the package is not in the graph. `rg -n "fetch\(|XMLHttpRequest|WebSocket|EventSource" ui/src` returns no matches. During AC15, `Get-NetTCPConnection -OwningProcess <host pid>` lists no non-loopback remote address.
- AC20 `tail_truncation_resets`: after truncating a tracked file below its offset, `read_new` resets to offset 0 and re-reads without error. Deleting the file returns `Ok(vec![])`, and the office drops that agent or session on the next scan.
- AC21 `config_defaults_and_invalid`: missing or invalid JSON config -> `Config::default()` plus one warning. `{"activeWindowMinutes":5}` overrides only that field.
- AC22 `session_lifecycle`: with an injected `ProcessProbe`, a registry session whose pid becomes dead -> status `closed`, then it is removed when `now` passes `closedGraceSeconds` (30). A non-registry session is removed once its newest mtime is older than the window. Error path: a missing `<claudeDir>` gives an empty snapshot plus `error:"claude dir not found: <path>"`, which the UI shows as an empty-state message.
- AC23 (data-access disclosure): `README.md` has a heading `## Data access and privacy` placed before any build or run instructions. The section states each of these facts in plain words:
  - (a) The app is granted read access to the user's local Claude Code data: `~/.claude/projects/**` (transcripts, subagent `.jsonl` and `.meta.json`, `custom-title.json`) and `~/.claude/sessions/<pid>.json` (session registry).
  - (b) This access is what it needs to build the agent office.
  - (c) It is read-only.
  - (d) It never sends anything over the network and uses no LLM tokens.
  - (e) It never opens `*.key` files.
  - (f) The only files it writes are its own config in `%APPDATA%\agent-office\` and the optional debug log in `%LOCALAPPDATA%\agent-office\`.

  Check: `rg -n "^## Data access and privacy" README.md` returns 1 match. Within that section, `rg -i` finds each of `read-only`, `\.claude[\\/]projects`, `\.claude[\\/]sessions`, `network`, and `\.key`. A manual read confirms (a) to (f).

## Design decisions and trade-offs
Stack comparison. These are estimates for a tiny 3D scene on Windows, checked against public benchmarks that disagree with each other.

| Option | Idle RAM | Idle CPU with on-demand | Installer | Dev effort | Watching + UI fit |
|---|---|---|---|---|---|
| Tauri 2 + Three.js | 80-150 MB tree (WebView2 is the floor) | ~0; Chromium pauses hidden pages | 3-10 MB (system WebView2) | Medium | Rust watcher; HTML/CSS sidebar and tooltips are trivial |
| Electron + Three.js | 120-300 MB | ~0 | 80-120 MB | Lowest | Good, but ships a whole Chromium and Node |
| Unity 6 | 150-300 MB | Player loop always ticks; OnDemandRendering only skips rendering | 60-120 MB | Medium-high, heavy editor | Works; overkill |
| Godot 4 | 60-120 MB | low-processor mode is good | 40-90 MB | Medium; not installed | OK |
| Bevy / wgpu+egui | 40-100 MB | `WinitSettings::desktop_app()` reactive is good | 10-40 MB | High: weak UI and text, long compiles, API churn | Same Rust core |

- Choice: Tauri 2 + Three.js. It has the best UI fit and a small installer, and its idle CPU is effectively zero with on-demand rendering. The RAM cost over native is moderate and bounded by WebView2. Unity works but is too heavy for this. Electron loses on size and RAM for no benefit. The user accepted the 150-200 MB budget on 2026-11-18 (Q1), so Bevy is only a fallback if AC16 fails materially.
- Renderer swap: all semantics (activity classification, roles, colors, lifecycle) live in `office-core` and are exposed as a serde JSON `Snapshot`/`Delta` contract. `office-dump --follow` streams the same contract as NDJSON on stdout, so a Bevy, Godot, or Unity renderer could consume it without touching the parser (`docs/decisions/0003-renderer-agnostic-core.md`).
- Open-session detection: the registry (`useSessionRegistry`, default ON, user-approved 2026-11-18) is a hint layered on the mtime rule. A session is open if its registry pid is alive (OpenProcess + creation time = `procStart`), or if it has no registry entry and its newest file mtime is within `activeWindowMinutes` (default 30). The registry `status:"waiting"` maps to the main agent `waiting` (for example, a permission prompt). Agent activity always comes from transcripts, because the registry can get stuck on `busy`.
- Hooks are not needed. Transcripts cover everything. Hooks would also edit the user's Claude settings and spawn a process per tool call inside their sessions.
- Watcher: `notify` 8 (ReadDirectoryChangesW) watches `projects/` recursively and `sessions/` non-recursively, with a 100 ms debounce. A fallback poll opens tracked active files and compares their length to the offset, every `pollFallbackMs` (3000) while any agent is working and every 15 s otherwise. This guards against NTFS delaying change notifications for files the writer holds open.
- Worker: one thread owns `Office`. It blocks on `recv_timeout(min(next_deadline, poll))`, so there is no fixed tick. It emits `office://delta` coalesced to at most one emit per 100 ms. While hidden (`set_visible(false)`) it only marks itself dirty, and on show the UI re-fetches the snapshot.
- Initial read is bounded: for files larger than `maxInitialReadBytes` (8 MiB) it seeks to size-8 MiB, skips to the next newline, and marks tokens `partial:true` (the UI shows `>=`). Done subagents whose files are older than the window are not parsed. Their done state comes from the parent tool_result or meta, and their tokens are unknown.

## Risks and edge cases
- The registry and the transcript schema are undocumented and can change. The parser ignores unknown fields and types, and the registry is optional. Both are covered by AC4 and AC22.
- PID reuse is handled by comparing creation time to `procStart`. If `procStart` is missing, pid liveness alone is used.
- Huge lines (MB-sized tool_results) are parsed as typed structs with only the needed fields, and `content` text is skipped via `serde::de::IgnoredAny` where possible. Lines over 16 MiB are skipped and counted.
- Sessions with 60+ subagents: the floor shows active agents plus the last `maxDoneShown` (8) done agents. The rest appear as `+N archived` on the floor sign.
- Nested subagents (spawnDepth 2) find their parent by looking up `toolUseId` in every agent's Agent tool_uses. If the parent is unknown, the subagent attaches to main.
- A meta file can be missing or arrive late. The role then falls back to the `attributionAgent` of the first assistant line, then to `"Subagent"`, and is updated when the meta appears.
- Truncation, deletion, and rotation are handled by AC20. A missing claude dir is handled by AC22.
- WebView2 RAM is outside our control. If AC16 fails by more than 25%, stop and escalate rather than tune endlessly.
- Clock: the app uses line timestamps for `activitySince` and the system clock for sleep thresholds. Timestamps that are wrong or skewed are clamped to `<= now`.

## Ordered steps
1. **Workspace scaffold.** Create root `Cargo.toml` and `.gitignore` (target, node_modules, ui/dist). Create `README.md` in this order: a one-paragraph description, `## Data access and privacy` (all of AC23 a-f, as a short bullet list), build and run, config keys with defaults (including `useSessionRegistry: true`), and the env vars `AGENT_OFFICE_CLAUDE_DIR` and `AGENT_OFFICE_LOG`. Create `ui/` with Vite + TS and the `three` and `@tauri-apps/api` dependencies, plus `@tauri-apps/cli` as a dev dependency. Create `src-tauri/` from the Tauri 2 minimal template with `frontendDist: "../ui/dist"` and `devUrl: "http://localhost:5173"`.
2. **`model.rs`** (serde `rename_all="camelCase"`):
   - `Activity` enum (lowercase strings): thinking, writing, reading, running, browsing, delegating, planning, tool, waiting, sleeping, done, error.
   - `Tokens { input, output, cache_read, cache_write, context: u64, partial: bool }`.
   - `RoleInfo { key, label, color, team }`.
   - `AgentView { id, parent_id: Option<String>, role, description: Option<String>, activity, tool_name: Option<String>, target: Option<String>, activity_since, started_at, last_activity_at: i64, tokens, model: Option<String>, spawn_depth: u8 }`.
   - `SessionView { id, title, project, cwd, git_branch: Option<String>, status: SessionStatus /*working|waiting|idle|closed*/, source: "registry"|"mtime", started_at, last_activity_at, agents: Vec<AgentView>, archived_done: u32 }`.
   - `Snapshot { generated_at, sessions, error: Option<String> }`.
   - `Delta { sessions_upserted: Vec<SessionView /* agents empty */>, sessions_removed: Vec<String>, agents_upserted: Vec<(String, AgentView)>, agents_removed: Vec<(String, String)>, spawned: Vec<Spawn { session_id, agent_id, from_agent_id }> }`.
3. **`line.rs`**: `pub fn parse_line(bytes: &[u8]) -> Result<Vec<LineEvent>, ParseError>`. The `LineEvent` variants are:
   - `ToolUse { id, name, target, msg_id, ts }`
   - `AssistantMsg { msg_id, usage: Option<Usage>, stop_reason: Option<String>, model, attribution_agent, ts }`
   - `ToolResult { tool_use_id, is_error, ts }`
   - `UserPrompt { ts }` (user line with string or text content)
   - `TurnEnd { ts }` (`stop_hook_summary`)
   - `ApiError { ts }`
   - `Title(String)` (custom-title), `AgentName(String)`, `LastPrompt(String)`
   - `Context { cwd, git_branch }`
   - An unknown type yields `Ok(vec![])`.

   Target extraction for `ToolUse`, truncated to 200 chars:
   - `file_path` / `notebook_path` for Read/Write/Edit/MultiEdit/NotebookEdit
   - `pattern` (+ ` in <path>`) for Grep/Glob
   - `command` for Bash/PowerShell
   - `url` / `query` for Web tools
   - `"<subagent_type>: <description>"` for Agent/Task, plus a `spawn: Option<SpawnInfo{subagent_type, description, background}>` field.
4. **`tail.rs`**: `pub struct TailReader { path, offset: u64, partial: Vec<u8> }`.
   - `new(path, start: StartAt::{Beginning, LastBytes(u64)})`.
   - `read_new(&mut self) -> io::Result<Vec<Vec<u8>>>`: opens with shared read/write/delete access (on Windows use `OpenOptionsExt::share_mode(7)`), reads from `offset` to EOF in 64 KiB chunks, returns complete lines, and keeps the remainder in `partial`. The `partial` buffer is capped at 16 MiB: when it overflows, enter skip mode until the next `\n` and increment `stats.oversized`. If len < offset, reset. A missing file returns `Ok(vec![])` and sets `gone=true`.
5. **`reducer.rs`**: `pub struct AgentTracker` holds the view fields, `pending: VecDeque<(id,name,target,ts)>` (cap 256, evict oldest), `recent` ring (cap 20), and `seen_msg_ids` (last 64). It has `apply(&mut self, ev: &LineEvent, now_ms) -> bool` (changed) and `tick(&mut self, now_ms, cfg) -> bool`.
   - Classification by tool name: Write/Edit/MultiEdit/NotebookEdit -> writing; Read/Grep/Glob/LS -> reading; Bash/PowerShell/BashOutput/KillShell -> running; WebFetch/WebSearch -> browsing; Agent/Task/SendMessage/ListAgents/ReadNotifications -> delegating; TodoWrite -> planning; anything else, including `mcp__*` -> tool.
   - Transitions:
     - A ToolUse sets the activity from the newest pending entry.
     - A ToolResult removes the entry from pending and falls back to the previous pending entry or `thinking`.
     - UserPrompt and AssistantMsg without tools -> thinking.
     - TurnEnd, or `end_turn` with an empty pending list -> waiting (main) or done (subagent).
     - A `SubagentHandback` tool_use -> done.
     - ApiError -> error for 10 s, then the previous state.
     - From `tick`: no line for `sleepAfterSeconds` (300) while waiting or thinking -> sleeping.
   - Tokens are summed once per new `msg_id`. `context` = the last usage's input + cache_read + cache_creation.
6. **`roles.rs`**: `pub fn role_for(agent_type: Option<&str>, is_main: bool) -> RoleInfo`. It strips everything up to the last `:`, then looks up the exact key:

   | Key | Label | Color |
   |---|---|---|
   | main | Main | #F2C14E |
   | software-architect / architect | Architect | #4F7CAC |
   | software-engineer / coder / engineer | Coder | #5FA35F |
   | software-test-engineer / tester | Tester | #D9A441 |
   | software-debugger | Debugger | #C8553D |
   | software-code-reviewer | Code reviewer | #8E6BBF |
   | software-security-reviewer | Security | #B23A48 |
   | software-performance-reviewer | Performance | #E07A2E |
   | software-db-engineer | DB engineer | #3A8F8F |
   | software-devops | DevOps | #6C7A89 |
   | software-diagrammer | Diagrammer | #C76FA8 |
   | software-docs-writer | Docs | #9AA36B |
   | Explore | Explorer | #7FB3D5 |
   | general-purpose | General | #A0A0A0 |
   | Plan | Planner | #6A8EAE |

   Prefix families, with the label from the suffix title-cased:
   - `product-*`: team product, base #2A9D8F
   - `content-*`: team content, base #E76F9A
   - `game-*`: team game, base #7B5EA7

   Unknown types get a title-cased label and a color from an 8-color palette indexed by FNV-1a hash.
7. **`config.rs`**: `Config { claude_dir: PathBuf, active_window_minutes: u32 = 30, sleep_after_seconds: u32 = 300, closed_grace_seconds: u32 = 30, max_done_shown: u32 = 8, ambient_fps: u32 = 2, poll_fallback_ms: u32 = 3000, max_initial_read_bytes: u64 = 8_388_608, use_session_registry: bool = true }` with serde defaults.
   - `Config::load(path) -> (Config, Option<String /*warning*/>)`.
   - `claude_dir` precedence: env `AGENT_OFFICE_CLAUDE_DIR` > config > `%USERPROFILE%\.claude`.
8. **`probe.rs` and `discovery.rs`**: `trait ProcessProbe { fn is_alive(&self, pid: u32, proc_start: Option<u64>) -> bool }`. The Windows impl uses `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)`, `GetExitCodeProcess == STILL_ACTIVE`, and `GetProcessTimes` creation time compared to `procStart`.
   - `is_registry_file(name)` matches `^\d+\.json$`.
   - `read_registry(dir) -> Vec<RegistryEntry>` skips bad files.
   - `slug_for(cwd)`.
   - `scan_recent(projects_dir, cutoff_ms) -> Vec<SessionFiles>` stats `*.jsonl` one level deep and lists `subagents/` only for sessions that pass.
   - Locate a registry session's transcript by slug first, then by `projects/*/<sessionId>.jsonl`.
9. **`office.rs`**: `pub struct Office` with:
   - `new(cfg, probe: Box<dyn ProcessProbe>)`
   - `initial_scan(now_ms) -> Snapshot`
   - `on_paths_changed(&[PathBuf], now_ms) -> Option<Delta>`: classifies paths as main jsonl, subagent jsonl, meta, `custom-title.json`, or registry. It ignores everything else and never opens `*.key`.
   - `poll(now_ms) -> Option<Delta>`, `tick(now_ms) -> Option<Delta>`, `next_deadline_ms() -> Option<i64>`, `snapshot() -> Snapshot`, `tracked_paths() -> Vec<PathBuf>`.
   - Title priority: registry `name` > latest custom-title (line or json) > agentName > lastPrompt (first 40 chars) > project basename.
   - Session status: waiting if main is waiting or the registry says waiting; working if any agent is in a working activity; idle otherwise; closed per AC22.
   - It emits `spawned` when a subagent first appears and sets `from_agent_id` to the agent that owns the matching Agent tool_use.
10. **`office-dump`**: flags `--claude-dir`, `--once` (prints the snapshot), `--follow` (polls every 500 ms and prints each `Delta` as NDJSON), `--now-ms` (fixed clock, for tests). It uses std only.
11. **Fixtures and tests**: write the fixtures named in the ACs, with synthetic ids and the made-up paths `C:\fake\a` and `C:\fake\b`. `fixtures/root/` contains:
    - `projects/C--fake-a/`: a session with custom-title, subagents a1 (foreground) and a2 (background), and their metas
    - `projects/C--fake-b/`: one session
    - `sessions/`: one registry json with a pid that the test probe reports as dead, and one `.key` file with dummy content
   
   Discovery tests copy the root to a tempdir and set mtimes with `filetime`.
12. **`src-tauri`**:
    - `lib.rs` builds the app, loads the config from `app_config_dir()/config.json`, spawns the worker, and registers the commands `get_snapshot() -> Snapshot` and `set_visible(visible: bool)`.
    - `watcher.rs` wraps `notify::RecommendedWatcher` and forwards paths into an `mpsc::Sender<WorkerMsg>`.
    - `worker.rs` runs the loop described in the design decisions and emits with `app.emit("office://delta", &delta)`.
    - `logging.rs` is active only when `AGENT_OFFICE_LOG=debug`. It writes timestamped lines (`emit delta n=<k>`) to stderr and `%LOCALAPPDATA%\agent-office\agent-office.log`.
    - `tauri.conf.json`: one window, 1200x760, title "Agent Office". `bundle.targets: ["nsis"]`. `app.security.csp`: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src ipc: http://ipc.localhost`.
    - `capabilities/default.json` grants only `core:event:default`.
13. **UI data**: `types.ts` mirrors `model.rs`. `ipc.ts` provides `getSnapshot()`, `onDelta(cb)`, and `setVisible(v)`. `store.ts` holds `Map<sessionId, SessionView>` and `selectedId`, and `apply(delta)` returns the changed ids. On `visibilitychange` the UI calls `setVisible`, and on show it re-fetches the snapshot.
14. **UI scene**:
    - `renderer.ts`: `requestRender()` coalesces into one rAF and increments `frames`. `runTween(ms)` keeps rAF only while tweens are active. An ambient `setInterval(1000/ambientFps)` runs only while visible and at least one agent on the selected floor is in a working activity. Everything stops when hidden.
    - The camera is an `OrthographicCamera` at (10,12,10) looking at the floor center. The mouse wheel zooms (0.5-2x) and re-renders.
    - Lights: one hemisphere and one directional light, using `MeshLambertMaterial({flatShading:true})`.
    - `office.ts`: `buildFloor(session)` builds a floor plane, 2 low walls, and a floor sign (DOM). `deskSlot(i)` places main front-center and subagents in a 4-column grid. It rebuilds only the agent groups whose ids changed and reuses shared geometries.
    - `desk.ts`: box desk, box monitor, box chair.
    - `avatar.ts`: a cylinder body and an icosahedron head (detail 0) in the role color. `setActivity(a)` sets the monitor color from `palette.ts`, the pose (seated, standing for delegating, head down for sleeping, greyed out for done), and a DOM bubble (`...`, `?`, `zZ`, `!`, a check mark).
    - `tweens.ts`: spawn drop-in (scale 0 to 1, 600 ms) and an envelope cube flying on a bezier from the parent desk to the new desk (800 ms).
15. **UI chrome**:
    - `sidebar.ts`: elevator panel with floors sorted by `startedAt` (newest on top), each showing number, title, project, a status dot, and a working-agent count.
    - `tooltip.ts`: pointermove throttled to 30 Hz, raycasting only against avatar meshes (AC14). The active time is computed client-side, and a 1 s update timer runs only while the tooltip is visible.
    - `debug.ts`: F3 overlay with frames, last delta time, and the number of sessions and agents.
16. **Scripts**:
    - `simulate.mjs` (node, no deps) supports the scenarios static, basic, spawn, multi, and soak. It writes registry files with its own pid, which it deletes on exit, and prints `WRITE <iso> <what>`.
    - `measure-resources.ps1` behaves as described above the ACs.

## Open questions and assumptions
- Q1 (resolved 2026-11-18): the user accepts 150-200 MB total RAM. The stack stays Tauri 2 + Three.js, and the AC16 budget is unchanged.
- Q2 (resolved 2026-11-18): the user allows the `~/.claude/sessions` registry and any other local Claude Code data the app needs. `useSessionRegistry` defaults to true. `*.key` files are still never opened.
- A1: one floor per session. Sessions whose transcripts are newer than the window but whose registry pid is dead are hidden. Claude Desktop scratch-workspace projects are treated like any other project.
- A2: ambient animation defaults to 2 fps (stop-motion style). Setting it to 0 gives zero frames when nothing changes.
- A3: prompts and paths are shown locally only, and nothing is persisted except the config.

## Nice-to-haves (not v1)
System tray with a badge, in-app settings, camera rotate in 90-degree steps, a timeline or replay, sound cues, hooks as an optional lower-latency source, WebView2 `MemoryUsageTargetLevel=Low` when minimized, and a Bevy renderer that consumes `office-dump --follow`.

## Memory
- Decisions recorded and accepted (user-approved 2026-11-18): `docs/decisions/0001-tauri-threejs-stack.md`, `0002-transcripts-only-data-source.md`, `0003-renderer-agnostic-core.md`.
- Lesson: Claude Code splits one API message across several JSONL lines that repeat `usage`. Always dedupe tokens by `message.id`.

Sources: https://betterstack.com/community/guides/scaling-nodejs/tauri-vs-electron-vs-deno-vs-electrobun/ , https://dev.to/gethopp/tauri-vs-electron-performance-bundle-size-and-the-real-trade-offs-1el4 , https://docs.rs/crate/bevy_winit/0.18.1/source/src/winit_config.rs , https://docs.unity3d.com/ScriptReference/Rendering.OnDemandRendering-renderFrameInterval.html , https://docs.rs/crate/notify/latest/source/src/windows.rs , https://pypi.org/project/claude-busy-monitor/ , https://claudeissues.com/issue/87131-bug-parking-a-mid-flight-turn-leaves-the-interactive-sessions-probe-stuck-at-sta

## Revisions
- 2026-11-18: The user approved the plan.
  - Q1 and Q2 are resolved: the 150-200 MB RAM budget is accepted, and use of the registry and other local Claude Code data is allowed. `useSessionRegistry` is stated as default ON, and `*.key` files are still never opened.
  - New requirement: the README discloses read-only access to local Claude Code data. `README.md` now has its own entry in Files to touch, Step 1 specifies the README structure, and AC23 was added.
  - ADRs 0001-0003 changed from proposed to accepted.
