//! Prints the office state as JSON. `--once` prints one Snapshot; `--follow`
//! then streams each Delta as NDJSON (renderer-agnostic contract, ADR 0003).
use office_core::{Config, Office, SystemProbe};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn real_now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn main() {
    let mut claude_dir: Option<PathBuf> = None;
    let mut once = false;
    let mut catalog = false;
    let mut follow = false;
    let mut fixed_now: Option<i64> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--claude-dir" => claude_dir = args.next().map(PathBuf::from),
            "--once" => once = true,
            "--follow" => follow = true,
            "--catalog" => catalog = true,
            "--now-ms" => fixed_now = args.next().and_then(|v| v.parse().ok()),
            "-h" | "--help" => {
                eprintln!("office-dump [--claude-dir DIR] [--once] [--follow] [--now-ms N] [--catalog]");
                return;
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }
    if catalog {
        println!("{}", serde_json::to_string(&office_core::roles::catalog()).unwrap_or_default());
        return;
    }
    let mut cfg = Config::default();
    if let Some(d) = std::env::var_os(office_core::config::ENV_CLAUDE_DIR) {
        cfg.claude_dir = PathBuf::from(d);
    }
    if let Some(d) = claude_dir {
        cfg.claude_dir = d;
    }
    if fixed_now.is_some() {
        // With a fixed clock, file mtimes are meaningless: disable the active window.
        cfg.active_window_minutes = 60 * 24 * 365 * 100;
    }
    let now = || fixed_now.unwrap_or_else(real_now);
    let mut office = Office::new(cfg, Box::new(SystemProbe));
    let snap = office.initial_scan(now());
    let out = std::io::stdout();
    if once || !follow {
        println!("{}", serde_json::to_string(&snap).unwrap_or_default());
        if !follow {
            return;
        }
    }
    if follow {
        loop {
            std::thread::sleep(Duration::from_millis(500));
            if let Some(d) = office.poll(now()) {
                let mut o = out.lock();
                let _ = writeln!(o, "{}", serde_json::to_string(&d).unwrap_or_default());
                let _ = o.flush();
            }
        }
    }
}
