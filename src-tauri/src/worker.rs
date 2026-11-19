//! One thread owns the Office. It blocks on `recv_timeout` until the nearest
//! deadline, so there is no fixed tick. Emits are coalesced to one per 100 ms.
use crate::logging::log;
use crate::watcher::OfficeWatcher;
use office_core::{Config, Delta, Office, Snapshot, SystemProbe};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

const DEBOUNCE_MS: i64 = 100;
const IDLE_POLL_MS: i64 = 15_000;

pub enum WorkerMsg {
    Paths(Vec<PathBuf>),
    Visible(bool),
    Snapshot(Sender<Snapshot>),
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn count(d: &Delta) -> usize {
    d.sessions_upserted.len()
        + d.sessions_removed.len()
        + d.agents_upserted.len()
        + d.agents_removed.len()
        + d.spawned.len()
}

pub fn run(app: AppHandle, cfg: Config, rx: Receiver<WorkerMsg>, mut watcher: Option<OfficeWatcher>) {
    let poll_working = cfg.poll_fallback_ms.max(250) as i64;
    let mut office = Office::new(cfg, Box::new(SystemProbe));
    let snap = office.initial_scan(now_ms());
    log(&format!("initial scan sessions={}", snap.sessions.len()));
    let mut visible = true;
    let mut dirty = false;
    let mut pending: HashSet<PathBuf> = HashSet::new();
    let mut first_pending: i64 = 0;
    let mut next_poll = now_ms() + poll_working;

    let emit = |d: Option<Delta>, visible: bool, dirty: &mut bool| {
        let Some(d) = d else { return };
        if !visible {
            *dirty = true;
            return;
        }
        log(&format!("emit delta n={}", count(&d)));
        let _ = app.emit("office://delta", &d);
    };

    loop {
        let now = now_ms();
        let mut wake = next_poll;
        if let Some(dl) = office.next_deadline_ms() {
            wake = wake.min(dl);
        }
        if !pending.is_empty() {
            wake = wake.min(first_pending + DEBOUNCE_MS);
        }
        let timeout = Duration::from_millis((wake - now).clamp(1, IDLE_POLL_MS) as u64);
        match rx.recv_timeout(timeout) {
            Ok(WorkerMsg::Paths(p)) => {
                // projects/ or sessions/ may have just been created: watch them and rescan.
                if let Some(w) = watcher.as_mut().filter(|w| !w.complete()) {
                    if w.ensure() {
                        office.request_rescan();
                        next_poll = now_ms();
                    }
                }
                if pending.is_empty() {
                    first_pending = now_ms();
                }
                pending.extend(p);
            }
            Ok(WorkerMsg::Visible(v)) => {
                visible = v;
                if v {
                    dirty = false;
                }
            }
            Ok(WorkerMsg::Snapshot(reply)) => {
                let _ = reply.send(office.snapshot());
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        let now = now_ms();
        if !pending.is_empty() && now - first_pending >= DEBOUNCE_MS {
            let paths: Vec<PathBuf> = pending.drain().collect();
            emit(office.on_paths_changed(&paths, now), visible, &mut dirty);
        }
        if now >= next_poll {
            let interval = if visible && office.any_working() { poll_working } else { IDLE_POLL_MS };
            next_poll = now + interval;
            emit(office.poll(now), visible, &mut dirty);
        } else if office.next_deadline_ms().map_or(false, |dl| now >= dl) {
            emit(office.tick(now), visible, &mut dirty);
        }
    }
}
