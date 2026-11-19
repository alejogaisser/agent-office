import * as THREE from "three";
import { BUBBLE } from "./palette";
import type { Activity, AgentView, RoleInfo } from "../types";
import { WORKING } from "../types";

// Seven shared geometries: head, hair cap, torso, arm (x2), leg (x2).
const headGeo = new THREE.IcosahedronGeometry(0.17, 0);
const hairGeo = new THREE.BoxGeometry(0.3, 0.1, 0.3);
const torsoGeo = new THREE.BoxGeometry(0.4, 0.5, 0.22);
const armGeo = new THREE.BoxGeometry(0.1, 0.45, 0.1).translate(0, -0.2, 0);
const legGeo = new THREE.BoxGeometry(0.14, 0.5, 0.14).translate(0, -0.25, 0);
const parcelGeo = new THREE.BoxGeometry(0.34, 0.26, 0.26);

const SKIN = [0xf1c9a5, 0xe0ac81, 0xc68b5c, 0x9b6a45, 0x6f4a31];
const HAIR = [0x2b2118, 0x5a3d26, 0x8a5a2b, 0xd2a85a, 0x1d1d22, 0x7d3b2e];
const PANTS = 0x3a4252;

function fnv1a(s: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h >>> 0;
}

const mats = new Map<number, THREE.MeshLambertMaterial>();
function material(color: number): THREE.MeshLambertMaterial {
  let m = mats.get(color);
  if (!m) {
    m = new THREE.MeshLambertMaterial({ color, flatShading: true });
    mats.set(color, m);
  }
  return m;
}

export type Pose = "sit" | "stand" | "walk" | "sleep" | "done";

export interface Person {
  id: string;
  root: THREE.Group;
  rig: THREE.Group;
  head: THREE.Mesh;
  torso: THREE.Mesh;
  armL: THREE.Group;
  armR: THREE.Group;
  legL: THREE.Group;
  legR: THREE.Group;
  parcel: THREE.Mesh;
  meshes: THREE.Mesh[];
  bubble: HTMLDivElement | null;
  pose: Pose;
  working: boolean;
  activity: Activity;
  /** Monitor mesh of the desk this person sits at (activity color), if any. */
  monitor: THREE.Mesh | null;
  /** True while a walk owns the person's transform. */
  busy: boolean;
  /** True while the person is seated at a desk. */
  seated: boolean;
}

export function roleColor(role: RoleInfo): number {
  return parseInt(role.color.replace("#", ""), 16) || 0xcccccc;
}

/**
 * Low-poly person: head, hair, torso, two arms, two legs (7 meshes) plus a hidden parcel.
 * Torso and arms use the role color; skin and hair come from small palettes by hash of `seed`.
 */
export function buildPerson(role: RoleInfo, seed: string, overlay: HTMLElement | null): Person {
  const h = fnv1a(seed);
  const shirt = material(roleColor(role));
  const root = new THREE.Group();
  const rig = new THREE.Group();
  root.add(rig);

  const torso = new THREE.Mesh(torsoGeo, shirt);
  torso.position.y = 0.75;
  const head = new THREE.Mesh(headGeo, material(SKIN[h % SKIN.length]));
  head.position.y = 1.2;
  const hair = new THREE.Mesh(hairGeo, material(HAIR[(h >>> 8) % HAIR.length]));
  hair.position.set(0, 0.15, -0.01);
  head.add(hair);

  const limb = (geo: THREE.BufferGeometry, mat: THREE.Material, x: number, y: number): [THREE.Group, THREE.Mesh] => {
    const g = new THREE.Group();
    g.position.set(x, y, 0);
    const m = new THREE.Mesh(geo, mat);
    g.add(m);
    return [g, m];
  };
  const [armL, armLm] = limb(armGeo, shirt, -0.26, 0.98);
  const [armR, armRm] = limb(armGeo, shirt, 0.26, 0.98);
  const [legL, legLm] = limb(legGeo, material(PANTS), -0.1, 0.5);
  const [legR, legRm] = limb(legGeo, material(PANTS), 0.1, 0.5);
  const parcel = new THREE.Mesh(parcelGeo, material(0xb98a55));
  parcel.position.set(0, 0.85, 0.3);
  parcel.visible = false;
  rig.add(torso, head, armL, armR, legL, legR, parcel);

  const meshes = [torso, head, hair, armLm, armRm, legLm, legRm];
  for (const m of meshes) m.userData.agentId = seed;

  let bubble: HTMLDivElement | null = null;
  if (overlay) {
    bubble = document.createElement("div");
    bubble.className = "bubble";
    bubble.style.display = "none";
    overlay.appendChild(bubble);
  }
  const p: Person = {
    id: seed,
    root,
    rig,
    head,
    torso,
    armL,
    armR,
    legL,
    legR,
    parcel,
    meshes,
    bubble,
    pose: "stand",
    working: false,
    activity: "waiting",
    monitor: null,
    busy: false,
    seated: false,
  };
  setPose(p, "stand");
  return p;
}

export function setPose(p: Person, pose: Pose): void {
  p.pose = pose;
  p.head.rotation.set(0, 0, 0);
  p.head.position.set(0, 1.2, 0);
  p.armL.rotation.set(0, 0, 0);
  p.armR.rotation.set(0, 0, 0);
  p.legL.rotation.set(0, 0, 0);
  p.legR.rotation.set(0, 0, 0);
  p.rig.position.y = 0;
  if (pose === "sit" || pose === "sleep") {
    p.legL.rotation.x = -Math.PI / 2;
    p.legR.rotation.x = -Math.PI / 2;
    p.armL.rotation.x = -1.0;
    p.armR.rotation.x = -1.0;
    p.rig.position.y = -0.02;
  }
  if (pose === "sleep") {
    p.head.rotation.x = 0.9;
    p.head.position.set(0, 1.12, 0.1);
    p.armL.rotation.x = -0.2;
    p.armR.rotation.x = -0.2;
  }
}

/** Walk cycle: legs and arms swing with a sine of the traveled distance. */
export function swing(p: Person, phase: number): void {
  const s = Math.sin(phase) * 0.7;
  p.legL.rotation.x = s;
  p.legR.rotation.x = -s;
  p.armL.rotation.x = p.parcel.visible ? -1.2 : -s;
  p.armR.rotation.x = p.parcel.visible ? -1.2 : s;
}

/** Bubble, working flag and pose for the agent's current activity. */
export function setActivity(p: Person, a: AgentView, atDesk: boolean): void {
  p.activity = a.activity;
  p.working = WORKING.has(a.activity);
  if (!p.busy) {
    p.seated = atDesk;
    setPose(p, !atDesk ? "stand" : a.activity === "sleeping" ? "sleep" : "sit");
    if (atDesk && a.activity === "delegating") p.armR.rotation.x = -2.6;
  }
  if (p.bubble) {
    const text = BUBBLE[a.activity];
    p.bubble.textContent = text;
    p.bubble.style.display = text ? "block" : "none";
  }
}

/** Arm typing at the keyboard while seated and working. */
export function animateTyping(p: Person, now: number): void {
  if (p.busy || p.pose !== "sit" || !p.working || p.activity === "delegating") return;
  const s = Math.sin(now / 110) * 0.12;
  p.armL.rotation.x = -1.0 + s;
  p.armR.rotation.x = -1.0 - s;
}

export function disposePerson(p: Person): void {
  p.root.removeFromParent();
  p.bubble?.remove();
}
