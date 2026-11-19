# Agent Office

Agent Office is a small Windows 11 desktop app that shows every open Claude Code session as a building of a low-poly 3D office, seen from an isometric camera. Each building has one floor per team, each known role has its own themed room, and the main agent has a boss office. People work at desks with live activity states (thinking, writing, running, waiting, sleeping, done) and walk between rooms to show handoffs. It only reads local files, never calls a model, and keeps RAM and CPU use low: it renders on demand and does nothing while its window is hidden.

## Data access and privacy

- The app is granted read access to your local Claude Code data: `~/.claude/projects/**` (session transcripts, subagent `.jsonl` and `.meta.json` files, and `custom-title.json`) and `~/.claude/sessions/<pid>.json` (the session registry).
- This access is what it needs to build the agent office: which sessions are open, which agents they run, and what each agent is doing.
- The access is read-only. The app never modifies, creates, or deletes anything under `~/.claude`.
- The app itself makes no network connections and sends none of your data anywhere, and it uses no LLM tokens. (The WebView2 runtime that hosts the page may still make its own background connections; see "Network lockdown".) There is no HTTP client in the app, the page has a strict content security policy, and no Tauri plugins are enabled.
- It never opens `*.key` files (the secrets that sit next to the registry files).
- The only files it writes are its own config in `%APPDATA%\agent-office\` and the optional debug log in `%LOCALAPPDATA%\agent-office\`. (The WebView2 runtime keeps its own browser cache under `%LOCALAPPDATA%\dev.agent-office.app\`; the app itself does not write there.)

Prompts, file paths, and commands from your sessions are shown on screen only. Nothing is persisted.

## Floors and teams

- A session is a building. The sidebar lists the buildings; the selected one expands into five floor rows (top-down): 5 Game, 4 Content, 3 Product, 2 Software, 1 General. Floors with no agents are dimmed but can still be opened. Only the selected floor is drawn.
- Each floor is a cutaway with one room per known role (for example Software has Architect, Coder, Tester, Debugger, ...), an Other room, the boss office for Main, a lounge, and a stairwell. Explore, Plan, general-purpose, and unknown agent types live on the General floor.
- A room is lit while an agent in it is working, dim while the agent waits, and dark when empty.
- Handoffs are visible walks. When Main spawns a subagent it walks along the corridors to that room and the new person appears at a desk. When a subagent finishes it walks back to the boss office. When work moves between floors (for example product spec returns and Main then spawns a software architect within 10 minutes) a person carrying a box takes the stairs, and the floor rows show an up or down arrow for 3 seconds. The 10 minute relay window is the `relayWindowSeconds` config key (default 600) and is a heuristic.
- The view buttons at the bottom right rotate the camera by 90 degrees and refit it. The mouse wheel zooms.
- Press F3 for a debug overlay (frames, floor, room light counts, walkers, draw calls, geometries, fit).

## Network lockdown

The WebView2 browser process gets extra command-line flags (`app.windows[0].additionalBrowserArgs` in `src-tauri/tauri.conf.json`) so that it makes no background connections of its own. The setting replaces Tauri's default `--disable-features` list, so that list is repeated first.

```
--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,DnsOverHttps,AsyncDns
--disable-background-networking --disable-component-update --disable-domain-reliability
--disable-sync --disable-client-side-phishing-detection --no-pings --disable-quic
--proxy-server=http://127.0.0.1:9
--proxy-bypass-list=localhost;*.localhost;127.0.0.1;[::1]
```

The proxy points at a closed loopback port, so any HTTP(S) request that is not to localhost fails; Tauri's IPC and the dev server stay on the bypass list. These are best-effort, undocumented flags: Microsoft documents only some of them and warns that WebView2 flags may change between runtime versions. **Known limitation.** The app itself makes no network connections and sends none of your data. Even with these flags, WebView2's own browser processes (`msedgewebview2.exe`) were observed contacting Microsoft, Cloudflare, and public DNS endpoints (for example 52.97.x.x:443, 172.66.2.5:80, 8.8.8.8:443 and 8.8.4.4:443) for the runtime's internal checks. This was accepted. If you want zero outside connections, add a Windows Firewall rule that blocks outbound traffic for `msedgewebview2.exe` when launched by Agent Office; that is your option and the app does not change firewall settings.

Check after every WebView2 update with `scripts/measure-resources.ps1 -Name agent-office -Seconds 120 -Network`, which lists any non-loopback TCP connection of the app and its WebView2 child processes (`remoteConnections` from the app process itself should be 0; WebView2 processes are expected to show some, see Known limitation).

## Build and run

Requirements: Windows 11, Node 20+, Rust (stable) with the MSVC toolchain, and the WebView2 runtime (included in Windows 11).

```powershell
npm ci --prefix ui
npx --prefix ui tauri dev        # development
npx --prefix ui tauri build      # NSIS installer in target/release/bundle/nsis/
cargo test -p office-core        # core tests (no GUI needed)
```

Other tools:

- `cargo run -p office-core --bin office-dump -- --once` prints the current state as JSON. `--follow` streams each change as NDJSON, so another renderer could consume the same contract.
- `node scripts/simulate.mjs --root <tmp> --scenario spawn|multi|static|basic|soak` writes fake sessions into a temp folder. Point the app at it with `AGENT_OFFICE_CLAUDE_DIR=<tmp>`.
- `pwsh scripts/measure-resources.ps1 -Name agent-office -Seconds 60` prints `{cpuPercent, privateMB, hostPrivateMB, processCount}` for the app and its WebView2 children.

## Config

Optional file `%APPDATA%\agent-office\config.json`. Missing or invalid files fall back to the defaults (one warning in the debug log). Keys are all optional:

| Key | Default | Meaning |
|---|---|---|
| `claudeDir` | `%USERPROFILE%\.claude` | Claude Code data folder |
| `activeWindowMinutes` | `30` | A session is shown while its transcript changed within this window, or while the registry says it is busy or waiting. A live but idle process alone is not enough (the desktop app keeps recent sessions running in the background) |
| `sleepAfterSeconds` | `300` | An idle agent (waiting or thinking) becomes sleeping after this |
| `closedGraceSeconds` | `30` | A closed session stays visible this long |
| `maxDoneShown` | `8` | Finished subagents shown per floor; the rest become `+N archived` |
| `ambientFps` | `2` | Idle animation rate while agents work. `0` means no frames unless something changes |
| `pollFallbackMs` | `3000` | Fallback file poll while any agent is working (15 s otherwise) |
| `maxInitialReadBytes` | `8388608` | At most this many trailing bytes of a transcript are read at startup |
| `useSessionRegistry` | `true` | Use `~/.claude/sessions/<pid>.json` to detect open and closed sessions |
| `relayWindowSeconds` | `600` | A subagent spawned by Main within this window after a subagent of another team returned counts as a cross-floor relay |

## Environment variables

- `AGENT_OFFICE_CLAUDE_DIR`: overrides `claudeDir` (config and default).
- `AGENT_OFFICE_LOG=debug`: logs to stderr and `%LOCALAPPDATA%\agent-office\agent-office.log`.

Press `F3` in the app for a debug overlay (frames rendered, last update, session and agent counts).
