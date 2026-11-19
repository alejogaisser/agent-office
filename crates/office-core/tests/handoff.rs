use office_core::{Config, Delta, Handoff, HandoffKind, Office, ProcessProbe};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct NoProbe;
impl ProcessProbe for NoProbe {
    fn is_alive(&self, _: u32, _: Option<u64>) -> bool {
        false
    }
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/handoff").join(name)
}

fn copy_dir(from: &Path, to: &Path, skip: &dyn Fn(&str) -> bool) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if skip(&name) {
            continue;
        }
        let dst = to.join(&name);
        if p.is_dir() {
            copy_dir(&p, &dst, skip);
        } else {
            std::fs::copy(&p, &dst).unwrap();
            filetime::set_file_mtime(&dst, filetime::FileTime::now()).unwrap();
        }
    }
}

fn office_for(dir: &Path) -> Office {
    let cfg = Config { claude_dir: dir.to_path_buf(), ..Config::default() };
    Office::new(cfg, Box::new(NoProbe))
}

fn subs_dir(root: &Path) -> PathBuf {
    root.join("projects/C--fake-h/sess-h/subagents")
}

/// Copies one held-back subagent into the tree and reports it to the office.
fn add_agent(office: &mut Office, root: &Path, id: &str) -> Delta {
    let src = subs_dir(&fixture("relay"));
    let src = if src.join(format!("agent-{id}.jsonl")).exists() { src } else { subs_dir(&fixture("basic")) };
    let dir = subs_dir(root);
    let mut paths = Vec::new();
    for n in [format!("agent-{id}.jsonl"), format!("agent-{id}.meta.json")] {
        std::fs::copy(src.join(&n), dir.join(&n)).unwrap();
        filetime::set_file_mtime(dir.join(&n), filetime::FileTime::now()).unwrap();
        paths.push(dir.join(&n));
    }
    office.on_paths_changed(&paths, now_ms()).unwrap_or_default()
}

fn handback(office: &mut Office, root: &Path, id: &str, ts: &str) -> Delta {
    let p = subs_dir(root).join(format!("agent-{id}.jsonl"));
    let line = format!(
        "{{\"type\":\"assistant\",\"timestamp\":\"{ts}\",\"message\":{{\"id\":\"msg_h{id}\",\"stop_reason\":\"tool_use\",\"content\":[{{\"type\":\"tool_use\",\"id\":\"toolu_h{id}\",\"name\":\"SubagentHandback\",\"input\":{{\"message\":\"x\"}}}}]}}}}\n"
    );
    OpenOptions::new().append(true).open(&p).unwrap().write_all(line.as_bytes()).unwrap();
    office.on_paths_changed(&[p], now_ms()).unwrap_or_default()
}

fn held_back(keep: &'static [&'static str]) -> impl Fn(&str) -> bool {
    move |n: &str| {
        n.starts_with("agent-") && !keep.iter().any(|k| n.starts_with(&format!("agent-{k}.")))
    }
}

fn kinds(d: &Delta, kind: HandoffKind) -> Vec<&Handoff> {
    d.handoffs.iter().filter(|h| h.kind == kind).collect()
}

#[test]
fn handoff_spawn_and_return() {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&fixture("basic"), tmp.path(), &held_back(&[]));
    let mut office = office_for(tmp.path());
    office.initial_scan(now_ms());

    let d = add_agent(&mut office, tmp.path(), "a1");
    assert_eq!(d.spawned.len(), 1);
    assert_eq!(d.spawned[0].agent_id, "a1");
    assert_eq!(d.handoffs.len(), 1, "{:?}", d.handoffs);
    let h = &d.handoffs[0];
    assert_eq!(h.kind, HandoffKind::Spawn);
    assert_eq!((h.from_agent_id.as_str(), h.to_agent_id.as_str()), ("main", "a1"));
    assert_eq!((h.from_team.as_deref(), h.to_team.as_deref()), (None, Some("software")));
    assert_eq!(h.relay_from_agent_id, None);
    let json = serde_json::to_value(h).unwrap();
    assert_eq!(json["kind"], "spawn");
    assert!(json["relayFromAgentId"].is_null());
    assert!(json["fromTeam"].is_null());

    let d = handback(&mut office, tmp.path(), "a1", "2026-11-17T10:00:30.000Z");
    let rets = kinds(&d, HandoffKind::Return);
    assert_eq!(rets.len(), 1);
    assert_eq!((rets[0].from_agent_id.as_str(), rets[0].to_agent_id.as_str()), ("a1", "main"));
    assert_eq!((rets[0].from_team.as_deref(), rets[0].to_team.as_deref()), (Some("software"), None));
    assert!(kinds(&d, HandoffKind::Spawn).is_empty());
}

#[test]
fn initial_scan_of_finished_session_emits_no_handoffs() {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&fixture("basic"), tmp.path(), &|_| false);
    let p = subs_dir(tmp.path()).join("agent-a1.jsonl");
    let line = "{\"type\":\"assistant\",\"timestamp\":\"2026-11-17T10:00:30.000Z\",\"message\":{\"id\":\"msg_hh\",\"stop_reason\":\"tool_use\",\"content\":[{\"type\":\"tool_use\",\"id\":\"toolu_hh\",\"name\":\"SubagentHandback\",\"input\":{\"message\":\"x\"}}]}}\n";
    OpenOptions::new().append(true).open(&p).unwrap().write_all(line.as_bytes()).unwrap();
    let mut office = office_for(tmp.path());
    let snap = office.initial_scan(now_ms());
    assert_eq!(snap.sessions.len(), 1);
    // the first diff after the scan is silent: no spawn and no handoff for existing agents
    let d = office.tick(now_ms() + 1000).unwrap_or_default();
    assert!(d.handoffs.is_empty() && d.spawned.is_empty(), "{d:?}");
}

#[test]
fn handoff_relay() {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&fixture("relay"), tmp.path(), &held_back(&[]));
    let mut office = office_for(tmp.path());
    office.initial_scan(now_ms());
    add_agent(&mut office, tmp.path(), "p1");
    let d = handback(&mut office, tmp.path(), "p1", "2026-11-17T10:00:30.000Z");
    assert_eq!(kinds(&d, HandoffKind::Return).len(), 1);

    // t+60 s
    let d = add_agent(&mut office, tmp.path(), "s1");
    let h = kinds(&d, HandoffKind::Spawn);
    assert_eq!(h.len(), 1, "{:?}", d.handoffs);
    assert_eq!(h[0].relay_from_agent_id.as_deref(), Some("p1"));
    assert_eq!(h[0].from_team.as_deref(), None);
    assert_eq!(h[0].to_team.as_deref(), Some("software"));

    // t+700 s: outside the 600 s window
    let tmp2 = tempfile::tempdir().unwrap();
    copy_dir(&fixture("relay"), tmp2.path(), &held_back(&[]));
    let mut office = office_for(tmp2.path());
    office.initial_scan(now_ms());
    add_agent(&mut office, tmp2.path(), "p1");
    handback(&mut office, tmp2.path(), "p1", "2026-11-17T10:00:30.000Z");
    let d = add_agent(&mut office, tmp2.path(), "s2");
    let h = kinds(&d, HandoffKind::Spawn);
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].relay_from_agent_id, None);

    // software -> software is never a relay
    let tmp3 = tempfile::tempdir().unwrap();
    copy_dir(&fixture("relay"), tmp3.path(), &held_back(&[]));
    let mut office = office_for(tmp3.path());
    office.initial_scan(now_ms());
    add_agent(&mut office, tmp3.path(), "q1");
    let d = handback(&mut office, tmp3.path(), "q1", "2026-11-17T10:00:20.000Z");
    assert_eq!(kinds(&d, HandoffKind::Return).len(), 1);
    let d = add_agent(&mut office, tmp3.path(), "s3");
    let h = kinds(&d, HandoffKind::Spawn);
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].relay_from_agent_id, None);
}

#[test]
fn active_teams() {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&fixture("relay"), tmp.path(), &held_back(&["q1", "c1"]));
    let mut office = office_for(tmp.path());
    let snap = office.initial_scan(now_ms());
    assert_eq!(snap.sessions[0].active_teams, vec!["software", "content"]);

    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&fixture("basic"), tmp.path(), &held_back(&[]));
    let mut office = office_for(tmp.path());
    let snap = office.initial_scan(now_ms());
    assert_eq!(snap.sessions[0].active_teams, vec!["general"]);
}
