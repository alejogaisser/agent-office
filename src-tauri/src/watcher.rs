//! Thin wrapper over `notify`: forwards changed paths to the worker.
use crate::worker::WorkerMsg;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

/// Watches `projects/` recursively and `sessions/` non-recursively. While one of
/// them does not exist yet, the claude dir itself is watched (non-recursively)
/// so their creation is noticed; `ensure` then adds the missing watches.
pub struct OfficeWatcher {
    w: RecommendedWatcher,
    root: PathBuf,
    projects: bool,
    sessions: bool,
    root_watched: bool,
}

impl OfficeWatcher {
    pub fn start(claude_dir: &Path, tx: Sender<WorkerMsg>) -> notify::Result<OfficeWatcher> {
        let w = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(ev) = res {
                if matches!(ev.kind, EventKind::Access(_)) {
                    return;
                }
                let _ = tx.send(WorkerMsg::Paths(ev.paths));
            }
        })?;
        let mut me = OfficeWatcher {
            w,
            root: claude_dir.to_path_buf(),
            projects: false,
            sessions: false,
            root_watched: false,
        };
        me.ensure();
        Ok(me)
    }

    /// True when every watch is in place.
    pub fn complete(&self) -> bool {
        self.projects && self.sessions
    }

    /// Adds watches for directories that appeared since the last call.
    /// Returns true when something new is now watched.
    pub fn ensure(&mut self) -> bool {
        let mut added = false;
        let projects = self.root.join("projects");
        let sessions = self.root.join("sessions");
        if !self.projects && projects.is_dir() && self.w.watch(&projects, RecursiveMode::Recursive).is_ok() {
            self.projects = true;
            added = true;
        }
        if !self.sessions && sessions.is_dir() && self.w.watch(&sessions, RecursiveMode::NonRecursive).is_ok() {
            self.sessions = true;
            added = true;
        }
        if self.complete() {
            if self.root_watched {
                let _ = self.w.unwatch(&self.root);
                self.root_watched = false;
            }
        } else if !self.root_watched
            && self.root.is_dir()
            && self.w.watch(&self.root, RecursiveMode::NonRecursive).is_ok()
        {
            self.root_watched = true;
        }
        added
    }
}
