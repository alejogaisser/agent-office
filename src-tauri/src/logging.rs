//! Debug log, active only with AGENT_OFFICE_LOG=debug. Writes to stderr and to
//! `%LOCALAPPDATA%\agent-office\agent-office.log`.
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static ENABLED: OnceLock<bool> = OnceLock::new();
static FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| std::env::var("AGENT_OFFICE_LOG").map(|v| v == "debug").unwrap_or(false))
}

/// UTC timestamp with ms, e.g. 2026-11-18T12:34:56.789Z (same shape as the simulator).
pub fn iso_now() -> String {
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
    let secs = ms.div_euclid(1000);
    let milli = ms.rem_euclid(1000);
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{milli:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

pub fn log(msg: &str) {
    if !enabled() {
        return;
    }
    let line = format!("{} {msg}\n", iso_now());
    eprint!("{line}");
    let slot = FILE.get_or_init(|| {
        let f = std::env::var_os("LOCALAPPDATA").and_then(|base| {
            let dir = std::path::PathBuf::from(base).join("agent-office");
            std::fs::create_dir_all(&dir).ok()?;
            OpenOptions::new().create(true).append(true).open(dir.join("agent-office.log")).ok()
        });
        Mutex::new(f)
    });
    if let Ok(mut g) = slot.lock() {
        if let Some(f) = g.as_mut() {
            let _ = f.write_all(line.as_bytes());
        }
    }
}
