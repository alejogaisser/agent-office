use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const ENV_CLAUDE_DIR: &str = "AGENT_OFFICE_CLAUDE_DIR";

fn default_claude_dir() -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".claude")
}

fn d_claude_dir() -> PathBuf {
    default_claude_dir()
}
fn d_active() -> u32 {
    30
}
fn d_sleep() -> u32 {
    300
}
fn d_grace() -> u32 {
    30
}
fn d_done() -> u32 {
    8
}
fn d_fps() -> u32 {
    2
}
fn d_poll() -> u32 {
    3000
}
fn d_initial() -> u64 {
    8_388_608
}
fn d_relay() -> u32 {
    600
}
fn d_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "d_claude_dir")]
    pub claude_dir: PathBuf,
    #[serde(default = "d_active")]
    pub active_window_minutes: u32,
    #[serde(default = "d_sleep")]
    pub sleep_after_seconds: u32,
    #[serde(default = "d_grace")]
    pub closed_grace_seconds: u32,
    #[serde(default = "d_done")]
    pub max_done_shown: u32,
    #[serde(default = "d_fps")]
    pub ambient_fps: u32,
    #[serde(default = "d_poll")]
    pub poll_fallback_ms: u32,
    #[serde(default = "d_initial")]
    pub max_initial_read_bytes: u64,
    #[serde(default = "d_true")]
    pub use_session_registry: bool,
    #[serde(default = "d_relay")]
    pub relay_window_seconds: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            claude_dir: d_claude_dir(),
            active_window_minutes: d_active(),
            sleep_after_seconds: d_sleep(),
            closed_grace_seconds: d_grace(),
            max_done_shown: d_done(),
            ambient_fps: d_fps(),
            poll_fallback_ms: d_poll(),
            max_initial_read_bytes: d_initial(),
            use_session_registry: true,
            relay_window_seconds: d_relay(),
        }
    }
}

impl Config {
    /// Loads the config file. Missing or invalid -> defaults plus one warning.
    /// `claude_dir` precedence: env AGENT_OFFICE_CLAUDE_DIR > config > default.
    pub fn load(path: &Path) -> (Config, Option<String>) {
        let (mut cfg, warning) = match std::fs::read(path) {
            Ok(bytes) => match serde_json::from_slice::<Config>(&bytes) {
                Ok(c) => (c, None),
                Err(e) => (
                    Config::default(),
                    Some(format!("invalid config {}: {e}; using defaults", path.display())),
                ),
            },
            Err(e) => (
                Config::default(),
                Some(format!("cannot read config {}: {e}; using defaults", path.display())),
            ),
        };
        if let Some(dir) = std::env::var_os(ENV_CLAUDE_DIR) {
            if !dir.is_empty() {
                cfg.claude_dir = PathBuf::from(dir);
            }
        }
        (cfg, warning)
    }
}
