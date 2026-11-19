use office_core::line::{parse_into, parse_line, parse_ts, LineEvent, ParseStats};
use office_core::reducer::{AgentTracker, PENDING_CAP, RECENT_CAP};
use office_core::Activity;
use std::path::PathBuf;

fn fixture(name: &str) -> Vec<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    std::fs::read(&p)
        .unwrap()
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| l.to_vec())
        .collect()
}

const NOW: i64 = 1_795_000_000_000;

#[test]
fn tolerates_unknown_and_malformed() {
    let lines = fixture("unknown_and_malformed.jsonl");
    assert_eq!(lines.len(), 6);
    let mut stats = ParseStats::default();
    let with_events = lines
        .iter()
        .map(|l| parse_into(l, &mut stats))
        .filter(|ev| !ev.is_empty())
        .count();
    assert_eq!(with_events, 3);
    assert_eq!(stats.malformed, 1);
    // unknown type is Ok(empty), not an error
    assert!(parse_line(&lines[1]).unwrap().is_empty());
    assert!(parse_line(&lines[2]).is_err());
}

#[test]
fn activity_transitions() {
    let lines = fixture("main_basic.jsonl");
    let mut t = AgentTracker::new("main", true, NOW);
    let mut after = Vec::new();
    for l in &lines {
        for ev in parse_line(l).unwrap() {
            t.apply(&ev, NOW);
        }
        after.push((t.activity, t.target.clone()));
    }
    assert_eq!(after[1].0, Activity::Writing);
    assert_eq!(after[1].1.as_deref(), Some("src/app.ts"));
    assert_eq!(after[2].0, Activity::Thinking);
    assert_eq!(after[3].0, Activity::Running);
    assert!(after[3].1.as_deref().unwrap().starts_with("npm test"));
    assert_eq!(after[4].0, Activity::Thinking);
    assert_eq!(after[5].0, Activity::Waiting);
    assert_eq!(after[6].0, Activity::Waiting);
}

#[test]
fn tokens_dedup_by_message_id() {
    let mut t = AgentTracker::new("main", true, NOW);
    for l in fixture("tokens_dup.jsonl") {
        for ev in parse_line(&l).unwrap() {
            t.apply(&ev, NOW);
        }
    }
    assert_eq!(t.tokens.output, 12);
    assert_eq!(t.tokens.input, 30);
    // last message: input 20 + cache_read 200 + cache_creation 30
    assert_eq!(t.tokens.context, 250);
}

#[test]
fn long_transcript_bounded() {
    let mut t = AgentTracker::new("main", true, NOW);
    let long_cmd = "x".repeat(500);
    let mut lines = 0usize;
    let mut i = 0usize;
    while lines < 200_000 {
        let id = format!("toolu_{i}");
        let tool_use = format!(
            r#"{{"type":"assistant","timestamp":"2026-11-17T10:00:00.000Z","message":{{"id":"m{i}","stop_reason":"tool_use","content":[{{"type":"tool_use","id":"{id}","name":"Bash","input":{{"command":"{long_cmd}"}}}}]}}}}"#
        );
        for ev in parse_line(tool_use.as_bytes()).unwrap() {
            t.apply(&ev, NOW);
        }
        lines += 1;
        if i % 10 != 0 {
            let res = format!(
                r#"{{"type":"user","timestamp":"2026-11-17T10:00:01.000Z","message":{{"role":"user","content":[{{"type":"tool_result","tool_use_id":"{id}","content":"ok"}}]}}}}"#
            );
            for ev in parse_line(res.as_bytes()).unwrap() {
                t.apply(&ev, NOW);
            }
            lines += 1;
        }
        i += 1;
    }
    assert!(t.recent.len() <= RECENT_CAP);
    assert!(t.pending.len() <= PENDING_CAP);
    assert!(!t.pending.is_empty());
    let max = 200;
    for p in &t.pending {
        assert!(p.id.chars().count() <= max && p.name.chars().count() <= max);
        assert!(p.target.as_deref().map_or(0, |s| s.chars().count()) <= max);
    }
    for r in &t.recent {
        assert!(r.tool.chars().count() <= max);
        assert!(r.target.as_deref().map_or(0, |s| s.chars().count()) <= max);
    }
    assert!(t.target.as_deref().map_or(0, |s| s.chars().count()) <= max);
    assert!(t.tool_name.as_deref().map_or(0, |s| s.chars().count()) <= max);
}

#[test]
fn targets_and_events() {
    let l = br#"{"type":"assistant","timestamp":"2026-11-17T10:00:00.000Z","message":{"id":"m","content":[{"type":"tool_use","id":"t","name":"Grep","input":{"pattern":"foo","path":"src"}},{"type":"tool_use","id":"u","name":"Agent","input":{"subagent_type":"coder","description":"d","run_in_background":true}}]}}"#;
    let ev = parse_line(l).unwrap();
    let grep = ev.iter().find_map(|e| match e {
        LineEvent::ToolUse { name, target, .. } if name == "Grep" => target.clone(),
        _ => None,
    });
    assert_eq!(grep.as_deref(), Some("foo in src"));
    let spawn = ev.iter().find_map(|e| match e {
        LineEvent::ToolUse { spawn, .. } => spawn.clone(),
        _ => None,
    });
    let spawn = spawn.unwrap();
    assert!(spawn.background);
    assert_eq!(spawn.subagent_type, "coder");
}

#[test]
fn timestamp_parsing() {
    assert_eq!(parse_ts("1970-01-01T00:00:00.000Z"), 0);
    assert_eq!(parse_ts("2026-11-17T10:00:00.250Z"), 1_794_909_600_250);
}
