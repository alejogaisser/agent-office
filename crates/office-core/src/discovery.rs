//! Finding sessions on disk: transcripts, subagent files, and the registry.
//! Never opens `*.key` files: only names matching `^\d+\.json$` are read.
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry {
    pub pid: u32,
    pub session_id: String,
    pub cwd: String,
    pub name: Option<String>,
    pub status: Option<String>,
    pub started_at: i64,
    pub updated_at: i64,
    pub proc_start: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SubFiles {
    pub agent_id: String,
    pub path: PathBuf,
    pub meta_path: Option<PathBuf>,
    pub mtime_ms: i64,
}

#[derive(Debug, Clone)]
pub struct SessionFiles {
    pub session_id: String,
    pub slug: String,
    pub main_path: PathBuf,
    pub mtime_ms: i64,
    pub subagents: Vec<SubFiles>,
}

/// `^\d+\.json$`
pub fn is_registry_file(name: &str) -> bool {
    match name.strip_suffix(".json") {
        Some(stem) => !stem.is_empty() && stem.bytes().all(|b| b.is_ascii_digit()),
        None => false,
    }
}

/// cwd with every char outside `[A-Za-z0-9]` replaced by `-`.
pub fn slug_for(cwd: &str) -> String {
    cwd.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Basename of cwd, or the slug without its drive prefix when cwd is unknown.
pub fn project_name(cwd: &str, slug: &str) -> String {
    let base = cwd
        .trim_end_matches(['\\', '/'])
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or("");
    if !base.is_empty() {
        return base.to_string();
    }
    let b = slug.as_bytes();
    if b.len() > 3 && b[0].is_ascii_alphabetic() && b[1] == b'-' && b[2] == b'-' {
        return slug[3..].to_string();
    }
    slug.to_string()
}

pub fn mtime_ms(path: &Path) -> Option<i64> {
    let m = std::fs::metadata(path).ok()?.modified().ok()?;
    let d = m.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(d.as_millis() as i64)
}

fn num(v: &Option<Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) => n.as_i64(),
        Some(Value::String(s)) => s.parse().ok(),
        _ => None,
    }
}

#[derive(Deserialize)]
struct RawReg {
    pid: Option<Value>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    cwd: Option<String>,
    name: Option<String>,
    status: Option<String>,
    #[serde(rename = "startedAt")]
    started_at: Option<Value>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<Value>,
    #[serde(rename = "procStart")]
    proc_start: Option<Value>,
}

pub fn parse_registry_file(path: &Path) -> Option<RegistryEntry> {
    let bytes = std::fs::read(path).ok()?;
    let r: RawReg = serde_json::from_slice(&bytes).ok()?;
    let pid = num(&r.pid)? as u32;
    Some(RegistryEntry {
        pid,
        session_id: r.session_id?,
        cwd: r.cwd.unwrap_or_default(),
        name: r.name.filter(|n| !n.trim().is_empty()),
        status: r.status,
        started_at: num(&r.started_at).unwrap_or(0),
        updated_at: num(&r.updated_at).unwrap_or(0),
        proc_start: num(&r.proc_start).map(|v| v as u64),
    })
}

/// Reads every `<pid>.json` in the sessions dir, skipping bad files.
pub fn read_registry(dir: &Path) -> Vec<RegistryEntry> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return out };
    for e in rd.flatten() {
        let name = e.file_name();
        let Some(name) = name.to_str() else { continue };
        if !is_registry_file(name) {
            continue;
        }
        if let Some(r) = parse_registry_file(&e.path()) {
            out.push(r);
        }
    }
    out
}

/// Lists `<session_dir>/subagents/agent-<id>.jsonl` (+ `.meta.json`).
pub fn list_subagents(session_dir: &Path) -> Vec<SubFiles> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(session_dir.join("subagents")) else { return out };
    for e in rd.flatten() {
        let name = e.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(id) = name.strip_prefix("agent-").and_then(|n| n.strip_suffix(".jsonl")) else {
            continue;
        };
        let path = e.path();
        let meta = session_dir.join("subagents").join(format!("agent-{id}.meta.json"));
        out.push(SubFiles {
            agent_id: id.to_string(),
            mtime_ms: mtime_ms(&path).unwrap_or(0),
            meta_path: if meta.is_file() { Some(meta) } else { None },
            path,
        });
    }
    out.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));
    out
}

/// Stats `*.jsonl` one level deep under each project; lists subagents only for
/// sessions whose main transcript is newer than `cutoff_ms`.
pub fn scan_recent(projects_dir: &Path, cutoff_ms: i64) -> Vec<SessionFiles> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(projects_dir) else { return out };
    for proj in rd.flatten() {
        if !proj.path().is_dir() {
            continue;
        }
        let slug = proj.file_name().to_string_lossy().to_string();
        let Ok(files) = std::fs::read_dir(proj.path()) else { continue };
        for f in files.flatten() {
            let name = f.file_name();
            let Some(name) = name.to_str() else { continue };
            let Some(id) = name.strip_suffix(".jsonl") else { continue };
            let path = f.path();
            let Some(m) = mtime_ms(&path) else { continue };
            if m < cutoff_ms {
                continue;
            }
            out.push(SessionFiles {
                session_id: id.to_string(),
                slug: slug.clone(),
                subagents: list_subagents(&proj.path().join(id)),
                main_path: path,
                mtime_ms: m,
            });
        }
    }
    out
}

/// Locates a session transcript: slug first, then `projects/*/<id>.jsonl`.
pub fn find_transcript(projects_dir: &Path, cwd: &str, session_id: &str) -> Option<SessionFiles> {
    let file = format!("{session_id}.jsonl");
    let mut candidates = Vec::new();
    if !cwd.is_empty() {
        let slug = slug_for(cwd);
        candidates.push((slug.clone(), projects_dir.join(&slug).join(&file)));
    }
    let found = candidates.into_iter().find(|(_, p)| p.is_file()).or_else(|| {
        let rd = std::fs::read_dir(projects_dir).ok()?;
        for d in rd.flatten() {
            let p = d.path().join(&file);
            if p.is_file() {
                return Some((d.file_name().to_string_lossy().to_string(), p));
            }
        }
        None
    });
    let (slug, main_path) = found?;
    let session_dir = main_path.parent()?.join(session_id);
    Some(SessionFiles {
        session_id: session_id.to_string(),
        slug,
        mtime_ms: mtime_ms(&main_path).unwrap_or(0),
        subagents: list_subagents(&session_dir),
        main_path,
    })
}
