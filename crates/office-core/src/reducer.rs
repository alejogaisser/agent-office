//! Per-agent state machine fed by parsed transcript events.
use crate::config::Config;
use crate::line::{truncate, LineEvent, MAX_STR};
use crate::model::{Activity, AgentView, RoleInfo, Tokens};
use crate::roles::role_for;
use std::collections::VecDeque;

pub const PENDING_CAP: usize = 256;
pub const RECENT_CAP: usize = 20;
pub const SEEN_IDS_CAP: usize = 64;
pub const ERROR_MS: i64 = 10_000;

#[derive(Debug, Clone)]
pub struct Pending {
    pub id: String,
    pub name: String,
    pub target: Option<String>,
    pub ts: i64,
}

#[derive(Debug, Clone)]
pub struct Recent {
    pub ts: i64,
    pub tool: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentTracker {
    pub id: String,
    pub parent_id: Option<String>,
    pub is_main: bool,
    pub role: RoleInfo,
    /// True once the role came from the meta file (or is main).
    pub role_final: bool,
    pub description: Option<String>,
    pub activity: Activity,
    pub tool_name: Option<String>,
    pub target: Option<String>,
    pub activity_since: i64,
    pub started_at: i64,
    pub last_activity_at: i64,
    pub tokens: Tokens,
    pub model: Option<String>,
    pub spawn_depth: u8,
    pub pending: VecDeque<Pending>,
    pub recent: VecDeque<Recent>,
    seen_msg_ids: VecDeque<String>,
    prev: Option<(Activity, Option<String>, Option<String>)>,
    error_until: i64,
}

pub fn classify(tool: &str) -> Activity {
    match tool {
        "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => Activity::Writing,
        "Read" | "Grep" | "Glob" | "LS" => Activity::Reading,
        "Bash" | "PowerShell" | "BashOutput" | "KillShell" => Activity::Running,
        "WebFetch" | "WebSearch" => Activity::Browsing,
        "Agent" | "Task" | "SendMessage" | "ListAgents" | "ReadNotifications" => Activity::Delegating,
        "TodoWrite" => Activity::Planning,
        _ => Activity::Tool,
    }
}

fn clamp_ts(ts: i64, now: i64) -> i64 {
    if ts <= 0 || ts > now {
        now
    } else {
        ts
    }
}

impl AgentTracker {
    pub fn new(id: &str, is_main: bool, now_ms: i64) -> AgentTracker {
        AgentTracker {
            id: id.to_string(),
            parent_id: if is_main { None } else { Some("main".to_string()) },
            is_main,
            role: role_for(None, is_main),
            role_final: is_main,
            description: None,
            activity: if is_main { Activity::Waiting } else { Activity::Thinking },
            tool_name: None,
            target: None,
            activity_since: now_ms,
            started_at: now_ms,
            last_activity_at: now_ms,
            tokens: Tokens::default(),
            model: None,
            spawn_depth: if is_main { 0 } else { 1 },
            pending: VecDeque::new(),
            recent: VecDeque::new(),
            seen_msg_ids: VecDeque::new(),
            prev: None,
            error_until: 0,
        }
    }

    pub fn view(&self) -> AgentView {
        AgentView {
            id: self.id.clone(),
            parent_id: self.parent_id.clone(),
            role: self.role.clone(),
            description: self.description.clone(),
            activity: self.activity,
            tool_name: self.tool_name.clone(),
            target: self.target.clone(),
            activity_since: self.activity_since,
            started_at: self.started_at,
            last_activity_at: self.last_activity_at,
            tokens: self.tokens,
            model: self.model.clone(),
            spawn_depth: self.spawn_depth,
        }
    }

    fn set(&mut self, a: Activity, tool: Option<String>, target: Option<String>, t: i64) {
        if self.activity != a {
            self.activity_since = t;
        }
        self.activity = a;
        self.tool_name = tool;
        self.target = target;
        self.prev = None;
    }

    fn settle_from_pending(&mut self, t: i64) {
        match self.pending.back() {
            Some(p) => {
                let (a, n, tg) = (classify(&p.name), Some(p.name.clone()), p.target.clone());
                self.set(a, n, tg, t);
            }
            None => self.set(Activity::Thinking, None, None, t),
        }
    }

    pub fn mark_done(&mut self, now_ms: i64) -> bool {
        if self.activity == Activity::Done {
            return false;
        }
        self.pending.clear();
        self.set(Activity::Done, None, None, now_ms);
        true
    }

    /// Applies one event; returns true when visible state may have changed.
    pub fn apply(&mut self, ev: &LineEvent, now_ms: i64) -> bool {
        if self.activity == Activity::Done {
            // Done is sticky; only keep counting tokens.
            if let LineEvent::AssistantMsg { msg_id, usage, .. } = ev {
                return self.count_tokens(msg_id, usage);
            }
            return false;
        }
        match ev {
            LineEvent::ToolUse { id, name, target, ts, .. } => {
                let t = clamp_ts(*ts, now_ms);
                self.last_activity_at = t;
                if name == "SubagentHandback" {
                    self.pending.clear();
                    self.set(Activity::Done, None, None, t);
                    return true;
                }
                let name = truncate(name, MAX_STR);
                let target = target.as_ref().map(|s| truncate(s, MAX_STR));
                self.pending.push_back(Pending {
                    id: truncate(id, MAX_STR),
                    name: name.clone(),
                    target: target.clone(),
                    ts: t,
                });
                while self.pending.len() > PENDING_CAP {
                    self.pending.pop_front();
                }
                self.recent.push_back(Recent { ts: t, tool: name.clone(), target: target.clone() });
                while self.recent.len() > RECENT_CAP {
                    self.recent.pop_front();
                }
                self.set(classify(&name), Some(name), target, t);
                true
            }
            LineEvent::AssistantMsg { msg_id, usage, stop_reason, model, attribution_agent, ts } => {
                let t = clamp_ts(*ts, now_ms);
                self.last_activity_at = t;
                self.count_tokens(msg_id, usage);
                if let Some(m) = model {
                    if self.model.as_deref() != Some(m.as_str()) {
                        self.model = Some(truncate(m, MAX_STR));
                    }
                }
                if !self.is_main && !self.role_final {
                    if let Some(a) = attribution_agent {
                        self.role = role_for(Some(a), false);
                    }
                }
                if stop_reason.as_deref() == Some("end_turn") && self.pending.is_empty() {
                    let a = if self.is_main { Activity::Waiting } else { Activity::Done };
                    self.set(a, None, None, t);
                } else if self.pending.is_empty() && self.activity != Activity::Thinking {
                    self.set(Activity::Thinking, None, None, t);
                }
                true
            }
            LineEvent::ToolResult { tool_use_id, ts, .. } => {
                let t = clamp_ts(*ts, now_ms);
                self.last_activity_at = t;
                let key = truncate(tool_use_id, MAX_STR);
                if let Some(pos) = self.pending.iter().position(|p| p.id == key) {
                    self.pending.remove(pos);
                    self.settle_from_pending(t);
                    true
                } else {
                    false
                }
            }
            LineEvent::UserPrompt { ts } => {
                let t = clamp_ts(*ts, now_ms);
                self.last_activity_at = t;
                self.pending.clear();
                self.set(Activity::Thinking, None, None, t);
                true
            }
            LineEvent::TurnEnd { ts } => {
                let t = clamp_ts(*ts, now_ms);
                self.last_activity_at = t;
                self.pending.clear();
                let a = if self.is_main { Activity::Waiting } else { Activity::Done };
                self.set(a, None, None, t);
                true
            }
            LineEvent::ApiError { ts } => {
                let t = clamp_ts(*ts, now_ms);
                self.last_activity_at = t;
                if self.activity != Activity::Error {
                    self.prev = Some((self.activity, self.tool_name.clone(), self.target.clone()));
                    self.activity_since = t;
                }
                self.activity = Activity::Error;
                self.error_until = t + ERROR_MS;
                true
            }
            _ => false,
        }
    }

    fn count_tokens(&mut self, msg_id: &Option<String>, usage: &Option<crate::line::Usage>) -> bool {
        let Some(u) = usage else { return false };
        let is_new = match msg_id {
            Some(id) => {
                if self.seen_msg_ids.iter().any(|s| s == id) {
                    false
                } else {
                    self.seen_msg_ids.push_back(id.clone());
                    while self.seen_msg_ids.len() > SEEN_IDS_CAP {
                        self.seen_msg_ids.pop_front();
                    }
                    true
                }
            }
            None => true,
        };
        if is_new {
            self.tokens.input += u.input;
            self.tokens.output += u.output;
            self.tokens.cache_read += u.cache_read;
            self.tokens.cache_write += u.cache_write;
        }
        self.tokens.context = u.input + u.cache_read + u.cache_write;
        true
    }

    /// Time-driven transitions. Returns true when state changed.
    pub fn tick(&mut self, now_ms: i64, cfg: &Config) -> bool {
        if self.activity == Activity::Error && now_ms >= self.error_until {
            match self.prev.take() {
                Some((a, tool, target)) => {
                    self.activity = a;
                    self.tool_name = tool;
                    self.target = target;
                }
                None => self.activity = Activity::Thinking,
            }
            return true;
        }
        if matches!(self.activity, Activity::Waiting | Activity::Thinking)
            && now_ms - self.last_activity_at >= cfg.sleep_after_seconds as i64 * 1000
        {
            self.activity = Activity::Sleeping;
            self.activity_since = now_ms;
            return true;
        }
        false
    }

    pub fn next_deadline_ms(&self, cfg: &Config) -> Option<i64> {
        match self.activity {
            Activity::Error => Some(self.error_until),
            Activity::Waiting | Activity::Thinking => {
                Some(self.last_activity_at + cfg.sleep_after_seconds as i64 * 1000)
            }
            _ => None,
        }
    }
}
