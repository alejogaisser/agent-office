import * as THREE from "three";
import { buildFloorScene, screenMaterial, type FloorScene, type Light } from "./floorplan";
import { Labels } from "./labels";
import { animateTyping, buildPerson, disposePerson, setActivity, type Person } from "./person";
import { camera, fitToBox, requestRender, scene } from "./renderer";
import { popIn } from "./tweens";
import { cancelAll, canWalk, routeFor, seatOf, walk, walkers, type Spot } from "./walk";
import type { AgentView, Catalog, Cell, Handoff, RoleInfo, SessionView, TeamSpec } from "../types";
import { WORKING } from "../types";

interface Entry {
  person: Person;
  room: string;
  slot: number;
}

/**
 * Pure desk assignment for one room. Agents keep their previous slot when it is still free;
 * new ones take the lowest free slot in `startedAt` order. Slots: desks first, then standing
 * spots, then the room center (overflow). Never throws, whatever the head count.
 */
export function assignDesks(agents: AgentView[], prev: Map<string, number> = new Map()): Map<string, number> {
  const sorted = [...agents].sort((a, b) => a.startedAt - b.startedAt || a.id.localeCompare(b.id));
  const out = new Map<string, number>();
  const used = new Set<number>();
  for (const a of sorted) {
    const s = prev.get(a.id);
    if (s !== undefined && !used.has(s)) {
      out.set(a.id, s);
      used.add(s);
    }
  }
  let next = 0;
  for (const a of sorted) {
    if (out.has(a.id)) continue;
    while (used.has(next)) next++;
    out.set(a.id, next);
    used.add(next);
  }
  return out;
}

interface Placement {
  x: number;
  z: number;
  yaw: number;
  atDesk: boolean;
}

function doorIsSouth(cell: Cell): boolean {
  return Math.abs(cell.door[1] - (cell.rect.z + cell.rect.d)) < 0.01;
}

function placement(cell: Cell, slot: number): Placement {
  const nd = cell.desks.length;
  const faceDoor = doorIsSouth(cell) ? 0 : Math.PI;
  if (slot < nd) {
    const s = seatOf(cell, slot);
    return { x: s.x, z: s.z, yaw: s.yaw, atDesk: true };
  }
  const k = slot - nd;
  if (k < cell.standing.length) {
    const [x, z] = cell.standing[k];
    return { x, z, yaw: faceDoor, atDesk: false };
  }
  const n = k - cell.standing.length;
  return {
    x: cell.rect.x + cell.rect.w / 2 + ((n % 3) - 1) * 0.7,
    z: cell.rect.z + cell.rect.d / 2 + Math.floor(n / 3) * 0.6,
    yaw: faceDoor,
    atDesk: false,
  };
}

function spotOf(cell: Cell, slot: number): Spot {
  const nd = cell.desks.length;
  if (slot < nd) return { desk: [cell.key, slot] };
  if (slot - nd < cell.standing.length) return { stand: [cell.key, slot - nd] };
  return { door: cell.key };
}

export class OfficeScene {
  sessionId: string | null = null;
  floorKey: string | null = null;
  private catalog: Catalog | null = null;
  private team: TeamSpec | null = null;
  private fs: FloorScene | null = null;
  private session: SessionView | null = null;
  private people = new Map<string, Entry>();
  private slots = new Map<string, number>();
  private couriers = new Set<Person>();
  private pending = new Set<string>();
  private popNext = new Set<string>();
  private gen = 0;
  private labels: Labels;
  private sign: HTMLElement;
  private overlay: HTMLElement;
  private tmp = new THREE.Vector3();
  private courierSeq = 0;

  constructor(overlay: HTMLElement, sign: HTMLElement) {
    this.overlay = overlay;
    this.sign = sign;
    this.labels = new Labels(overlay);
  }

  setCatalog(c: Catalog): void {
    this.catalog = c;
  }

  teamSpec(key: string | null): TeamSpec | null {
    return this.catalog?.teams.find((t) => t.key === key) ?? null;
  }

  private clear(): void {
    cancelAll();
    this.gen++;
    for (const e of this.people.values()) disposePerson(e.person);
    for (const c of this.couriers) disposePerson(c);
    this.people.clear();
    this.couriers.clear();
    this.slots.clear();
    this.pending.clear();
    this.popNext.clear();
    this.labels.clear();
    this.fs?.dispose();
    this.fs = null;
  }

  /** Full rebuild for a (newly selected) building and floor. */
  buildFloor(s: SessionView | null, floor: string | null): void {
    this.clear();
    this.session = s;
    this.sessionId = s?.id ?? null;
    this.floorKey = floor;
    this.team = this.teamSpec(floor);
    if (!s || !this.team) {
      this.updateSign(null);
      requestRender();
      return;
    }
    this.fs = buildFloorScene(this.team);
    scene.add(this.fs.group);
    this.labels.build(this.team.plan.cells);
    fitToBox(this.fs.box);
    this.sync(s);
    this.updateSign(s);
    requestRender();
  }

  /** Applies a delta: handoffs first (they reserve spawn slots and busy walkers), then the diff. */
  applyDelta(s: SessionView, handoffs: Handoff[]): void {
    this.session = s;
    if (!this.team || !this.fs) return;
    const jobs: (() => Promise<void>)[] = [];
    for (const h of handoffs) {
      if (h.sessionId !== s.id) continue;
      const job = this.prepare(h, s);
      if (job) jobs.push(job);
    }
    this.sync(s);
    this.updateSign(s);
    for (const j of jobs) void j();
    requestRender();
  }

  // ---- state sync ---------------------------------------------------------

  private roomCell(room: string): Cell | undefined {
    const cells = this.team!.plan.cells;
    return cells.find((c) => c.key === room) ?? cells.find((c) => c.key === `${this.team!.key}-other`);
  }

  private bossCell(): Cell {
    return this.team!.plan.cells.find((c) => c.kind === "boss")!;
  }

  /** Brings people, monitors, lights and labels in line with the session state. */
  private sync(s: SessionView): void {
    const team = this.team;
    const fs = this.fs;
    if (!team || !fs) return;
    const byRoom = new Map<string, AgentView[]>();
    const doneByRoom = new Map<string, number>();
    const occupants = new Map<string, { agent: AgentView; cell: Cell }>();
    for (const a of s.agents) {
      if (a.id === "main" || a.role.team !== team.key) continue;
      const cell = this.roomCell(a.role.room);
      if (!cell) continue;
      if (a.activity === "done") {
        doneByRoom.set(cell.key, (doneByRoom.get(cell.key) ?? 0) + 1);
        continue;
      }
      occupants.set(a.id, { agent: a, cell });
      let list = byRoom.get(cell.key);
      if (!list) byRoom.set(cell.key, (list = []));
      list.push(a);
    }
    const main = s.agents.find((a) => a.id === "main");
    const mainHere = !!main && s.activeTeams.includes(team.key);
    const boss = this.bossCell();
    if (main && mainHere) occupants.set("main", { agent: main, cell: boss });

    // desks
    for (const list of byRoom.values()) {
      const prev = new Map<string, number>();
      for (const a of list) {
        const p = this.slots.get(a.id);
        if (p !== undefined) prev.set(a.id, p);
      }
      for (const [id, slot] of assignDesks(list, prev)) this.slots.set(id, slot);
    }
    if (main && mainHere) this.slots.set("main", 0);

    // people
    for (const [id, e] of [...this.people]) {
      if (occupants.has(id) || e.person.busy) continue;
      disposePerson(e.person);
      this.people.delete(id);
      this.slots.delete(id);
    }
    for (const [id, { agent, cell }] of occupants) {
      if (this.pending.has(id)) continue;
      const slot = this.slots.get(id) ?? 0;
      let e = this.people.get(id);
      if (!e) {
        const person = buildPerson(agent.role, id, this.overlay);
        scene.add(person.root);
        e = { person, room: cell.key, slot };
        this.people.set(id, e);
        if (this.popNext.delete(id)) popIn(person.root, 300);
      } else {
        e.room = cell.key;
        e.slot = slot;
      }
      const pl = placement(cell, slot);
      if (!e.person.busy) {
        e.person.root.position.set(pl.x, 0, pl.z);
        e.person.root.rotation.y = pl.yaw;
      }
      setActivity(e.person, agent, pl.atDesk);
    }

    // monitors
    for (const o of fs.cells.values()) for (const m of o.monitors) m.material = screenMaterial("off");
    for (const [id, e] of this.people) {
      const o = fs.cells.get(e.room);
      const occ = occupants.get(id);
      const mon = o?.monitors[e.slot];
      if (mon && occ && o && e.slot < o.cell.desks.length) mon.material = screenMaterial(occ.agent.activity);
    }

    // lights and tallies
    for (const o of fs.cells.values()) {
      const c = o.cell;
      if (c.kind !== "role" && c.kind !== "other" && c.kind !== "boss") continue;
      let light: Light = "dark";
      if (c.kind === "boss") {
        if (main && mainHere) light = WORKING.has(main.activity) ? "lit" : "dim";
      } else {
        const list = byRoom.get(c.key) ?? [];
        if (list.some((a) => WORKING.has(a.activity))) light = "lit";
        else if (list.length > 0) light = "dim";
      }
      fs.setRoomLight(c.key, light);
      this.labels.setTally(c.key, doneByRoom.get(c.key) ?? 0);
    }
    this.updateSign(s); // also refreshes the head count after walks end
  }

  // ---- handoff animations -----------------------------------------------

  /** Reserves what the handoff needs and returns the animation to run after the sync. */
  private prepare(h: Handoff, s: SessionView): (() => Promise<void>) | null {
    const team = this.team!;
    const plan = team.plan;
    const gen = this.gen;
    const alive = (): boolean => gen === this.gen;
    const boss = this.bossCell();

    if (h.kind === "spawn") {
      const agent = s.agents.find((a) => a.id === h.toAgentId);
      const relay = h.relayFromAgentId ? s.agents.find((a) => a.id === h.relayFromAgentId) : undefined;
      const relayTeam = relay?.role.team ?? null;
      const down = relayTeam && h.toTeam ? this.floorOf(h.toTeam) < this.floorOf(relayTeam) : false;

      if (relay && relayTeam === team.key && h.toTeam !== team.key) {
        // from-floor: a courier carries a box from the room to the stairs and leaves
        const from = this.roomCell(relay.role.room);
        if (!from || !canWalk()) return null;
        return async () => {
          const c = this.makeCourier(s);
          await walk(c, routeFor(plan, { desk: [from.key, 0] }, "stairs"), {
            carry: true,
            endAction: down ? "sink" : "vanish",
          });
          this.dropCourier(c);
        };
      }
      if (h.toTeam !== team.key || !agent) return null;
      const room = this.roomCell(agent.role.room)!;
      this.pending.add(agent.id);
      this.popNext.add(agent.id);
      const arrive = (): void => {
        if (!alive()) return;
        this.pending.delete(agent.id);
        if (this.session) this.sync(this.session);
        requestRender();
      };
      if (!canWalk()) {
        // over the walker cap: skip the walk and jump to the end state
        this.pending.delete(agent.id);
        return null;
      }
      if (relay) {
        // to-floor: a courier rises from the stairs and walks to the room door
        return async () => {
          const c = this.makeCourier(s);
          await walk(c, routeFor(plan, "stairs", { door: room.key }), { carry: true, endAction: "rise" });
          if (!alive()) return;
          arrive();
          this.dropCourier(c);
        };
      }
      // main walks to the room door, the new agent appears, main walks back
      const mainEntry = this.people.get("main");
      return async () => {
        const useMain = !!mainEntry && !mainEntry.person.busy;
        const actor = useMain ? mainEntry!.person : this.makeCourier(s);
        if (useMain) actor.busy = true;
        await walk(actor, routeFor(plan, "boss", { door: room.key }), { endAction: useMain ? undefined : "vanish" });
        if (!alive()) return;
        actor.busy = true; // keep the sync from seating main while it walks back
        arrive();
        if (useMain) {
          const seat = seatOf(boss, 0);
          await walk(actor, routeFor(plan, { door: room.key }, "boss"), { endAction: "sit", finalYaw: seat.yaw });
          if (!alive()) return;
          if (this.session) this.sync(this.session);
        } else {
          this.dropCourier(actor);
        }
        requestRender();
      };
    }

    // return: the agent walks to the boss door (or the stairs when the parent is on another floor)
    const e = this.people.get(h.fromAgentId);
    if (!e || e.person.busy || h.fromTeam !== team.key) return null;
    const cell = this.roomCell(e.room);
    if (!cell || !canWalk()) return null;
    const person = e.person;
    person.busy = true;
    if (person.bubble) person.bubble.style.display = "none";
    const toStairs = h.toTeam !== null && h.toTeam !== team.key;
    return async () => {
      const route = routeFor(plan, spotOf(cell, e.slot), toStairs ? "stairs" : { door: "boss" });
      await walk(person, route, { endAction: toStairs ? "sink" : "vanish" });
      if (!alive()) return;
      disposePerson(person);
      this.people.delete(h.fromAgentId);
      this.slots.delete(h.fromAgentId);
      if (this.session) this.sync(this.session);
      requestRender();
    };
  }

  private floorOf(team: string): number {
    return this.catalog?.teams.find((t) => t.key === team)?.floor ?? 0;
  }

  private makeCourier(s: SessionView): Person {
    const main = s.agents.find((a) => a.id === "main");
    const role: RoleInfo = main?.role ?? { key: "main", label: "Main", color: "#F2C14E", team: null, room: "boss" };
    const c = buildPerson(role, `courier-${this.courierSeq++}`, null);
    c.busy = true;
    for (const m of c.meshes) m.userData.agentId = "main";
    scene.add(c.root);
    this.couriers.add(c);
    return c;
  }

  private dropCourier(c: Person): void {
    this.couriers.delete(c);
    disposePerson(c);
    requestRender();
  }

  // ---- queries and per-frame ---------------------------------------------

  private updateSign(s: SessionView | null): void {
    if (!s || !this.team) {
      this.sign.textContent = "";
      this.sign.hidden = true;
      return;
    }
    this.sign.hidden = false;
    const extra = s.archivedDone > 0 ? ` +${s.archivedDone} archived` : "";
    const n = this.people.size;
    this.sign.textContent = `${s.title}  ·  ${s.project}  ·  floor ${this.team.floor} ${this.team.label}  ·  ${n} ${n === 1 ? "person" : "people"}${extra}`;
  }

  hasWorking(): boolean {
    for (const e of this.people.values()) if (e.person.working) return true;
    return false;
  }

  pickables(): THREE.Object3D[] {
    const out: THREE.Object3D[] = [];
    for (const e of this.people.values()) out.push(...e.person.meshes);
    for (const c of this.couriers) out.push(...c.meshes);
    return out;
  }

  repositionLabels(width: number, height: number): void {
    this.labels.reposition(width, height);
  }

  debug(): { floor: string; lit: number; dim: number; dark: number; walkers: number; cells: number } {
    const n = this.fs?.lightCounts() ?? { lit: 0, dim: 0, dark: 0 };
    return { floor: this.floorKey ?? "-", ...n, walkers: walkers(), cells: this.labels.count() };
  }

  /** Per-frame: typing arms and speech bubble positions. */
  frame(now: number, width: number, height: number): void {
    for (const e of this.people.values()) {
      const p = e.person;
      animateTyping(p, now);
      if (!p.bubble || p.bubble.style.display === "none") continue;
      p.root.updateWorldMatrix(true, false);
      this.tmp.set(0, 1.75, 0);
      p.root.localToWorld(this.tmp);
      this.tmp.project(camera);
      p.bubble.style.transform = `translate(${((this.tmp.x + 1) / 2) * width}px, ${((1 - this.tmp.y) / 2) * height}px) translate(-50%, -100%)`;
    }
  }
}
