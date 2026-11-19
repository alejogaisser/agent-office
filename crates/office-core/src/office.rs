//! The office: owns every tracked session, turns file changes into deltas.
use crate::config::Config;
use crate::discovery::{
    self, find_transcript, list_subagents, mtime_ms, project_name, read_registry, scan_recent,
    RegistryEntry, SessionFiles, SubFiles,
};
use crate::line::{parse_into, truncate, LineEvent, ParseStats, SpawnInfo};
use crate::model::*;
use crate::probe::ProcessProbe;
use crate::reducer::AgentTracker;
use crate::roles::{role_for, team_order};
use crate::tail::{StartAt, TailReader};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

const RESCAN_MS: i64 = 15_000;
const MAX_SPAWNS: usize = 2048;

struct SpawnRec {
    owner: String,
    info: SpawnInfo,
}

struct Sub {
    tracker: AgentTracker,
    tail: TailReader,
    meta_path: Option<PathBuf>,
    tool_use_id: Option<String>,
    announced: bool,
    mtime: i64,
}

struct Emitted {
    header: SessionView,
    agents: HashMap<String, AgentView>,
}

struct Session {
    id: String,
    session_dir: PathBuf,
    main_path: PathBuf,
    slug: String,
    main: AgentTracker,
    main_tail: TailReader,
    subs: BTreeMap<String, Sub>,
    cwd: String,
    git_branch: Option<String>,
    source: SessionSource,
    reg: Option<RegistryEntry>,
    custom_title: Option<String>,
    agent_name: Option<String>,
    last_prompt: Option<String>,
    first_ts: Option<i64>,
    newest_mtime: i64,
    closed_at: Option<i64>,
    spawns: HashMap<String, SpawnRec>,
    results: HashSet<String>,
    emitted: Option<Emitted>,
    /// Last subagent return: (agent id, team, at_ms). Drives the relay rule.
    last_return: Option<(String, String, i64)>,
    stats: ParseStats,
}

#[derive(Deserialize)]
struct Meta {
    #[serde(rename = "agentType")]
    agent_type: Option<String>,
    description: Option<String>,
    #[serde(rename = "toolUseId")]
    tool_use_id: Option<String>,
    #[serde(rename = "spawnDepth")]
    spawn_depth: Option<u64>,
}

#[derive(Deserialize)]
struct TitleJson {
    #[serde(rename = "customTitle")]
    custom_title: Option<String>,
}

fn apply_meta(sub: &mut Sub, path: &Path) {
    let Ok(bytes) = std::fs::read(path) else { return };
    let Ok(m) = serde_json::from_slice::<Meta>(&bytes) else { return };
    if let Some(t) = m.agent_type.as_deref().filter(|t| !t.is_empty()) {
        sub.tracker.role = role_for(Some(t), false);
        sub.tracker.role_final = true;
    }
    if let Some(d) = m.description {
        sub.tracker.description = Some(truncate(&d, 200));
    }
    if let Some(t) = m.tool_use_id {
        sub.tool_use_id = Some(truncate(&t, 200));
    }
    if let Some(d) = m.spawn_depth {
        sub.tracker.spawn_depth = d.min(255) as u8;
    }
    sub.meta_path = Some(path.to_path_buf());
}

fn event_ts(ev: &LineEvent) -> Option<i64> {
    match ev {
        LineEvent::ToolUse { ts, .. }
        | LineEvent::AssistantMsg { ts, .. }
        | LineEvent::ToolResult { ts, .. }
        | LineEvent::UserPrompt { ts }
        | LineEvent::TurnEnd { ts }
        | LineEvent::ApiError { ts } => Some(*ts),
        _ => None,
    }
}

impl Session {
    fn ingest(&mut self, agent: &str, lines: Vec<Vec<u8>>, now: i64) {
        for l in lines {
            for ev in parse_into(&l, &mut self.stats) {
                if let Some(ts) = event_ts(&ev).filter(|t| *t > 0 && *t <= now) {
                    self.first_ts = Some(self.first_ts.map_or(ts, |f| f.min(ts)));
                    if agent == "main" {
                        self.main.started_at = self.main.started_at.min(ts);
                    } else if let Some(s) = self.subs.get_mut(agent) {
                        s.tracker.started_at = s.tracker.started_at.min(ts);
                    }
                }
                if agent == "main" {
                    self.main.apply(&ev, now);
                } else if let Some(s) = self.subs.get_mut(agent) {
                    s.tracker.apply(&ev, now);
                }
                self.on_event(agent, &ev);
            }
        }
        self.resolve(now);
    }

    fn on_event(&mut self, agent: &str, ev: &LineEvent) {
        let main = agent == "main";
        match ev {
            LineEvent::Context { cwd, git_branch } => {
                if self.cwd.is_empty() {
                    if let Some(c) = cwd {
                        self.cwd = c.clone();
                    }
                }
                if main {
                    if let Some(b) = git_branch {
                        self.git_branch = Some(truncate(b, 200));
                    }
                }
            }
            LineEvent::Title(t) if main => self.custom_title = Some(t.clone()),
            LineEvent::AgentName(t) if main => self.agent_name = Some(t.clone()),
            LineEvent::LastPrompt(t) if main => self.last_prompt = Some(t.clone()),
            LineEvent::ToolUse { id, spawn: Some(info), .. } => {
                if self.spawns.len() < MAX_SPAWNS {
                    self.spawns.insert(
                        truncate(id, 200),
                        SpawnRec { owner: agent.to_string(), info: info.clone() },
                    );
                }
            }
            LineEvent::ToolResult { tool_use_id, .. } => {
                if let Some(rec) = self.spawns.get(tool_use_id.as_str()) {
                    if !rec.info.background {
                        self.results.insert(tool_use_id.clone());
                    }
                }
            }
            _ => {}
        }
    }

    /// Parents, descriptions and foreground completion from the spawn table.
    fn resolve(&mut self, now: i64) {
        let Session { subs, spawns, results, .. } = self;
        for (id, sub) in subs.iter_mut() {
            let Some(t) = sub.tool_use_id.as_ref() else { continue };
            let Some(rec) = spawns.get(t) else { continue };
            if rec.owner != *id && sub.tracker.parent_id.as_deref() != Some(rec.owner.as_str()) {
                sub.tracker.parent_id = Some(rec.owner.clone());
            }
            if sub.tracker.description.is_none() && !rec.info.description.is_empty() {
                sub.tracker.description = Some(rec.info.description.clone());
            }
            if results.contains(t) {
                sub.tracker.mark_done(now);
            }
        }
    }

    fn add_sub(&mut self, sf: SubFiles, cutoff: i64, now: i64, max_initial: u64, announced: bool) {
        let id = sf.agent_id.clone();
        let mut tracker = AgentTracker::new(&id, false, now);
        let parse = sf.mtime_ms >= cutoff;
        let tail = TailReader::new(
            &sf.path,
            if parse { StartAt::LastBytes(max_initial) } else { StartAt::End },
        );
        tracker.tokens.partial = tail.truncated_start || !parse;
        let mut sub = Sub { tracker, tail, meta_path: None, tool_use_id: None, announced, mtime: sf.mtime_ms };
        if let Some(mp) = &sf.meta_path {
            apply_meta(&mut sub, mp);
        }
        if !parse {
            sub.tracker.mark_done(now);
        }
        self.subs.insert(id.clone(), sub);
        if parse {
            self.read_agent(&id, now);
        } else {
            self.resolve(now);
        }
    }

    fn read_agent(&mut self, agent: &str, now: i64) -> bool {
        let lines = if agent == "main" {
            self.main_tail.read_new().unwrap_or_default()
        } else {
            match self.subs.get_mut(agent) {
                Some(s) => s.tail.read_new().unwrap_or_default(),
                None => return false,
            }
        };
        let path = if agent == "main" {
            Some(self.main_path.clone())
        } else {
            self.subs.get(agent).map(|s| s.tail.path.clone())
        };
        if let Some(m) = path.as_deref().and_then(mtime_ms) {
            self.newest_mtime = self.newest_mtime.max(m);
            if let Some(s) = self.subs.get_mut(agent) {
                s.mtime = m;
            }
        }
        let any = !lines.is_empty();
        self.ingest(agent, lines, now);
        any
    }

    fn load_title_json(&mut self) {
        let p = self.session_dir.join("custom-title.json");
        if let Ok(b) = std::fs::read(&p) {
            if let Ok(t) = serde_json::from_slice::<TitleJson>(&b) {
                if let Some(t) = t.custom_title.filter(|t| !t.is_empty()) {
                    self.custom_title = Some(truncate(&t, 200));
                }
            }
        }
    }

    fn refresh_mtime(&mut self) {
        let mut newest = self.newest_mtime;
        if let Some(m) = mtime_ms(&self.main_path) {
            newest = newest.max(m);
        }
        for s in self.subs.values() {
            if let Some(m) = mtime_ms(&s.tail.path) {
                newest = newest.max(m);
            }
        }
        self.newest_mtime = newest;
    }

    fn title(&self) -> String {
        if let Some(n) = self.reg.as_ref().and_then(|r| r.name.clone()) {
            return n;
        }
        if let Some(t) = self.custom_title.as_ref().filter(|t| !t.is_empty()) {
            return t.clone();
        }
        if let Some(t) = self.agent_name.as_ref().filter(|t| !t.is_empty()) {
            return t.clone();
        }
        if let Some(t) = self.last_prompt.as_ref().filter(|t| !t.is_empty()) {
            return truncate(t, 40);
        }
        self.project()
    }

    fn project(&self) -> String {
        let cwd = if self.cwd.is_empty() {
            self.reg.as_ref().map(|r| r.cwd.as_str()).unwrap_or("")
        } else {
            self.cwd.as_str()
        };
        project_name(cwd, &self.slug)
    }

    fn status(&self) -> SessionStatus {
        if self.closed_at.is_some() {
            return SessionStatus::Closed;
        }
        let reg_waiting = self.reg.as_ref().and_then(|r| r.status.as_deref()) == Some("waiting");
        if self.main.activity == Activity::Waiting || reg_waiting {
            return SessionStatus::Waiting;
        }
        if self.main.activity.is_working() || self.subs.values().any(|s| s.tracker.activity.is_working()) {
            return SessionStatus::Working;
        }
        SessionStatus::Idle
    }

    /// Header (agents empty) plus the agents shown on the floor.
    fn views(&self, cfg: &Config) -> (SessionView, Vec<AgentView>) {
        let mut agents = vec![self.main.view()];
        let mut active: Vec<AgentView> = Vec::new();
        let mut done: Vec<AgentView> = Vec::new();
        for s in self.subs.values() {
            let v = s.tracker.view();
            if v.activity == Activity::Done {
                done.push(v);
            } else {
                active.push(v);
            }
        }
        done.sort_by(|a, b| b.last_activity_at.cmp(&a.last_activity_at));
        let keep = cfg.max_done_shown as usize;
        let archived = done.len().saturating_sub(keep) as u32;
        done.truncate(keep);
        active.extend(done);
        active.sort_by(|a, b| a.started_at.cmp(&b.started_at).then_with(|| a.id.cmp(&b.id)));
        agents.extend(active);
        let last = std::iter::once(self.main.last_activity_at)
            .chain(self.subs.values().map(|s| s.tracker.last_activity_at))
            .max()
            .unwrap_or(0);
        let started = match self.reg.as_ref().map(|r| r.started_at) {
            Some(s) if s > 0 => s,
            _ => self.first_ts.unwrap_or(self.main.started_at),
        };
        let cwd = if self.cwd.is_empty() {
            self.reg.as_ref().map(|r| r.cwd.clone()).unwrap_or_default()
        } else {
            self.cwd.clone()
        };
        let mut active_teams: Vec<String> = Vec::new();
        for a in agents.iter().skip(1) {
            if let Some(t) = a.role.team.as_ref() {
                if !active_teams.contains(t) {
                    active_teams.push(t.clone());
                }
            }
        }
        active_teams.sort_by_key(|t| team_order(t));
        if active_teams.is_empty() {
            active_teams.push("general".into());
        }
        let header = SessionView {
            id: self.id.clone(),
            title: self.title(),
            project: self.project(),
            cwd,
            git_branch: self.git_branch.clone(),
            status: self.status(),
            source: self.source,
            started_at: started,
            last_activity_at: last,
            agents: Vec::new(),
            archived_done: archived,
            active_teams,
        };
        (header, agents)
    }

    fn diff(&mut self, cfg: &Config, delta: &mut Delta) {
        let (header, visible) = self.views(cfg);
        let prev = self.emitted.take();
        let mut agents = HashMap::new();
        match prev {
            None => {
                delta.sessions_upserted.push(header.clone());
                for a in &visible {
                    delta.agents_upserted.push((self.id.clone(), a.clone()));
                    agents.insert(a.id.clone(), a.clone());
                }
                for s in self.subs.values_mut() {
                    s.announced = true;
                }
            }
            Some(p) => {
                if p.header != header {
                    delta.sessions_upserted.push(header.clone());
                }
                for a in &visible {
                    let known = p.agents.get(&a.id);
                    if known != Some(a) {
                        delta.agents_upserted.push((self.id.clone(), a.clone()));
                    }
                    let team_of = |id: &str| -> Option<String> {
                        visible.iter().find(|v| v.id == id).and_then(|v| v.role.team.clone())
                    };
                    if let Some(k) = known {
                        if a.id != "main" && k.activity != Activity::Done && a.activity == Activity::Done {
                            let to = a.parent_id.clone().unwrap_or_else(|| "main".into());
                            let team = a.role.team.clone();
                            delta.handoffs.push(Handoff {
                                session_id: self.id.clone(),
                                kind: HandoffKind::Return,
                                from_agent_id: a.id.clone(),
                                to_team: if to == "main" { None } else { team_of(&to) },
                                to_agent_id: to,
                                from_team: team.clone(),
                                relay_from_agent_id: None,
                                at_ms: a.activity_since,
                            });
                            if let Some(t) = team {
                                self.last_return = Some((a.id.clone(), t, a.activity_since));
                            }
                        }
                    }
                    if known.is_none() {
                        if let Some(s) = self.subs.get_mut(&a.id) {
                            if !s.announced {
                                let from = a.parent_id.clone().unwrap_or_else(|| "main".into());
                                delta.spawned.push(Spawn {
                                    session_id: self.id.clone(),
                                    agent_id: a.id.clone(),
                                    from_agent_id: from.clone(),
                                });
                                let to_team = a.role.team.clone();
                                let window = cfg.relay_window_seconds as i64 * 1000;
                                let relay = match (&self.last_return, &to_team) {
                                    (Some((rid, rteam, rat)), Some(tt))
                                        if from == "main"
                                            && rteam != tt
                                            && a.started_at >= *rat
                                            && a.started_at - *rat <= window =>
                                    {
                                        Some(rid.clone())
                                    }
                                    _ => None,
                                };
                                delta.handoffs.push(Handoff {
                                    session_id: self.id.clone(),
                                    kind: HandoffKind::Spawn,
                                    from_team: if from == "main" { None } else { team_of(&from) },
                                    from_agent_id: from,
                                    to_agent_id: a.id.clone(),
                                    to_team,
                                    relay_from_agent_id: relay,
                                    at_ms: a.started_at,
                                });
                            }
                            s.announced = true;
                        }
                    }
                    agents.insert(a.id.clone(), a.clone());
                }
                for id in p.agents.keys() {
                    if !agents.contains_key(id) {
                        delta.agents_removed.push((self.id.clone(), id.clone()));
                    }
                }
            }
        }
        self.emitted = Some(Emitted { header, agents });
    }
}

enum Class {
    Registry,
    Title { sid: String },
    Main { slug: String, sid: String },
    Meta { sid: String, agent: String, path: PathBuf },
    Sub { slug: String, sid: String },
}

fn unify(p: &Path) -> String {
    let s = p.to_string_lossy().replace('/', "\\");
    s.strip_prefix("\\\\?\\").map(str::to_string).unwrap_or(s)
}

fn strip_dir<'a>(file: &'a str, dir: &str) -> Option<&'a str> {
    let dir = dir.trim_end_matches('\\');
    if file.len() > dir.len() + 1
        && file.is_char_boundary(dir.len())
        && file[..dir.len()].eq_ignore_ascii_case(dir)
        && file.as_bytes()[dir.len()] == b'\\'
    {
        Some(&file[dir.len() + 1..])
    } else {
        None
    }
}

pub struct Office {
    cfg: Config,
    probe: Box<dyn ProcessProbe>,
    sessions: BTreeMap<String, Session>,
    registry: HashMap<String, RegistryEntry>,
    removed: Vec<String>,
    error: Option<String>,
    last_scan_ms: i64,
    last_now: i64,
}

impl Office {
    pub fn new(cfg: Config, probe: Box<dyn ProcessProbe>) -> Office {
        Office {
            cfg,
            probe,
            sessions: BTreeMap::new(),
            registry: HashMap::new(),
            removed: Vec::new(),
            error: None,
            last_scan_ms: 0,
            last_now: 0,
        }
    }

    /// Makes the next `poll` rescan the claude dir (used when watched dirs appear late).
    pub fn request_rescan(&mut self) {
        self.last_scan_ms = i64::MIN / 2;
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    pub fn parse_stats(&self) -> ParseStats {
        let mut t = ParseStats::default();
        for s in self.sessions.values() {
            t.malformed += s.stats.malformed;
            t.oversized += s.stats.oversized + s.main_tail.stats.oversized;
        }
        t
    }

    fn projects_dir(&self) -> PathBuf {
        self.cfg.claude_dir.join("projects")
    }

    fn sessions_dir(&self) -> PathBuf {
        self.cfg.claude_dir.join("sessions")
    }

    fn window_ms(&self) -> i64 {
        self.cfg.active_window_minutes as i64 * 60_000
    }

    fn alive(&self, e: &RegistryEntry) -> bool {
        self.probe.is_alive(e.pid, e.proc_start)
    }

    /// A live process alone is not enough: the desktop app keeps recent sessions
    /// warm in the background. Busy/waiting, or transcript activity within the
    /// active window, is required.
    fn reg_in_use(&self, e: &RegistryEntry, files: &SessionFiles, now: i64) -> bool {
        if registry_busy(e) {
            return true;
        }
        let newest = files.subagents.iter().map(|s| s.mtime_ms).fold(files.mtime_ms, i64::max);
        newest >= now - self.window_ms()
    }

    fn reload_registry(&mut self) {
        self.registry.clear();
        if !self.cfg.use_session_registry {
            return;
        }
        for e in read_registry(&self.sessions_dir()) {
            let better = match self.registry.get(&e.session_id) {
                None => true,
                Some(old) => {
                    let (a_new, a_old) = (self.alive(&e), self.alive(old));
                    (a_new && !a_old) || (a_new == a_old && e.updated_at > old.updated_at)
                }
            };
            if better {
                self.registry.insert(e.session_id.clone(), e);
            }
        }
    }

    pub fn initial_scan(&mut self, now_ms: i64) -> Snapshot {
        self.last_now = now_ms;
        self.sessions.clear();
        self.removed.clear();
        self.last_scan_ms = now_ms;
        self.error = None;
        if !self.cfg.claude_dir.is_dir() {
            self.error = Some(format!("claude dir not found: {}", self.cfg.claude_dir.display()));
            return self.snapshot_at(now_ms);
        }
        self.scan(now_ms);
        let mut sink = Delta::default();
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            if let Some(s) = self.sessions.get_mut(&id) {
                s.diff(&self.cfg, &mut sink);
            }
        }
        self.snapshot_at(now_ms)
    }

    /// Registry sessions first, then recent transcripts. Adds only new sessions.
    fn scan(&mut self, now: i64) {
        self.reload_registry();
        let entries: Vec<RegistryEntry> = self.registry.values().cloned().collect();
        for e in entries {
            if self.sessions.contains_key(&e.session_id) || !self.alive(&e) {
                continue;
            }
            if let Some(f) = find_transcript(&self.projects_dir(), &e.cwd, &e.session_id) {
                if self.reg_in_use(&e, &f, now) {
                    self.add_session(f, now);
                }
            }
        }
        let cutoff = now - self.window_ms();
        for f in scan_recent(&self.projects_dir(), cutoff) {
            if !self.sessions.contains_key(&f.session_id) {
                self.add_session(f, now);
            }
        }
    }

    /// Adds a session unless its registry entry says the process is gone.
    fn add_session(&mut self, files: SessionFiles, now: i64) -> bool {
        let reg = self.registry.get(&files.session_id).cloned();
        let source = match &reg {
            Some(e) if self.alive(e) => SessionSource::Registry,
            Some(_) => return false,
            None => SessionSource::Mtime,
        };
        let cutoff = now - self.window_ms();
        let main_tail = TailReader::new(&files.main_path, StartAt::LastBytes(self.cfg.max_initial_read_bytes));
        let mut main = AgentTracker::new("main", true, now);
        main.tokens.partial = main_tail.truncated_start;
        let mut s = Session {
            id: files.session_id.clone(),
            session_dir: files.main_path.with_extension(""),
            main_path: files.main_path.clone(),
            slug: files.slug.clone(),
            main,
            main_tail,
            subs: BTreeMap::new(),
            cwd: String::new(),
            git_branch: None,
            source,
            reg,
            custom_title: None,
            agent_name: None,
            last_prompt: None,
            first_ts: None,
            newest_mtime: files.mtime_ms,
            closed_at: None,
            spawns: HashMap::new(),
            results: HashSet::new(),
            emitted: None,
            last_return: None,
            stats: ParseStats::default(),
        };
        s.load_title_json();
        s.read_agent("main", now);
        for sf in files.subagents {
            s.newest_mtime = s.newest_mtime.max(sf.mtime_ms);
            s.add_sub(sf, cutoff, now, self.cfg.max_initial_read_bytes, true);
        }
        self.sessions.insert(files.session_id, s);
        true
    }

    fn remove_session(&mut self, id: &str) {
        if let Some(s) = self.sessions.remove(id) {
            if s.emitted.is_some() {
                self.removed.push(id.to_string());
            }
        }
    }

    fn classify(&self, p: &Path) -> Option<Class> {
        let name = p.file_name()?.to_str()?;
        if name.ends_with(".key") {
            return None;
        }
        let full = unify(p);
        if let Some(rest) = strip_dir(&full, &unify(&self.sessions_dir())) {
            return if !rest.contains('\\') && discovery::is_registry_file(rest) {
                Some(Class::Registry)
            } else {
                None
            };
        }
        let rest = strip_dir(&full, &unify(&self.projects_dir()))?;
        let comps: Vec<&str> = rest.split('\\').collect();
        match comps.as_slice() {
            [slug, f] => {
                let sid = f.strip_suffix(".jsonl")?;
                Some(Class::Main { slug: slug.to_string(), sid: sid.to_string() })
            }
            [_, sid, "custom-title.json"] => Some(Class::Title { sid: sid.to_string() }),
            [slug, sid, "subagents", f] => {
                if f.starts_with("agent-") && f.ends_with(".jsonl") {
                    Some(Class::Sub { slug: slug.to_string(), sid: sid.to_string() })
                } else if let Some(a) = f.strip_prefix("agent-").and_then(|f| f.strip_suffix(".meta.json")) {
                    Some(Class::Meta { sid: sid.to_string(), agent: a.to_string(), path: p.to_path_buf() })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Adds the session for a changed file when it is fresh and not hidden.
    fn ensure_session(&mut self, slug: &str, sid: &str, now: i64) {
        if self.sessions.contains_key(sid) {
            return;
        }
        let main_path = self.projects_dir().join(slug).join(format!("{sid}.jsonl"));
        let Some(m) = mtime_ms(&main_path) else { return };
        let live_registry = self.registry.get(sid).map(|e| self.alive(e)).unwrap_or(false);
        if m < now - self.window_ms() && !live_registry {
            return;
        }
        let session_dir = main_path.with_extension("");
        let files = SessionFiles {
            session_id: sid.to_string(),
            slug: slug.to_string(),
            subagents: list_subagents(&session_dir),
            main_path,
            mtime_ms: m,
        };
        self.add_session(files, now);
    }

    pub fn on_paths_changed(&mut self, paths: &[PathBuf], now_ms: i64) -> Option<Delta> {
        self.last_now = now_ms;
        if self.error.is_some() {
            if self.cfg.claude_dir.is_dir() {
                self.error = None;
            } else {
                return None;
            }
        }
        let mut classes: Vec<(u8, Class, PathBuf)> = Vec::new();
        for p in paths {
            let Some(c) = self.classify(p) else { continue };
            let pri = match c {
                Class::Registry => 0,
                Class::Title { .. } => 1,
                Class::Main { .. } => 2,
                Class::Meta { .. } => 3,
                Class::Sub { .. } => 4,
            };
            classes.push((pri, c, p.clone()));
        }
        if classes.is_empty() {
            return None;
        }
        classes.sort_by_key(|c| c.0);
        let mut touched: HashSet<String> = HashSet::new();
        let mut registry_done = false;
        for (_, class, path) in classes {
            match class {
                Class::Registry => {
                    if !registry_done {
                        registry_done = true;
                        self.apply_registry(now_ms);
                        touched.extend(self.sessions.keys().cloned());
                    }
                }
                Class::Title { sid } => {
                    if let Some(s) = self.sessions.get_mut(&sid) {
                        s.load_title_json();
                        touched.insert(sid);
                    }
                }
                Class::Main { slug, sid } => {
                    self.ensure_session(&slug, &sid, now_ms);
                    if let Some(s) = self.sessions.get_mut(&sid) {
                        s.read_agent("main", now_ms);
                        touched.insert(sid);
                    }
                }
                Class::Meta { sid, agent, path } => {
                    if let Some(s) = self.sessions.get_mut(&sid) {
                        if let Some(sub) = s.subs.get_mut(&agent) {
                            apply_meta(sub, &path);
                            s.resolve(now_ms);
                            touched.insert(sid);
                        }
                    }
                }
                Class::Sub { slug, sid } => {
                    self.ensure_session(&slug, &sid, now_ms);
                    let Some(agent) = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .and_then(|n| n.strip_prefix("agent-"))
                        .and_then(|n| n.strip_suffix(".jsonl"))
                        .map(str::to_string)
                    else {
                        continue;
                    };
                    let max_initial = self.cfg.max_initial_read_bytes;
                    if let Some(s) = self.sessions.get_mut(&sid) {
                        if s.subs.contains_key(&agent) {
                            s.read_agent(&agent, now_ms);
                        } else {
                            let meta = path
                                .with_file_name(format!("agent-{agent}.meta.json"));
                            s.add_sub(
                                SubFiles {
                                    agent_id: agent.clone(),
                                    mtime_ms: mtime_ms(&path).unwrap_or(now_ms),
                                    meta_path: if meta.is_file() { Some(meta) } else { None },
                                    path: path.clone(),
                                },
                                i64::MIN,
                                now_ms,
                                max_initial,
                                false,
                            );
                        }
                        touched.insert(sid);
                    }
                }
            }
        }
        self.finish(now_ms, Some(&touched))
    }

    /// Re-reads the registry, updates liveness and adds newly alive sessions.
    fn apply_registry(&mut self, now: i64) {
        self.reload_registry();
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            let entry = self.registry.get(&id).cloned();
            let alive = entry.as_ref().map(|e| self.alive(e));
            let Some(s) = self.sessions.get_mut(&id) else { continue };
            match (entry, alive) {
                (Some(e), Some(true)) => {
                    s.reg = Some(e);
                    s.source = SessionSource::Registry;
                    s.closed_at = None;
                }
                (Some(e), Some(false)) => {
                    s.reg = Some(e);
                    if s.source == SessionSource::Registry {
                        s.closed_at.get_or_insert(now);
                    } else {
                        self.remove_session(&id);
                    }
                }
                _ => {
                    if s.source == SessionSource::Registry {
                        s.closed_at.get_or_insert(now);
                    }
                }
            }
        }
        let entries: Vec<RegistryEntry> = self.registry.values().cloned().collect();
        for e in entries {
            if self.sessions.contains_key(&e.session_id) || !self.alive(&e) {
                continue;
            }
            if let Some(f) = find_transcript(&self.projects_dir(), &e.cwd, &e.session_id) {
                if self.reg_in_use(&e, &f, now) {
                    self.add_session(f, now);
                }
            }
        }
    }

    /// Process death, closed grace and mtime expiry. No file reads.
    fn lifecycle(&mut self, now: i64) {
        let window = self.window_ms();
        let grace = self.cfg.closed_grace_seconds as i64 * 1000;
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            let Some(s) = self.sessions.get_mut(&id) else { continue };
            match s.source {
                SessionSource::Registry => {
                    if s.closed_at.is_none() {
                        let dead = match &s.reg {
                            Some(e) => !self.probe.is_alive(e.pid, e.proc_start),
                            None => true,
                        };
                        if dead {
                            s.closed_at = Some(now);
                        }
                    }
                    if let Some(c) = s.closed_at {
                        if now - c >= grace {
                            self.remove_session(&id);
                            continue;
                        }
                    }
                    // Alive but parked in the background: hide once quiet for the window.
                    let busy = s.reg.as_ref().map(registry_busy).unwrap_or(false);
                    if s.closed_at.is_none() && !busy {
                        s.refresh_mtime();
                        if now - s.newest_mtime > window {
                            self.remove_session(&id);
                        }
                    }
                }
                SessionSource::Mtime => {
                    s.refresh_mtime();
                    if now - s.newest_mtime > window {
                        self.remove_session(&id);
                    }
                }
            }
        }
    }

    fn tick_trackers(&mut self, now: i64) {
        let cfg = self.cfg.clone();
        for s in self.sessions.values_mut() {
            s.main.tick(now, &cfg);
            for sub in s.subs.values_mut() {
                sub.tracker.tick(now, &cfg);
            }
        }
    }

    pub fn tick(&mut self, now_ms: i64) -> Option<Delta> {
        self.last_now = now_ms;
        self.lifecycle(now_ms);
        self.tick_trackers(now_ms);
        self.finish(now_ms, None)
    }

    /// Fallback poll: reads files that grew, drops vanished ones, rescans now and then.
    pub fn poll(&mut self, now_ms: i64) -> Option<Delta> {
        self.last_now = now_ms;
        if self.error.is_some() {
            if self.cfg.claude_dir.is_dir() {
                self.error = None;
                self.last_scan_ms = now_ms;
                self.scan(now_ms);
            }
            return self.finish(now_ms, None);
        }
        let window_cutoff = now_ms - self.window_ms();
        let max_initial = self.cfg.max_initial_read_bytes;
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            let Some(s) = self.sessions.get_mut(&id) else { continue };
            if s.main_tail.has_new_data() {
                s.read_agent("main", now_ms);
            }
            if s.main_tail.gone {
                self.remove_session(&id);
                continue;
            }
            let agents: Vec<String> = s.subs.keys().cloned().collect();
            for a in agents {
                let (grew, gone) = match s.subs.get(&a) {
                    Some(sub) => (sub.tail.has_new_data(), sub.tail.gone),
                    None => continue,
                };
                if grew {
                    s.read_agent(&a, now_ms);
                }
                if s.subs.get(&a).map(|x| x.tail.gone).unwrap_or(gone) {
                    s.subs.remove(&a);
                }
            }
            // pick up subagent files whose notifications were missed
            if now_ms - self.last_scan_ms >= RESCAN_MS {
                for sf in list_subagents(&s.session_dir) {
                    if !s.subs.contains_key(&sf.agent_id) {
                        s.add_sub(sf, window_cutoff, now_ms, max_initial, false);
                    }
                }
            }
        }
        if now_ms - self.last_scan_ms >= RESCAN_MS {
            self.last_scan_ms = now_ms;
            self.apply_registry(now_ms);
            self.scan(now_ms);
        }
        self.lifecycle(now_ms);
        self.tick_trackers(now_ms);
        self.finish(now_ms, None)
    }

    pub fn next_deadline_ms(&self) -> Option<i64> {
        let window = self.window_ms();
        let grace = self.cfg.closed_grace_seconds as i64 * 1000;
        let mut best: Option<i64> = None;
        let mut upd = |v: Option<i64>| {
            if let Some(v) = v {
                best = Some(best.map_or(v, |b| b.min(v)));
            }
        };
        for s in self.sessions.values() {
            upd(s.main.next_deadline_ms(&self.cfg));
            for sub in s.subs.values() {
                upd(sub.tracker.next_deadline_ms(&self.cfg));
            }
            match s.source {
                SessionSource::Registry => upd(s.closed_at.map(|c| c + grace)),
                SessionSource::Mtime => upd(Some(s.newest_mtime + window + 1)),
            }
        }
        best
    }

    fn finish(&mut self, now: i64, touched: Option<&HashSet<String>>) -> Option<Delta> {
        let mut delta = Delta::default();
        for id in self.removed.drain(..) {
            delta.sessions_removed.push(id);
        }
        let _ = now;
        for (id, s) in self.sessions.iter_mut() {
            if touched.map_or(true, |t| t.contains(id)) {
                s.diff(&self.cfg, &mut delta);
            }
        }
        if delta.is_empty() {
            None
        } else {
            Some(delta)
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot_at(self.last_now)
    }

    fn snapshot_at(&self, generated_at: i64) -> Snapshot {
        let mut sessions: Vec<SessionView> = self
            .sessions
            .values()
            .map(|s| {
                let (mut h, agents) = s.views(&self.cfg);
                h.agents = agents;
                h
            })
            .collect();
        sessions.sort_by(|a, b| a.started_at.cmp(&b.started_at).then_with(|| a.id.cmp(&b.id)));
        Snapshot { generated_at, sessions, error: self.error.clone() }
    }

    pub fn tracked_paths(&self) -> Vec<PathBuf> {
        let mut v = Vec::new();
        for s in self.sessions.values() {
            v.push(s.main_path.clone());
            for sub in s.subs.values() {
                v.push(sub.tail.path.clone());
            }
        }
        v
    }

    /// True when any tracked agent is in a working activity.
    pub fn any_working(&self) -> bool {
        self.sessions
            .values()
            .any(|s| s.main.activity.is_working() || s.subs.values().any(|x| x.tracker.activity.is_working()))
    }
}

/// Registry says the session is doing something (not just parked).
fn registry_busy(e: &RegistryEntry) -> bool {
    matches!(e.status.as_deref(), Some("busy") | Some("waiting"))
}
