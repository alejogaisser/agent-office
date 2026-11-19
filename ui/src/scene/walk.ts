import * as THREE from "three";
import { localToWorld } from "./floorplan";
import { setPose, swing, type Person } from "./person";
import { runTween } from "./renderer";
import { killTween, type Tween } from "./tweens";
import type { Cell, FloorPlan } from "../types";

export const MAX_WALKERS = 6;
const SPEED = 3; // units per second
const MAX_LEG_S = 3.5; // a whole walk takes at most this long
const SEAT_LZ = 1.45;
const DESK_LX = [1.2, 3.0, 4.8];

export type Spot =
  | { desk: [string, number] }
  | { stand: [string, number] }
  | { door: string }
  | "boss"
  | "stairs";

export interface WalkOpts {
  carry?: boolean;
  endAction?: "sink" | "rise" | "vanish" | "sit";
  /** Yaw to take when sitting down. */
  finalYaw?: number;
}

/** Shortest node path between two graph nodes (BFS over the plan's edges). */
export function bfs(plan: FloorPlan, from: number, to: number): number[] {
  if (from === to) return [from];
  const adj: number[][] = plan.nodes.map(() => []);
  for (const [a, b] of plan.edges) {
    adj[a].push(b);
    adj[b].push(a);
  }
  const prev = new Map<number, number>([[from, -1]]);
  const q = [from];
  for (let i = 0; i < q.length; i++) {
    const n = q[i];
    if (n === to) break;
    for (const k of adj[n]) {
      if (!prev.has(k)) {
        prev.set(k, n);
        q.push(k);
      }
    }
  }
  if (!prev.has(to)) return [from, to];
  const path: number[] = [];
  for (let n = to; n !== -1; n = prev.get(n) ?? -1) path.push(n);
  return path.reverse();
}

function cellOf(plan: FloorPlan, key: string): Cell {
  const c = plan.cells.find((x) => x.key === key);
  if (!c) throw new Error(`no cell ${key}`);
  return c;
}

/** Where the person sits (world x, z, yaw) for a desk slot. */
export function seatOf(c: Cell, i: number): { x: number; z: number; yaw: number } {
  const d = c.desks[i] ?? c.desks[0];
  return { x: d[0] - Math.sin(d[2]) * 0.85, z: d[1] - Math.cos(d[2]) * 0.85, yaw: d[2] };
}

/** Stairs landing (world): next to the well; people sink into it and rise out of it. */
export function stairsSpot(c: Cell): [number, number] {
  return [c.rect.x + 4.4, c.rect.z + 3.0];
}

/** A point just inside the cell, in front of its door. */
function inside(c: Cell, off: number): [number, number] {
  const south = Math.abs(c.door[1] - (c.rect.z + c.rect.d)) < 0.01;
  return [c.door[0], c.door[1] + (south ? -off : off)];
}

/** Interior waypoints (world) from a spot inside a cell to its door, and the door node. */
function interior(plan: FloorPlan, s: Spot): { pts: [number, number][]; node: number } {
  if (s === "stairs") {
    const c = plan.cells.find((x) => x.kind === "stairs")!;
    return { pts: [stairsSpot(c), inside(c, 0.6)], node: c.doorNode };
  }
  if (s === "boss") return interior(plan, { desk: ["boss", 0] });
  if ("door" in s) {
    const c = cellOf(plan, s.door);
    return { pts: [], node: c.doorNode };
  }
  if ("stand" in s) {
    const c = cellOf(plan, s.stand[0]);
    const st = c.standing[s.stand[1]] ?? c.standing[0];
    return { pts: [st, inside(c, 0.6)], node: c.doorNode };
  }
  const c = cellOf(plan, s.desk[0]);
  const lx = c.kind === "boss" ? 6 : (DESK_LX[s.desk[1]] ?? 3);
  const side = lx < c.rect.w / 2 ? 0.3 : c.rect.w - 0.3;
  const w = c.rect.w;
  const loc: [number, number][] = [
    [lx, SEAT_LZ],
    [lx, 0.75],
    [side, 0.75],
    [side, 4.6],
    [w / 2, 5.5],
  ];
  const pts = loc.map(([x, z]) => localToWorld(c, x, z));
  return { pts, node: c.doorNode };
}

/** Route desk -> door -> graph path -> door -> desk as world points (y = 0). */
export function routeFor(plan: FloorPlan, from: Spot, to: Spot): THREE.Vector3[] {
  const a = interior(plan, from);
  const b = interior(plan, to);
  const out: [number, number][] = [...a.pts];
  for (const n of bfs(plan, a.node, b.node)) out.push(plan.nodes[n]);
  out.push(...[...b.pts].reverse());
  const pts: THREE.Vector3[] = [];
  for (const [x, z] of out) {
    const last = pts[pts.length - 1];
    if (last && Math.hypot(last.x - x, last.z - z) < 0.02) continue;
    pts.push(new THREE.Vector3(x, 0, z));
  }
  return pts;
}

interface Walk {
  person: Person;
  tween: Tween | null;
  finish: () => void;
}

const active = new Set<Walk>();

export function walkers(): number {
  return active.size;
}

export function canWalk(): boolean {
  return active.size < MAX_WALKERS;
}

/** Cancels every running walk and applies its end state (used on floor switches). */
export function cancelAll(): void {
  for (const w of [...active]) w.finish();
}

function endState(p: Person, action: WalkOpts["endAction"], yaw: number | undefined): void {
  p.parcel.visible = false;
  p.busy = false;
  if (action === "sink") {
    p.root.scale.setScalar(0.001);
    p.root.position.y = -1.2;
  } else if (action === "vanish") {
    p.root.scale.setScalar(0.001);
  } else if (action === "sit") {
    setPose(p, "sit");
    if (yaw !== undefined) p.root.rotation.y = yaw;
    p.seated = true;
  } else {
    setPose(p, "stand");
  }
}

/**
 * Moves a person along the route at 3 units/s (sped up so the whole walk takes at most 3.5 s),
 * swinging limbs and facing the direction of travel. rAF runs only through the tween.
 */
export function walk(p: Person, route: THREE.Vector3[], opts: WalkOpts = {}): Promise<void> {
  return new Promise<void>((resolve) => {
    if (route.length < 2 || active.size >= MAX_WALKERS) {
      if (route.length) p.root.position.copy(route[route.length - 1]);
      endState(p, opts.endAction, opts.finalYaw);
      resolve();
      return;
    }
    const lens: number[] = [];
    let total = 0;
    for (let i = 1; i < route.length; i++) {
      const l = route[i].distanceTo(route[i - 1]);
      lens.push(l);
      total += l;
    }
    const speed = Math.max(SPEED, total / MAX_LEG_S);
    const ms = (total / speed) * 1000;
    const w: Walk = { person: p, tween: null, finish: () => undefined };
    let finished = false;
    const finish = (): void => {
      if (finished) return;
      finished = true;
      if (w.tween) killTween(w.tween);
      active.delete(w);
      p.root.position.copy(route[route.length - 1]);
      p.root.position.y = 0;
      p.root.scale.setScalar(1);
      endState(p, opts.endAction, opts.finalYaw);
      resolve();
    };
    w.finish = finish;
    active.add(w);

    const startMoving = (): void => {
      if (finished) return;
      setPose(p, "walk");
      p.busy = true;
      p.parcel.visible = !!opts.carry;
      p.root.position.copy(route[0]);
      w.tween = runTween(
        ms,
        (t) => {
          let d = t * total;
          let i = 0;
          while (i < lens.length - 1 && d > lens[i]) {
            d -= lens[i];
            i++;
          }
          const k = lens[i] > 0 ? Math.min(1, d / lens[i]) : 1;
          p.root.position.lerpVectors(route[i], route[i + 1], k);
          const dx = route[i + 1].x - route[i].x;
          const dz = route[i + 1].z - route[i].z;
          if (dx || dz) p.root.rotation.y = Math.atan2(dx, dz);
          swing(p, t * total * 4.5);
        },
        () => {
          if (finished) return;
          if (opts.endAction === "sink" || opts.endAction === "vanish") {
            const sink = opts.endAction === "sink";
            const y0 = p.root.position.y;
            setPose(p, "stand");
            w.tween = runTween(
              sink ? 400 : 200,
              (u) => {
                p.root.scale.setScalar(Math.max(0.001, 1 - u));
                p.root.position.y = y0 - (sink ? 1.2 * u : 0);
              },
              finish,
            );
          } else {
            finish();
          }
        },
      );
    };

    if (opts.endAction === "rise") {
      // rise from the stairs first (reverse of sink), then walk
      p.busy = true;
      p.root.position.copy(route[0]);
      p.root.position.y = -1.2;
      p.root.scale.setScalar(0.001);
      w.tween = runTween(
        400,
        (u) => {
          p.root.scale.setScalar(Math.max(0.001, u));
          p.root.position.y = -1.2 * (1 - u);
        },
        () => {
          p.root.position.y = 0;
          p.root.scale.setScalar(1);
          // after rising, a plain walk that ends in a stand pose
          opts = { ...opts, endAction: undefined };
          startMoving();
        },
      );
    } else {
      startMoving();
    }
  });
}
