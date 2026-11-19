import type { AgentView, Delta, Handoff, SessionView, Snapshot } from "./types";

export interface Changes {
  /** Sessions whose header or agents changed. */
  sessions: Set<string>;
  /** Changed agent ids per session. */
  agents: Map<string, Set<string>>;
  removedAgents: Map<string, Set<string>>;
  removedSessions: Set<string>;
  /** True when sessions were added or removed (sidebar needs a rebuild). */
  structural: boolean;
}

export class Store {
  sessions = new Map<string, SessionView>();
  selectedId: string | null = null;
  /** Team key of the floor shown in the selected building. */
  selectedFloor: string | null = null;
  /** True while the floor was auto-picked before any subagent was known. */
  private floorPending = false;
  /** Handoffs of the delta being processed; cleared when the window is hidden. */
  pendingHandoffs: Handoff[] = [];
  error: string | null = null;

  load(snap: Snapshot): void {
    this.sessions.clear();
    for (const s of snap.sessions) this.sessions.set(s.id, s);
    this.error = snap.error;
    if (!this.selectedId || !this.sessions.has(this.selectedId)) {
      this.selectedId = this.sorted()[0]?.id ?? null;
      this.selectedFloor = null;
    }
    this.ensureFloor();
  }

  /** The team whose agents were active most recently; "general" when there are no subagents. */
  autoFloor(s: SessionView | null): string {
    let best: string | null = null;
    let at = -1;
    for (const a of s?.agents ?? []) {
      if (a.id === "main" || !a.role.team) continue;
      if (a.lastActivityAt > at) {
        at = a.lastActivityAt;
        best = a.role.team;
      }
    }
    return best ?? "general";
  }

  /** Picks a floor when none is chosen; a chosen floor stays across deltas. */
  ensureFloor(): void {
    const s = this.selected();
    const has = !!s?.agents.some((a) => a.id !== "main");
    if (!this.selectedFloor || (this.floorPending && has)) {
      this.selectedFloor = this.autoFloor(s);
      this.floorPending = !has;
    }
  }

  selectFloor(team: string): void {
    this.selectedFloor = team;
    this.floorPending = false;
  }

  selectBuilding(id: string): void {
    if (this.selectedId === id) return;
    this.selectedId = id;
    this.selectedFloor = null;
    this.ensureFloor();
  }

  /** Newest first. */
  sorted(): SessionView[] {
    return [...this.sessions.values()].sort(
      (a, b) => b.startedAt - a.startedAt || a.id.localeCompare(b.id),
    );
  }

  selected(): SessionView | null {
    return this.selectedId ? (this.sessions.get(this.selectedId) ?? null) : null;
  }

  apply(d: Delta): Changes {
    const ch: Changes = {
      sessions: new Set(),
      agents: new Map(),
      removedAgents: new Map(),
      removedSessions: new Set(),
      structural: false,
    };
    for (const id of d.sessionsRemoved) {
      if (this.sessions.delete(id)) {
        ch.removedSessions.add(id);
        ch.structural = true;
      }
    }
    for (const h of d.sessionsUpserted) {
      const prev = this.sessions.get(h.id);
      if (!prev) ch.structural = true;
      this.sessions.set(h.id, { ...h, agents: prev ? prev.agents : [] });
      ch.sessions.add(h.id);
    }
    for (const [sid, a] of d.agentsUpserted) {
      const s = this.sessions.get(sid);
      if (!s) continue;
      const i = s.agents.findIndex((x) => x.id === a.id);
      if (i >= 0) s.agents[i] = a;
      else s.agents.push(a);
      ch.sessions.add(sid);
      let set = ch.agents.get(sid);
      if (!set) ch.agents.set(sid, (set = new Set()));
      set.add(a.id);
    }
    for (const [sid, aid] of d.agentsRemoved) {
      const s = this.sessions.get(sid);
      if (!s) continue;
      s.agents = s.agents.filter((x) => x.id !== aid);
      ch.sessions.add(sid);
      let set = ch.removedAgents.get(sid);
      if (!set) ch.removedAgents.set(sid, (set = new Set()));
      set.add(aid);
    }
    if (this.selectedId && !this.sessions.has(this.selectedId)) {
      this.selectedId = this.sorted()[0]?.id ?? null;
      this.selectedFloor = null;
    } else if (!this.selectedId) {
      this.selectedId = this.sorted()[0]?.id ?? null;
      this.selectedFloor = null;
    }
    this.ensureFloor();
    this.pendingHandoffs = d.handoffs ?? [];
    return ch;
  }

  agent(sid: string, aid: string): AgentView | undefined {
    return this.sessions.get(sid)?.agents.find((a) => a.id === aid);
  }
}
