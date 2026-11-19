use crate::worker::WorkerMsg;
use office_core::{Catalog, Snapshot};
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use tauri::State;

pub struct AppState {
    pub tx: Sender<WorkerMsg>,
    pub ambient_fps: u32,
}

#[tauri::command]
pub async fn get_snapshot(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let (tx, rx) = channel();
    state.tx.send(WorkerMsg::Snapshot(tx)).map_err(|e| e.to_string())?;
    rx.recv_timeout(Duration::from_secs(30)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_visible(state: State<'_, AppState>, visible: bool) {
    let _ = state.tx.send(WorkerMsg::Visible(visible));
}

#[tauri::command]
pub fn get_ambient_fps(state: State<'_, AppState>) -> u32 {
    state.ambient_fps
}

#[tauri::command]
pub fn get_catalog() -> Catalog {
    office_core::roles::catalog()
}
