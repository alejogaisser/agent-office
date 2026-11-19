//! QA edge-case tests beyond the plan's acceptance criteria. Synthetic data only.
use office_core::line::{parse_into, parse_line, LineEvent, ParseStats};
use office_core::reducer::{AgentTracker, PENDING_CAP, RECENT_CAP};
use office_core::roles::role_for;
use office_core::tail::{StartAt, TailReader};
use office_core::{Activity, Config, Office, ProcessProbe, SessionStatus, SystemProbe};
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

const NOW: i64 = 1_795_000_000_000;

#[derive(Clone, Default)]
struct Probe(Arc<Mutex<HashSet<u32>>>);
impl ProcessProbe for Probe {
    fn is_alive(&self, pid: u32, _: Option<u64>) -> bool {
        self.0.lock().unwrap().contains(&pid)
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
}

fn tool_use_line(id: &str, name: &str, input: &str) -> String {
    format!(
        "{{\"type\":\"assistant\",\"timestamp\":\"2026-11-17T10:00:00.000Z\",\"message\":{{\"id\":\"m_{id}\",\"stop_reason\":\"tool_use\",\"content\":[{{\"type\":\"tool_use\",\"id\":\"{id}\",\"name\":\"{name}\",\"input\":{input}}}]}}}}"
    )
}

fn feed(t: &mut AgentTracker, line: &str, now: i64) {
    for ev in parse_line(line.as_bytes()).unwrap() {
        t.apply(&ev, now);
    }
}

#[test]
fn oversized_line_is_skipped_and_reading_continues() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("big.jsonl");
    {
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(b"{\"a\":1}\n").unwrap();
        let chunk = vec![b'x'; 1024 * 1024];
        for _ in 0..17 {
            f.write_all(&chunk).unwrap();
        }
        f.write_all(b"\n{\"b\":2}\n").unwrap();
    }
    let mut r = TailReader::new(&p, StartAt::Beginning);
    let lines = r.read_new().unwrap();
    assert_eq!(lines, vec![b"{\"a\":1}".to_vec(), b"{\"b\":2}".to_vec()]);
    assert_eq!(r.stats.oversized, 1);
    assert!(r.partial.len() < 100);
}

#[test]
fn crlf_and_delete_then_recreate() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.jsonl");
    std::fs::write(&p, b"{\"a\":1}\r\n").unwrap();
    let mut r = TailReader::new(&p, StartAt::Beginning);
    assert_eq!(r.read_new().unwrap(), vec![b"{\"a\":1}".to_vec()]);
    std::fs::remove_file(&p).unwrap();
    assert!(r.read_new().unwrap().is_empty());
    assert!(r.gone);
    std::fs::write(&p, b"{\"c\":3}\n").unwrap();
    assert_eq!(r.read_new().unwrap(), vec![b"{\"c\":3}".to_vec()]);
    assert!(!r.gone);
}

#[test]
fn parser_never_panics_on_garbage() {
    let cases: Vec<Vec<u8>> = vec![
        b"".to_vec(),
        b"null".to_vec(),
        b"[]".to_vec(),
        b"123".to_vec(),
        b"\"str\"".to_vec(),
        b"{".to_vec(),
        vec![0xff, 0xfe, 0x00, b'{'],
        b"{\"type\":\"assistant\"}".to_vec(),
        b"{\"type\":\"assistant\",\"message\":null}".to_vec(),
        b"{\"type\":\"assistant\",\"message\":\"x\"}".to_vec(),
        b"{\"type\":\"assistant\",\"message\":{\"content\":\"text\"}}".to_vec(),
        b"{\"type\":\"assistant\",\"message\":{\"content\":[1,null,\"a\",{\"type\":\"tool_use\"}]}}".to_vec(),
        b"{\"type\":\"user\",\"message\":{\"content\":[{\"type\":\"tool_result\"}]}}".to_vec(),
        b"{\"type\":\"system\",\"subtype\":5}".to_vec(),
        b"{\"type\":\"assistant\",\"timestamp\":\"garbage\",\"message\":{\"usage\":{\"input_tokens\":-5,\"output_tokens\":1e30}}}".to_vec(),
    ];
    let mut stats = ParseStats::default();
    for c in &cases {
        let _ = parse_into(c, &mut stats);
    }
    // deeply nested JSON must not overflow the stack
    let deep = format!("{}{}", "[".repeat(5000), "]".repeat(5000));
    let _ = parse_into(deep.as_bytes(), &mut stats);
    let deep_obj = format!("{{\"type\":\"user\",\"x\":{}1{}}}", "[".repeat(5000), "]".repeat(5000));
    let _ = parse_into(deep_obj.as_bytes(), &mut stats);
}

#[test]
fn multibyte_strings_truncate_on_char_boundary() {
    let long = "é".repeat(500);
    let line = tool_use_line("t1", "Bash", &format!("{{\"command\":\"{long}\"}}"));
    let evs = parse_line(line.as_bytes()).unwrap();
    let mut t = AgentTracker::new("main", true, NOW);
    for ev in &evs {
        if let LineEvent::ToolUse { target: Some(tg), .. } = ev {
            assert!(tg.chars().count() <= 200, "parser target too long: {}", tg.chars().count());
        }
        t.apply(ev, NOW);
    }
    assert!(t.target.as_ref().unwrap().chars().count() <= 200);
}

#[test]
fn pending_and_recent_bounded_with_unresolved() {
    let mut t = AgentTracker::new("main", true, NOW);
    for i in 0..1000 {
        feed(&mut t, &tool_use_line(&format!("id{i}"), "Read", "{\"file_path\":\"a.ts\"}"), NOW);
    }
    assert!(t.pending.len() <= PENDING_CAP);
    assert!(t.recent.len() <= RECENT_CAP);
}

#[test]
fn error_reverts_after_ten_seconds_and_sleep_after_300() {
    let cfg = Config::default();
    let mut t = AgentTracker::new("main", true, NOW);
    feed(&mut t, &tool_use_line("e1", "Bash", "{\"command\":\"npm test\"}"), NOW);
    assert_eq!(t.activity, Activity::Running);
    t.apply(&LineEvent::ApiError { ts: NOW }, NOW);
    assert_eq!(t.activity, Activity::Error);
    assert!(!t.tick(NOW + 5_000, &cfg));
    assert_eq!(t.activity, Activity::Error);
    assert!(t.tick(NOW + 10_000, &cfg));
    assert_eq!(t.activity, Activity::Running);

    let mut w = AgentTracker::new("main", true, NOW);
    w.apply(&LineEvent::TurnEnd { ts: NOW }, NOW);
    assert_eq!(w.activity, Activity::Waiting);
    assert!(!w.tick(NOW + 299_000, &cfg));
    assert!(w.tick(NOW + 300_000, &cfg));
    assert_eq!(w.activity, Activity::Sleeping);
    // waking up
    w.apply(&LineEvent::UserPrompt { ts: NOW + 301_000 }, NOW + 301_000);
    assert_eq!(w.activity, Activity::Thinking);
}

#[test]
fn skewed_future_timestamps_are_clamped() {
    let mut t = AgentTracker::new("main", true, NOW);
    let far_future = NOW + 10 * 86_400_000;
    t.apply(&LineEvent::UserPrompt { ts: far_future }, NOW);
    assert!(t.last_activity_at <= NOW);
    assert!(t.activity_since <= NOW);
    t.apply(&LineEvent::ToolUse { id: "x".into(), name: "Read".into(), target: None, msg_id: None, ts: far_future, spawn: None }, NOW);
    assert!(t.last_activity_at <= NOW && t.activity_since <= NOW);
}

#[test]
fn role_edge_cases_do_not_panic() {
    for s in ["", ":", "::", "gaisser-agents:", "a:b:c", "  ", "ünï-cödé", "game-x-y"] {
        let r = role_for(Some(s), false);
        assert!(!r.label.is_empty(), "empty label for {s:?}");
        assert!(r.color.starts_with('#'));
    }
    assert_eq!(role_for(Some("gaisser-agents:product-writer"), false).label, "Writer");
    assert_eq!(role_for(None, false).label, "Subagent");
    // legacy + prefixed same role
    assert_eq!(role_for(Some("coder"), false).color, role_for(Some("gaisser-agents:software-engineer"), false).color);
}

#[test]
#[cfg(windows)]
fn system_probe_real_processes() {
    let me = std::process::id();
    assert!(SystemProbe.is_alive(me, None));
    // wrong creation time (pid reuse guard) -> not alive
    assert!(!SystemProbe.is_alive(me, Some(1)));
    // a pid that does not exist
    assert!(!SystemProbe.is_alive(0x7FFF_FFF0, None));
    // an exited child
    let mut c = std::process::Command::new("cmd").args(["/C", "exit 0"]).spawn().unwrap();
    let pid = c.id();
    c.wait().unwrap();
    assert!(!SystemProbe.is_alive(pid, None));
}

fn write_session(root: &Path, slug: &str, sid: &str, reg: Option<(u32, &str)>) {
    let proj = root.join("projects").join(slug);
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::create_dir_all(root.join("sessions")).unwrap();
    std::fs::write(
        proj.join(format!("{sid}.jsonl")),
        "{\"type\":\"user\",\"timestamp\":\"2026-11-17T10:00:00.000Z\",\"cwd\":\"C:\\\\q\",\"message\":{\"role\":\"user\",\"content\":\"hi\"}}\n",
    )
    .unwrap();
    if let Some((pid, status)) = reg {
        std::fs::write(
            root.join("sessions").join(format!("{pid}.json")),
            format!("{{\"pid\":{pid},\"sessionId\":\"{sid}\",\"cwd\":\"C:\\\\q\",\"name\":\"Named {sid}\",\"status\":\"{status}\"}}"),
        )
        .unwrap();
    }
}

#[test]
fn dead_registry_session_hidden_and_waiting_status_mapped() {
    let tmp = tempfile::tempdir().unwrap();
    write_session(tmp.path(), "C--q", "dead", Some((7, "busy")));
    write_session(tmp.path(), "C--q", "wait", Some((8, "waiting")));
    write_session(tmp.path(), "C--q", "plain", None);
    let probe = Probe::default();
    probe.0.lock().unwrap().insert(8);
    let cfg = Config { claude_dir: tmp.path().to_path_buf(), ..Config::default() };
    let mut office = Office::new(cfg, Box::new(probe));
    let snap = office.initial_scan(now_ms());
    let ids: HashSet<&str> = snap.sessions.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, HashSet::from(["wait", "plain"]), "dead registry session must be hidden (plan A1)");
    let w = snap.sessions.iter().find(|s| s.id == "wait").unwrap();
    assert_eq!(w.status, SessionStatus::Waiting);
    assert_eq!(w.title, "Named wait");
}

#[test]
fn registry_disabled_ignores_registry_files() {
    let tmp = tempfile::tempdir().unwrap();
    write_session(tmp.path(), "C--q", "dead", Some((7, "busy")));
    let cfg = Config { claude_dir: tmp.path().to_path_buf(), use_session_registry: false, ..Config::default() };
    let mut office = Office::new(cfg, Box::new(Probe::default()));
    let snap = office.initial_scan(now_ms());
    assert_eq!(snap.sessions.len(), 1);
    assert_ne!(snap.sessions[0].title, "Named dead");
}

#[test]
fn key_named_like_registry_is_never_read() {
    // a `<pid>.<hash>.key` must be ignored even when it holds valid registry JSON
    let tmp = tempfile::tempdir().unwrap();
    write_session(tmp.path(), "C--q", "s", None);
    std::fs::write(
        tmp.path().join("sessions/55.abcdef.key"),
        "{\"pid\":55,\"sessionId\":\"s\",\"name\":\"FROM KEY\"}",
    )
    .unwrap();
    let cfg = Config { claude_dir: tmp.path().to_path_buf(), ..Config::default() };
    let mut office = Office::new(cfg, Box::new(Probe::default()));
    let snap = office.initial_scan(now_ms());
    assert!(snap.sessions.iter().all(|s| s.title != "FROM KEY"));
}

#[test]
fn bare_family_prefix_has_a_label() {
    for s in ["product-", "content-", "game-"] {
        assert!(!role_for(Some(s), false).label.is_empty(), "empty label for {s:?}");
    }
}

fn line_user(ts: &str) -> String {
    format!("{{\"type\":\"user\",\"timestamp\":\"{ts}\",\"cwd\":\"C:\\q\",\"message\":{{\"role\":\"user\",\"content\":\"hi\"}}}}\n")
}

fn agent_spawn(id: &str, ty: &str) -> String {
    format!(
        "{{\"type\":\"assistant\",\"timestamp\":\"2026-11-17T10:00:00.000Z\",\"message\":{{\"id\":\"m_{id}\",\"stop_reason\":\"tool_use\",\"content\":[{{\"type\":\"tool_use\",\"id\":\"{id}\",\"name\":\"Agent\",\"input\":{{\"subagent_type\":\"{ty}\",\"description\":\"d {id}\",\"run_in_background\":false}}}}]}}}}\n"
    )
}

fn handback(id: &str) -> String {
    format!(
        "{{\"type\":\"assistant\",\"timestamp\":\"2026-11-17T10:00:01.000Z\",\"message\":{{\"id\":\"mh_{id}\",\"stop_reason\":\"tool_use\",\"content\":[{{\"type\":\"tool_use\",\"id\":\"h_{id}\",\"name\":\"SubagentHandback\",\"input\":{{}}}}]}}}}\n"
    )
}

#[test]
fn many_done_subagents_are_archived() {
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("projects/C--q");
    let subs = proj.join("big/subagents");
    std::fs::create_dir_all(&subs).unwrap();
    let mut main = line_user("2026-11-17T09:59:00.000Z");
    for i in 0..60 {
        main.push_str(&agent_spawn(&format!("tu{i}"), "gaisser-agents:software-engineer"));
        std::fs::write(subs.join(format!("agent-s{i}.jsonl")), format!("{}{}", line_user("2026-11-17T10:00:00.000Z"), handback(&i.to_string()))).unwrap();
        std::fs::write(
            subs.join(format!("agent-s{i}.meta.json")),
            format!("{{\"agentType\":\"gaisser-agents:software-engineer\",\"description\":\"d\",\"toolUseId\":\"tu{i}\",\"spawnDepth\":1}}"),
        )
        .unwrap();
    }
    std::fs::write(proj.join("big.jsonl"), main).unwrap();
    let cfg = Config { claude_dir: tmp.path().to_path_buf(), ..Config::default() };
    let mut office = Office::new(cfg, Box::new(Probe::default()));
    let snap = office.initial_scan(now_ms());
    let s = snap.sessions.iter().find(|s| s.id == "big").expect("session");
    let done = s.agents.iter().filter(|a| a.activity == Activity::Done).count();
    assert!(done <= 8, "shown done agents {done} > maxDoneShown");
    assert_eq!(s.archived_done as usize + done, 60, "archived + shown must cover all done agents");
}

#[test]
fn nested_and_orphan_subagents_attach() {
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("projects/C--q");
    let subs = proj.join("nest/subagents");
    std::fs::create_dir_all(&subs).unwrap();
    std::fs::write(proj.join("nest.jsonl"), format!("{}{}", line_user("2026-11-17T09:59:00.000Z"), agent_spawn("tuA", "Explore"))).unwrap();
    // A spawned by main; B spawned by A (nested, depth 2); C has an unknown toolUseId (orphan)
    std::fs::write(subs.join("agent-A.jsonl"), format!("{}{}", line_user("2026-11-17T10:00:00.000Z"), agent_spawn("tuB", "Plan"))).unwrap();
    std::fs::write(subs.join("agent-A.meta.json"), "{\"agentType\":\"Explore\",\"description\":\"a\",\"toolUseId\":\"tuA\",\"spawnDepth\":1}").unwrap();
    std::fs::write(subs.join("agent-B.jsonl"), line_user("2026-11-17T10:00:00.000Z")).unwrap();
    std::fs::write(subs.join("agent-B.meta.json"), "{\"agentType\":\"Plan\",\"description\":\"b\",\"toolUseId\":\"tuB\",\"spawnDepth\":2}").unwrap();
    std::fs::write(subs.join("agent-C.jsonl"), line_user("2026-11-17T10:00:00.000Z")).unwrap();
    std::fs::write(subs.join("agent-C.meta.json"), "{\"agentType\":\"general-purpose\",\"description\":\"c\",\"toolUseId\":\"nope\",\"spawnDepth\":1}").unwrap();
    // D: meta missing entirely, role from attributionAgent of first assistant line
    std::fs::write(
        subs.join("agent-D.jsonl"),
        "{\"type\":\"assistant\",\"timestamp\":\"2026-11-17T10:00:00.000Z\",\"attributionAgent\":\"gaisser-agents:software-debugger\",\"message\":{\"id\":\"md\",\"stop_reason\":null,\"content\":[{\"type\":\"text\",\"text\":\"x\"}]}}\n",
    )
    .unwrap();
    let cfg = Config { claude_dir: tmp.path().to_path_buf(), ..Config::default() };
    let mut office = Office::new(cfg, Box::new(Probe::default()));
    let snap = office.initial_scan(now_ms());
    let s = snap.sessions.iter().find(|s| s.id == "nest").unwrap();
    let get = |id: &str| s.agents.iter().find(|a| a.id == id).unwrap_or_else(|| panic!("agent {id} missing"));
    assert_eq!(get("A").parent_id.as_deref(), Some("main"));
    assert_eq!(get("B").parent_id.as_deref(), Some("A"));
    assert_eq!(get("C").parent_id.as_deref(), Some("main"));
    assert_eq!(get("D").role.label, "Debugger", "role should fall back to attributionAgent");
}

#[test]
fn oversized_initial_file_marks_tokens_partial() {
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("projects/C--q");
    std::fs::create_dir_all(&proj).unwrap();
    let mut body = line_user("2026-11-17T09:00:00.000Z");
    for i in 0..200 {
        body.push_str(&tool_use_line(&format!("t{i}"), "Read", "{\"file_path\":\"a.ts\"}"));
        body.push('\n');
    }
    std::fs::write(proj.join("p.jsonl"), body).unwrap();
    let cfg = Config { claude_dir: tmp.path().to_path_buf(), max_initial_read_bytes: 4096, ..Config::default() };
    let mut office = Office::new(cfg, Box::new(Probe::default()));
    let snap = office.initial_scan(now_ms());
    let s = &snap.sessions[0];
    let main = s.agents.iter().find(|a| a.id == "main").unwrap();
    assert!(main.tokens.partial, "tokens.partial must be set when the start was cut");
}

#[test]
fn live_subagent_appears_then_tool_result_flow_via_poll_only() {
    // watcher-less path: poll() alone must discover appended lines and a new subagent
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("projects/C--q");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("pl.jsonl"), line_user("2026-11-17T09:59:00.000Z")).unwrap();
    let cfg = Config { claude_dir: tmp.path().to_path_buf(), ..Config::default() };
    let mut office = Office::new(cfg, Box::new(Probe::default()));
    office.initial_scan(now_ms());
    let subs = proj.join("pl/subagents");
    std::fs::create_dir_all(&subs).unwrap();
    let mut f = std::fs::OpenOptions::new().append(true).open(proj.join("pl.jsonl")).unwrap();
    f.write_all(agent_spawn("tuP", "Explore").as_bytes()).unwrap();
    std::fs::write(subs.join("agent-P.jsonl"), line_user("2026-11-17T10:00:00.000Z")).unwrap();
    std::fs::write(subs.join("agent-P.meta.json"), "{\"agentType\":\"Explore\",\"description\":\"p\",\"toolUseId\":\"tuP\",\"spawnDepth\":1}").unwrap();
    let d = office.poll(now_ms() + 120_000).expect("poll must notice changes");
    assert_eq!(d.spawned.len(), 1);
    assert_eq!(d.spawned[0].agent_id, "P");
    assert_eq!(d.spawned[0].from_agent_id, "main");
}

#[test]
fn parked_idle_registry_session_is_hidden_until_active() {
    // The desktop app relaunches recent sessions in the background: the process is
    // alive and registered as "idle", but nothing has happened for hours.
    let tmp = tempfile::tempdir().unwrap();
    write_session(tmp.path(), "C--q", "parked", Some((11, "idle")));
    write_session(tmp.path(), "C--q", "working", Some((12, "busy")));
    write_session(tmp.path(), "C--q", "fresh", Some((13, "idle")));
    let old = filetime::FileTime::from_unix_time((now_ms() / 1000) - 2 * 3600, 0);
    for sid in ["parked", "working"] {
        filetime::set_file_mtime(tmp.path().join(format!("projects/C--q/{sid}.jsonl")), old).unwrap();
    }
    let probe = Probe::default();
    probe.0.lock().unwrap().extend([11, 12, 13]);
    let cfg = Config { claude_dir: tmp.path().to_path_buf(), ..Config::default() };
    let mut office = Office::new(cfg, Box::new(probe));
    let snap = office.initial_scan(now_ms());
    let ids: HashSet<&str> = snap.sessions.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, HashSet::from(["working", "fresh"]), "alive+idle with an old transcript must be hidden");
}
