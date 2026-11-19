use filetime::{set_file_mtime, FileTime};
use office_core::discovery::{is_registry_file, read_registry, slug_for};
use office_core::{Config, Office, ProcessProbe};
use std::time::{SystemTime, UNIX_EPOCH};

struct AllDead;
impl ProcessProbe for AllDead {
    fn is_alive(&self, _: u32, _: Option<u64>) -> bool {
        false
    }
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

fn line() -> &'static str {
    "{\"type\":\"user\",\"timestamp\":\"2026-11-17T10:00:00.000Z\",\"message\":{\"role\":\"user\",\"content\":\"hi\"}}\n"
}

#[test]
fn discovery_active_window() {
    let dir = tempfile::tempdir().unwrap();
    let proj = dir.path().join("projects").join("C--t");
    std::fs::create_dir_all(&proj).unwrap();
    let now = now_ms();
    for (name, age_min) in [("s1", 2), ("s2", 10), ("s3", 120)] {
        let p = proj.join(format!("{name}.jsonl"));
        std::fs::write(&p, line()).unwrap();
        let t = (now / 1000) - age_min * 60;
        set_file_mtime(&p, FileTime::from_unix_time(t, 0)).unwrap();
    }
    let cfg = Config {
        claude_dir: dir.path().to_path_buf(),
        active_window_minutes: 30,
        ..Config::default()
    };
    let mut office = Office::new(cfg, Box::new(AllDead));
    let snap = office.initial_scan(now);
    assert_eq!(snap.sessions.len(), 2);
    assert!(snap.error.is_none());

    assert!(is_registry_file("123.json"));
    assert!(!is_registry_file("123.abc.key"));
    assert!(!is_registry_file("x.json"));
    assert!(!is_registry_file(".json"));
}

#[test]
fn registry_skips_keys_and_bad_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("7.json"),
        r#"{"pid":7,"sessionId":"s","cwd":"C:\\x","procStart":"123","name":"N","status":"busy"}"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("8.json"), "not json").unwrap();
    std::fs::write(dir.path().join("7.deadbeef.key"), "SECRET").unwrap();
    let r = read_registry(dir.path());
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].pid, 7);
    assert_eq!(r[0].proc_start, Some(123));
}

#[test]
fn slug_rule() {
    assert_eq!(
        slug_for(r"C:\Users\ASUS\OneDrive\Desktop\Content creator"),
        "C--Users-ASUS-OneDrive-Desktop-Content-creator"
    );
}
