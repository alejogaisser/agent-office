import * as THREE from "three";
import { MONITOR_COLOR } from "./palette";
import { Mesher, P, chair, desk, stairs } from "./props";
import { THEMES } from "./themes";
import type { Activity, Cell, Rect, TeamSpec } from "../types";

export type Light = "lit" | "dim" | "dark";

const LOCAL_DESK_X = [1.2, 3.0, 4.8];
const WALL_H = 0.7;
const WALL_T = 0.15;
const DOOR_W = 1.4;
const TOP = 2.4;

// Three shared materials: the room light state is a material swap (no per-room lights).
function lightMat(factor: number): THREE.MeshLambertMaterial {
  const m = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true });
  m.color.setScalar(factor);
  return m;
}
const MATS: Record<Light, THREE.MeshLambertMaterial> = { lit: lightMat(1), dim: lightMat(0.6), dark: lightMat(0.28) };

const screenGeo = new THREE.BoxGeometry(0.6, 0.38, 0.04);
const bulbGeo = new THREE.IcosahedronGeometry(0.13, 0);
const bulbMat = new THREE.MeshBasicMaterial({ color: 0xffe08a });
const screenMats = new Map<string, THREE.MeshBasicMaterial>();

/** Screen material: "off" or an activity color. */
export function screenMaterial(a: Activity | "off"): THREE.MeshBasicMaterial {
  let m = screenMats.get(a);
  if (!m) {
    m = new THREE.MeshBasicMaterial({ color: a === "off" ? 0x20242c : MONITOR_COLOR[a] });
    screenMats.set(a, m);
  }
  return m;
}

function mix(a: number, b: number, t: number): number {
  return new THREE.Color(a).lerp(new THREE.Color(b), t).getHex();
}

function doorSouth(c: Cell): boolean {
  return Math.abs(c.door[1] - (c.rect.z + c.rect.d)) < 0.01;
}

/** Cell-local (door on +z) to world. */
export function localToWorld(c: Cell, lx: number, lz: number): [number, number] {
  return doorSouth(c) ? [c.rect.x + lx, c.rect.z + lz] : [c.rect.x + c.rect.w - lx, c.rect.z + c.rect.d - lz];
}

export interface CellObj {
  cell: Cell;
  mesh: THREE.Mesh;
  light: Light;
  bulb: THREE.Mesh | null;
  monitors: THREE.Mesh[];
}

export interface FloorScene {
  team: TeamSpec;
  group: THREE.Group;
  box: THREE.Box3;
  cells: Map<string, CellObj>;
  corridors: THREE.Mesh;
  setRoomLight(key: string, light: Light): void;
  /** Desk monitor mesh (activity color); the mesh stays, only its material changes. */
  monitor(cellKey: string, i: number): THREE.Mesh | undefined;
  /** Counts of lit/dim/dark among the boss and role rooms. */
  lightCounts(): { lit: number; dim: number; dark: number };
  dispose(): void;
}

function walls(m: Mesher, w: number, d: number, color: number): void {
  const h = WALL_H;
  m.box(WALL_T / 2, 0, d / 2, WALL_T, h, d, color);
  m.box(w - WALL_T / 2, 0, d / 2, WALL_T, h, d, color);
  m.box(w / 2, 0, WALL_T / 2, w, h, WALL_T, color);
  const gap = DOOR_W / 2;
  const left = w / 2 - gap;
  m.box(left / 2, 0, d - WALL_T / 2, left, h, WALL_T, color);
  m.box(w - left / 2, 0, d - WALL_T / 2, left, h, WALL_T, color);
  // door frame posts
  m.box(w / 2 - gap, 0, d - WALL_T / 2, 0.1, 1.2, 0.2, P.darkWood);
  m.box(w / 2 + gap, 0, d - WALL_T / 2, 0.1, 1.2, 0.2, P.darkWood);
}

function buildCell(c: Cell, color: number, floor: number): THREE.Mesh {
  const south = doorSouth(c);
  const { w, d } = c.rect;
  const m = new Mesher();
  m.setCellFrame(c.rect, south);
  m.push(0, 0);
  const theme = THEMES[c.kind === "lounge" ? "lounge" : c.kind === "stairs" ? "stairs" : c.key];
  const tint = theme?.tint ?? mix(color, 0xffffff, 0.8);
  m.box(w / 2, -0.1, d / 2, w, 0.1, d, tint);
  walls(m, w, d, mix(color, 0xb8c0cc, 0.45));
  theme?.props(m);
  if (c.kind === "stairs") stairs(m, floor < 5, floor > 1);
  if (c.kind === "role" || c.kind === "other" || c.kind === "boss") {
    const xs = c.kind === "boss" ? [6] : LOCAL_DESK_X;
    for (const x of xs) {
      desk(m, x, 2.3, 0, c.kind === "boss" ? P.darkWood : P.wood);
      chair(m, x, 1.45, 0, c.kind === "boss" ? 0x5a3a2a : P.navy);
    }
    // floor lamp in the near-left corner; its bulb is a separate mesh toggled by the light state
    m.push(0, 0);
    m.cyl(0.4, 0, 0.45, 0.14, 0.04, P.dark);
    m.cyl(0.4, 0.04, 0.45, 0.025, 1.3, P.metal);
  }
  const mesh = new THREE.Mesh(m.build(), MATS.dim);
  return mesh;
}

/** One merged mesh per room plus one for the corridors; people and monitors are separate. */
export function buildFloorScene(team: TeamSpec): FloorScene {
  const plan = team.plan;
  const group = new THREE.Group();
  const cells = new Map<string, CellObj>();
  const colorOf = new Map(team.rooms.map((r) => [r.key, parseInt(r.color.replace("#", ""), 16)]));

  for (const c of plan.cells) {
    const color = colorOf.get(c.key) ?? (c.kind === "boss" ? 0xf2c14e : c.kind === "stairs" ? 0x9aa3b0 : 0x7a8a7a);
    const mesh = buildCell(c, color, team.floor);
    group.add(mesh);
    const obj: CellObj = { cell: c, mesh, light: "dim", bulb: null, monitors: [] };
    if (c.kind === "role" || c.kind === "other" || c.kind === "boss") {
      const [bx, bz] = localToWorld(c, 0.4, 0.45);
      obj.bulb = new THREE.Mesh(bulbGeo, bulbMat);
      obj.bulb.position.set(bx, 1.42, bz);
      obj.bulb.visible = false;
      group.add(obj.bulb);
      c.desks.forEach(([x, z, r]) => {
        const s = new THREE.Mesh(screenGeo, screenMaterial("off"));
        s.position.set(x - Math.sin(r) * 0.2, 1.08, z - Math.cos(r) * 0.2);
        s.rotation.y = r;
        group.add(s);
        obj.monitors.push(s);
      });
    }
    cells.set(c.key, obj);
    setLight(obj, c.kind === "lounge" || c.kind === "stairs" ? "dim" : "dark");
  }

  // corridors, lane marks and the slab under the building
  const cm = new Mesher();
  cm.setOrigin(0, 0, 0);
  cm.push(0, 0);
  cm.box(plan.width / 2, -0.6, plan.depth / 2, plan.width + 0.4, 0.5, plan.depth + 0.4, 0x2a2f3a);
  for (const r of plan.corridors) {
    cm.box(r.x + r.w / 2, -0.1, r.z + r.d / 2, r.w, 0.1, r.d, 0xbac1cd);
    const horizontal = r.w > r.d;
    const len = horizontal ? r.w : r.d;
    for (let t = 1; t < len - 1; t += 2) {
      if (horizontal) cm.box(r.x + t, 0, r.z + r.d / 2, 0.8, 0.01, 0.08, 0xdfe3ea);
      else cm.box(r.x + r.w / 2, 0, r.z + t, 0.08, 0.01, 0.8, 0xdfe3ea);
    }
  }
  const corridors = new THREE.Mesh(cm.build(), MATS.lit);
  group.add(corridors);

  const box = new THREE.Box3(new THREE.Vector3(0, -0.5, 0), new THREE.Vector3(plan.width, TOP, plan.depth));

  function setLight(o: CellObj, l: Light): void {
    o.light = l;
    o.mesh.material = MATS[l];
    if (o.bulb) o.bulb.visible = l === "lit";
  }

  return {
    team,
    group,
    box,
    cells,
    corridors,
    setRoomLight(key, l) {
      const o = cells.get(key);
      if (o && o.light !== l) setLight(o, l);
    },
    monitor(key, i) {
      return cells.get(key)?.monitors[i];
    },
    lightCounts() {
      const n = { lit: 0, dim: 0, dark: 0 };
      for (const o of cells.values()) {
        if (o.cell.kind === "role" || o.cell.kind === "other" || o.cell.kind === "boss") n[o.light]++;
      }
      return n;
    },
    dispose() {
      for (const o of cells.values()) o.mesh.geometry.dispose();
      corridors.geometry.dispose();
      group.removeFromParent();
      group.clear();
    },
  };
}

export function rectCenter(r: Rect): [number, number] {
  return [r.x + r.w / 2, r.z + r.d / 2];
}
