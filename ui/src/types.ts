// Mirrors crates/office-core/src/model.rs (serde camelCase).
export type Activity =
  | "thinking"
  | "writing"
  | "reading"
  | "running"
  | "browsing"
  | "delegating"
  | "planning"
  | "tool"
  | "waiting"
  | "sleeping"
  | "done"
  | "error";

export interface Tokens {
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
  context: number;
  partial: boolean;
}

export interface RoleInfo {
  key: string;
  label: string;
  color: string;
  team: string | null;
  room: string;
}

export interface AgentView {
  id: string;
  parentId: string | null;
  role: RoleInfo;
  description: string | null;
  activity: Activity;
  toolName: string | null;
  target: string | null;
  activitySince: number;
  startedAt: number;
  lastActivityAt: number;
  tokens: Tokens;
  model: string | null;
  spawnDepth: number;
}

export type SessionStatus = "working" | "waiting" | "idle" | "closed";

export interface SessionView {
  id: string;
  title: string;
  project: string;
  cwd: string;
  gitBranch: string | null;
  status: SessionStatus;
  source: "registry" | "mtime";
  startedAt: number;
  lastActivityAt: number;
  agents: AgentView[];
  archivedDone: number;
  activeTeams: string[];
}

export interface Snapshot {
  generatedAt: number;
  sessions: SessionView[];
  error: string | null;
}

export interface Spawn {
  sessionId: string;
  agentId: string;
  fromAgentId: string;
}

export interface Delta {
  sessionsUpserted: SessionView[];
  sessionsRemoved: string[];
  agentsUpserted: [string, AgentView][];
  agentsRemoved: [string, string][];
  spawned: Spawn[];
  handoffs: Handoff[];
}

export interface Handoff {
  sessionId: string;
  kind: "spawn" | "return";
  fromAgentId: string;
  toAgentId: string;
  fromTeam: string | null;
  toTeam: string | null;
  relayFromAgentId: string | null;
  atMs: number;
}

export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

export type CellKind = "boss" | "role" | "other" | "stairs" | "lounge";

export interface Cell {
  key: string;
  label: string;
  kind: CellKind;
  rect: Rect;
  door: [number, number];
  doorNode: number;
  /** x, z, rotY: the desk center; rotY points to the door wall. */
  desks: [number, number, number][];
  standing: [number, number][];
}

export interface FloorPlan {
  width: number;
  depth: number;
  cells: Cell[];
  corridors: Rect[];
  nodes: [number, number][];
  edges: [number, number][];
}

export interface RoomSpec {
  key: string;
  label: string;
  color: string;
}

export interface TeamSpec {
  key: string;
  label: string;
  floor: number;
  rooms: RoomSpec[];
  plan: FloorPlan;
}

export interface Catalog {
  teams: TeamSpec[];
}

export const WORKING: ReadonlySet<Activity> = new Set<Activity>([
  "thinking",
  "writing",
  "reading",
  "running",
  "browsing",
  "delegating",
  "planning",
  "tool",
]);
