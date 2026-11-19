use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Activity {
    Thinking,
    Writing,
    Reading,
    Running,
    Browsing,
    Delegating,
    Planning,
    Tool,
    Waiting,
    Sleeping,
    Done,
    Error,
}

impl Activity {
    /// Activities that count as "the agent is doing something".
    pub fn is_working(self) -> bool {
        !matches!(
            self,
            Activity::Waiting | Activity::Sleeping | Activity::Done | Activity::Error
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tokens {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub context: u64,
    pub partial: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleInfo {
    pub key: String,
    pub label: String,
    pub color: String,
    pub team: Option<String>,
    /// Room key on the team floor (`boss` for main).
    pub room: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentView {
    pub id: String,
    pub parent_id: Option<String>,
    pub role: RoleInfo,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Working,
    Waiting,
    Idle,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionSource {
    Registry,
    Mtime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub id: String,
    pub title: String,
    pub project: String,
    pub cwd: String,
    pub git_branch: Option<String>,
    pub status: SessionStatus,
    pub source: SessionSource,
    pub started_at: i64,
    pub last_activity_at: i64,
    pub agents: Vec<AgentView>,
    pub archived_done: u32,
    /// Teams with visible subagents, in floor order (`["general"]` when none).
    #[serde(default)]
    pub active_teams: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub generated_at: i64,
    pub sessions: Vec<SessionView>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spawn {
    pub session_id: String,
    pub agent_id: String,
    pub from_agent_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delta {
    /// Session headers only: `agents` is empty here.
    pub sessions_upserted: Vec<SessionView>,
    pub sessions_removed: Vec<String>,
    pub agents_upserted: Vec<(String, AgentView)>,
    pub agents_removed: Vec<(String, String)>,
    pub spawned: Vec<Spawn>,
    #[serde(default)]
    pub handoffs: Vec<Handoff>,
}

impl Delta {
    pub fn is_empty(&self) -> bool {
        self.sessions_upserted.is_empty()
            && self.sessions_removed.is_empty()
            && self.agents_upserted.is_empty()
            && self.agents_removed.is_empty()
            && self.spawned.is_empty()
            && self.handoffs.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HandoffKind {
    Spawn,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Handoff {
    pub session_id: String,
    pub kind: HandoffKind,
    pub from_agent_id: String,
    pub to_agent_id: String,
    pub from_team: Option<String>,
    pub to_team: Option<String>,
    /// Spawn only: the agent of another team that returned shortly before.
    pub relay_from_agent_id: Option<String>,
    pub at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: f32,
    pub z: f32,
    pub w: f32,
    pub d: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellKind {
    Boss,
    Role,
    Other,
    Stairs,
    Lounge,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    pub key: String,
    pub label: String,
    pub kind: CellKind,
    pub rect: Rect,
    pub door: [f32; 2],
    pub door_node: u16,
    /// Desk centers: x, z, rotY (forward points to the door wall).
    pub desks: Vec<[f32; 3]>,
    pub standing: Vec<[f32; 2]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FloorPlan {
    pub width: f32,
    pub depth: f32,
    pub cells: Vec<Cell>,
    pub corridors: Vec<Rect>,
    pub nodes: Vec<[f32; 2]>,
    pub edges: Vec<[u16; 2]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomSpec {
    pub key: String,
    pub label: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamSpec {
    pub key: String,
    pub label: String,
    pub floor: u8,
    pub rooms: Vec<RoomSpec>,
    pub plan: FloorPlan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub teams: Vec<TeamSpec>,
}
