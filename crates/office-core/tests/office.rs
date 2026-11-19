use office_core::{Activity, Config, Delta, Office, ProcessProbe, SessionSource, SessionStatus};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Default)]
struct Probe(Arc<Mutex<HashSet<u32>>>);
impl ProcessProbe for Probe {
    fn is_alive(&self, pid: u32, _: Option<u64>) -> bool {
        self.0.lock().unwrap().contains(&pid)
    }
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/root")
}

fn copy_dir(from: &Path, to: &Path, skip: &dyn Fn(&Path) -> bool) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if skip(&p) {
            continue;
        }
        let dst = to.join(e.file_name());
        if p.is_dir() {
            copy_dir(&p, &dst, skip);
        } else {
            std::fs::copy(&p, &dst).unwrap();
            filetime::set_file_mtime(&dst, filetime::FileTime::now()).unwrap();
        }
    }
}

fn append(path: &Path, text: &str) {
    let mut f = OpenOptions::new().append(true).open(path).unwrap();
    f.write_all(text.as_bytes()).unwrap();
}

fn cfg(dir: &Path) -> Config {
    Config { claude_dir: dir.to_path_buf(), ..Config::default() }
}

fn result_line(id: &str) -> String {
    format!(
        "{{\"type\":\"user\",\"timestamp\":\"2026-11-17T10:05:00.000Z\",\"message\":{{\"role\":\"user\",\"content\":[{{\"type\":\"tool_result\",\"tool_use_id\":\"{id}\",\"content\":\"ok\"}}]}}}}\n"
    )
}

fn agent_activity(office: &Office, sid: &str, agent: &str) -> Activity {
    let snap = office.snapshot();
    let s = snap.sessions.iter().find(|s| s.id == sid).unwrap();
    s.agents.iter().find(|a| a.id == agent).unwrap().activity
}

#[test]
fn subagent_spawn_and_done() {
    let tmp = tempfile::tempdir().unwrap();
    // hold back a1's files so it "appears" after the initial scan
    copy_dir(&fixtures_root(), tmp.path(), &|p| {
        p.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.starts_with("agent-a1"))
    });
    let mut office = Office::new(cfg(tmp.path()), Box::new(Probe::default()));
    let snap = office.initial_scan(now_ms());
    let sess = snap.sessions.iter().find(|s| s.id == "sess-a").expect("session a");
    assert!(sess.agents.iter().all(|a| a.id != "a1"));

    let sub_dir = tmp.path().join("projects/C--fake-a/sess-a/subagents");
    let src = fixtures_root().join("projects/C--fake-a/sess-a/subagents");
    for n in ["agent-a1.jsonl", "agent-a1.meta.json"] {
        std::fs::copy(src.join(n), sub_dir.join(n)).unwrap();
        filetime::set_file_mtime(sub_dir.join(n), filetime::FileTime::now()).unwrap();
    }
    let delta: Delta = office
        .on_paths_changed(&[sub_dir.join("agent-a1.jsonl"), sub_dir.join("agent-a1.meta.json")], now_ms())
        .expect("delta");
    assert_eq!(delta.spawned.len(), 1);
    assert_eq!(delta.spawned[0].agent_id, "a1");
    assert_eq!(delta.spawned[0].from_agent_id, "main");

    let snap = office.snapshot();
    let a1 = snap.sessions.iter().find(|s| s.id == "sess-a").unwrap().agents.iter().find(|a| a.id == "a1").unwrap();
    assert_eq!(a1.role.label, "Tester");
    assert_eq!(a1.parent_id.as_deref(), Some("main"));
    assert_eq!(a1.description.as_deref(), Some("Run the checks"));
    assert!(a1.activity != Activity::Done);

    // foreground: parent tool_result completes a1
    let main_path = tmp.path().join("projects/C--fake-a/sess-a.jsonl");
    append(&main_path, &result_line("toolu_a1"));
    office.on_paths_changed(&[main_path.clone()], now_ms());
    assert_eq!(agent_activity(&office, "sess-a", "a1"), Activity::Done);

    // background: tool_result does not complete a2
    append(&main_path, &result_line("toolu_a2"));
    office.on_paths_changed(&[main_path.clone()], now_ms());
    assert_ne!(agent_activity(&office, "sess-a", "a2"), Activity::Done);

    // only its own SubagentHandback does
    let a2 = sub_dir.join("agent-a2.jsonl");
    append(
        &a2,
        "{\"type\":\"assistant\",\"timestamp\":\"2026-11-17T10:06:00.000Z\",\"message\":{\"id\":\"msg_h\",\"stop_reason\":\"tool_use\",\"content\":[{\"type\":\"tool_use\",\"id\":\"toolu_h\",\"name\":\"SubagentHandback\",\"input\":{\"message\":\"x\"}}]}}\n",
    );
    office.on_paths_changed(&[a2], now_ms());
    assert_eq!(agent_activity(&office, "sess-a", "a2"), Activity::Done);
}

#[test]
fn key_and_unknown_files_are_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&fixtures_root(), tmp.path(), &|_| false);
    let mut office = Office::new(cfg(tmp.path()), Box::new(Probe::default()));
    office.initial_scan(now_ms());
    let key = tmp.path().join("sessions/4242.0123456789abcdef.key");
    let ignored = tmp.path().join("projects/C--fake-a/sess-a/tool-results/ignored.txt");
    assert!(office.on_paths_changed(&[key, ignored], now_ms()).is_none());
}

#[test]
fn office_dump_fixture_root() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_office-dump"))
        .args(["--claude-dir"])
        .arg(fixtures_root())
        .args(["--once", "--now-ms", "1795000000000"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let sessions = v["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 2);
    let titles: HashSet<&str> = sessions.iter().map(|s| s["title"].as_str().unwrap()).collect();
    assert_eq!(titles, HashSet::from(["Fixture A", "fake-b"]));
}

#[test]
fn config_defaults_and_invalid() {
    let tmp = tempfile::tempdir().unwrap();
    let mut expected = Config::default();
    if let Some(d) = std::env::var_os("AGENT_OFFICE_CLAUDE_DIR") {
        expected.claude_dir = PathBuf::from(d);
    }
    let (c, w) = Config::load(&tmp.path().join("missing.json"));
    assert_eq!(c, expected);
    assert!(w.is_some());

    let bad = tmp.path().join("bad.json");
    std::fs::write(&bad, "{not json").unwrap();
    let (c, w) = Config::load(&bad);
    assert_eq!(c, expected);
    assert!(w.is_some());

    let ok = tmp.path().join("ok.json");
    std::fs::write(&ok, r#"{"activeWindowMinutes":5}"#).unwrap();
    let (c, w) = Config::load(&ok);
    assert!(w.is_none());
    assert_eq!(c.active_window_minutes, 5);
    let mut rest = c.clone();
    rest.active_window_minutes = expected.active_window_minutes;
    assert_eq!(rest, expected);
    assert!(expected.use_session_registry);
}

#[test]
fn session_lifecycle() {
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("projects/C--l");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::create_dir_all(tmp.path().join("sessions")).unwrap();
    let body = "{\"type\":\"user\",\"timestamp\":\"2026-11-17T10:00:00.000Z\",\"cwd\":\"C:\\\\l\",\"message\":{\"role\":\"user\",\"content\":\"hi\"}}\n";
    std::fs::write(proj.join("s1.jsonl"), body).unwrap();
    std::fs::write(proj.join("s2.jsonl"), body).unwrap();
    std::fs::write(
        tmp.path().join("sessions/100.json"),
        r#"{"pid":100,"sessionId":"s1","cwd":"C:\\l","name":"Reg Name","status":"idle","procStart":"5"}"#,
    )
    .unwrap();
    let probe = Probe::default();
    probe.0.lock().unwrap().insert(100);
    let mut office = Office::new(cfg(tmp.path()), Box::new(probe.clone()));
    let now = now_ms();
    let snap = office.initial_scan(now);
    assert_eq!(snap.sessions.len(), 2);
    let s1 = snap.sessions.iter().find(|s| s.id == "s1").unwrap();
    assert_eq!(s1.title, "Reg Name");
    assert_eq!(s1.source, SessionSource::Registry);

    // pid dies -> closed
    probe.0.lock().unwrap().clear();
    let d = office.tick(now + 1000).expect("delta");
    let closed = d.sessions_upserted.iter().find(|s| s.id == "s1").unwrap();
    assert_eq!(closed.status, SessionStatus::Closed);

    // not yet past the grace period
    assert!(office.tick(now + 29_000).map_or(true, |d| d.sessions_removed.is_empty()));
    // past closedGraceSeconds (30) -> removed
    let d = office.tick(now + 32_000).expect("delta");
    assert_eq!(d.sessions_removed, vec!["s1".to_string()]);

    // non-registry session: removed once its newest mtime is older than the window
    let later = now + 31 * 60_000;
    let d = office.tick(later).expect("delta");
    assert_eq!(d.sessions_removed, vec!["s2".to_string()]);
    assert!(office.snapshot().sessions.is_empty());
}

#[test]
fn missing_claude_dir_reports_error() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("nope");
    let mut office = Office::new(cfg(&missing), Box::new(Probe::default()));
    let snap = office.initial_scan(now_ms());
    assert!(snap.sessions.is_empty());
    assert_eq!(snap.error, Some(format!("claude dir not found: {}", missing.display())));
}

#[test]
fn vanished_files_are_dropped_on_poll() {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&fixtures_root(), tmp.path(), &|_| false);
    let mut office = Office::new(cfg(tmp.path()), Box::new(Probe::default()));
    office.initial_scan(now_ms());
    std::fs::remove_file(tmp.path().join("projects/C--fake-a/sess-a/subagents/agent-a2.jsonl")).unwrap();
    let d = office.poll(now_ms()).expect("delta");
    assert!(d.agents_removed.iter().any(|(s, a)| s == "sess-a" && a == "a2"));
    std::fs::remove_file(tmp.path().join("projects/C--fake-b/sess-b.jsonl")).unwrap();
    let d = office.poll(now_ms()).expect("delta");
    assert_eq!(d.sessions_removed, vec!["sess-b".to_string()]);
}
