mod commands;
mod logging;
mod watcher;
mod worker;

use commands::AppState;
use office_core::Config;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager, WindowEvent};

static MINIMIZED: AtomicBool = AtomicBool::new(false);

/// `%APPDATA%\agent-office\config.json` (the only file the app may write is
/// inside this folder, and v1 never writes it).
fn config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default();
    base.join("agent-office").join("config.json")
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::get_snapshot, commands::set_visible, commands::get_ambient_fps, commands::get_catalog])
        .on_window_event(|window, event| {
            // WebView2 never reports minimize as page-hidden, so detect it here,
            // tell the UI to stop rendering, and let the worker stop emitting.
            if let WindowEvent::Resized(_) = event {
                let min = window.is_minimized().unwrap_or(false);
                if MINIMIZED.swap(min, Ordering::SeqCst) != min {
                    let _ = window.emit("office://minimized", min);
                    if let Some(state) = window.try_state::<AppState>() {
                        let _ = state.tx.send(worker::WorkerMsg::Visible(!min));
                    }
                }
            }
        })
        .setup(|app| {
            let (cfg, warning) = Config::load(&config_path());
            if let Some(w) = warning {
                logging::log(&format!("config: {w}"));
            }
            let (tx, rx) = channel();
            let watcher = match watcher::OfficeWatcher::start(&cfg.claude_dir, tx.clone()) {
                Ok(w) => Some(w),
                Err(e) => {
                    logging::log(&format!("watcher failed: {e}; polling only"));
                    None
                }
            };
            app.manage(AppState { tx, ambient_fps: cfg.ambient_fps });
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("office-worker".into())
                .spawn(move || worker::run(handle, cfg, rx, watcher))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Agent Office");
}
