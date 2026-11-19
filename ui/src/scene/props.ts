import * as THREE from "three";

/** Palette for props (sRGB hex). */
export const P = {
  wood: 0xa98259,
  darkWood: 0x6b4f36,
  metal: 0x7b8794,
  dark: 0x2b2f38,
  darker: 0x15181e,
  white: 0xeef1f5,
  paper: 0xf4efe4,
  green: 0x5fa35f,
  leaf: 0x7cc07c,
  pot: 0xb5653a,
  red: 0xd64545,
  lime: 0x4cd964,
  blue: 0x4f7cac,
  navy: 0x2c3e5c,
  yellow: 0xf2c94c,
  orange: 0xe07a2e,
  purple: 0x8e6bbf,
  pink: 0xe76f9a,
  teal: 0x2a9d8f,
  cloth: 0x6d7f99,
  cardboard: 0xb98a55,
};

const boxG = new THREE.BoxGeometry(1, 1, 1).toNonIndexed();
const cylG = new THREE.CylinderGeometry(1, 1, 1, 8).toNonIndexed();
const icoG = new THREE.IcosahedronGeometry(1, 0).toNonIndexed();

/**
 * Collects transformed, vertex-colored primitives into one BufferGeometry.
 * `setCellFrame` places a whole cell, `push` places one prop inside it (+z is the prop's front).
 */
export class Mesher {
  private pos: number[] = [];
  private nor: number[] = [];
  private col: number[] = [];
  private origin = new THREE.Matrix4();
  private local = new THREE.Matrix4();
  private m = new THREE.Matrix4();
  private nm = new THREE.Matrix3();
  private q = new THREE.Quaternion();
  private e = new THREE.Euler();
  private v = new THREE.Vector3();
  private s = new THREE.Vector3();
  private c = new THREE.Color();
  private out = new THREE.Vector3();

  setOrigin(x: number, z: number, ry: number): void {
    this.origin.makeRotationY(ry).setPosition(x, 0, z);
    this.local.identity();
  }

  /** Cell frame: local (0,0) is the far-left corner when the door is on the +z side. */
  setCellFrame(rect: { x: number; z: number; w: number; d: number }, doorSouth: boolean): void {
    if (doorSouth) this.setOrigin(rect.x, rect.z, 0);
    else this.setOrigin(rect.x + rect.w, rect.z + rect.d, Math.PI);
  }

  push(x: number, z: number, ry = 0): void {
    this.local.makeRotationY(ry).setPosition(x, 0, z);
  }

  private add(
    g: THREE.BufferGeometry,
    x: number,
    y: number,
    z: number,
    sx: number,
    sy: number,
    sz: number,
    color: number,
    rx: number,
    ry: number,
    rz: number,
  ): void {
    this.q.setFromEuler(this.e.set(rx, ry, rz));
    this.m.compose(this.v.set(x, y, z), this.q, this.s.set(sx, sy, sz));
    this.m.premultiply(this.local).premultiply(this.origin);
    this.nm.getNormalMatrix(this.m);
    this.c.setHex(color);
    const p = g.getAttribute("position");
    const n = g.getAttribute("normal");
    for (let i = 0; i < p.count; i++) {
      this.out.fromBufferAttribute(p, i).applyMatrix4(this.m);
      this.pos.push(this.out.x, this.out.y, this.out.z);
      this.out.fromBufferAttribute(n, i).applyMatrix3(this.nm).normalize();
      this.nor.push(this.out.x, this.out.y, this.out.z);
      this.col.push(this.c.r, this.c.g, this.c.b);
    }
  }

  /** Box with its bottom center at (x, y, z). Rotations turn it around its own center. */
  box(x: number, y: number, z: number, w: number, h: number, d: number, color: number, ry = 0, rx = 0, rz = 0): void {
    this.add(boxG, x, y + h / 2, z, w, h, d, color, rx, ry, rz);
  }

  cyl(x: number, y: number, z: number, r: number, h: number, color: number, rx = 0, rz = 0): void {
    this.add(cylG, x, y + h / 2, z, r, h, r, color, rx, 0, rz);
  }

  /** Icosahedron centered at (x, y, z). */
  ico(x: number, y: number, z: number, r: number, color: number, sy = 1): void {
    this.add(icoG, x, y, z, r, r * sy, r, color, 0, 0, 0);
  }

  build(): THREE.BufferGeometry {
    const g = new THREE.BufferGeometry();
    g.setAttribute("position", new THREE.Float32BufferAttribute(this.pos, 3));
    g.setAttribute("normal", new THREE.Float32BufferAttribute(this.nor, 3));
    g.setAttribute("color", new THREE.Float32BufferAttribute(this.col, 3));
    g.computeBoundingSphere();
    return g;
  }
}

// ---- props (local frame: +z is the front, y is up) -------------------------

export function desk(m: Mesher, x: number, z: number, ry = 0, top: number = P.wood): void {
  m.push(x, z, ry);
  m.box(0, 0.66, 0, 1.5, 0.07, 0.8, top);
  m.box(-0.68, 0, 0, 0.07, 0.66, 0.7, P.darkWood);
  m.box(0.68, 0, 0, 0.07, 0.66, 0.7, P.darkWood);
  m.box(0, 0.73, 0.15, 0.5, 0.03, 0.18, P.dark); // keyboard
  m.cyl(0, 0.73, -0.2, 0.07, 0.16, P.dark); // monitor stand
}

export function chair(m: Mesher, x: number, z: number, ry = 0, color: number = P.navy): void {
  m.push(x, z, ry);
  m.cyl(0, 0, 0, 0.22, 0.05, P.dark);
  m.cyl(0, 0.05, 0, 0.05, 0.3, P.dark);
  m.box(0, 0.34, 0, 0.5, 0.08, 0.5, color);
  m.box(0, 0.42, -0.23, 0.5, 0.5, 0.08, color);
}

export function plant(m: Mesher, x: number, z: number): void {
  m.push(x, z);
  m.cyl(0, 0, 0, 0.22, 0.3, P.pot);
  m.ico(0, 0.55, 0, 0.3, P.green);
  m.ico(0.1, 0.82, 0.05, 0.2, P.leaf);
}

export function sofa(m: Mesher, x: number, z: number, ry = 0, w = 2, color: number = P.cloth): void {
  m.push(x, z, ry);
  m.box(0, 0, 0, w, 0.4, 0.85, color);
  m.box(0, 0.4, -0.34, w, 0.45, 0.18, color);
  m.box(-(w / 2 - 0.1), 0.4, 0.05, 0.2, 0.25, 0.7, color);
  m.box(w / 2 - 0.1, 0.4, 0.05, 0.2, 0.25, 0.7, color);
}

export function armchair(m: Mesher, x: number, z: number, ry = 0, color: number = P.darkWood): void {
  sofa(m, x, z, ry, 0.9, color);
}

export function shelf(m: Mesher, x: number, z: number, ry = 0, w = 1.4): void {
  m.push(x, z, ry);
  m.box(-w / 2, 0, 0, 0.06, 1.5, 0.35, P.wood);
  m.box(w / 2, 0, 0, 0.06, 1.5, 0.35, P.wood);
  const books = [P.red, P.blue, P.yellow, P.green, P.purple, P.orange];
  for (let i = 0; i < 4; i++) {
    m.box(0, 0.05 + i * 0.4, 0, w, 0.05, 0.35, P.wood);
    for (let j = 0; j < 5; j++) {
      m.box(-w / 2 + 0.2 + j * ((w - 0.4) / 4), 0.1 + i * 0.4, 0, 0.1, 0.25 + (j % 2) * 0.05, 0.22, books[(i + j) % books.length]);
    }
  }
}

export function rack(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.box(0, 0, 0, 0.7, 1.6, 0.6, P.dark);
  for (let i = 0; i < 6; i++) {
    m.box(0, 0.15 + i * 0.24, 0.31, 0.6, 0.16, 0.03, P.darker);
    m.box(-0.2, 0.2 + i * 0.24, 0.33, 0.05, 0.05, 0.02, i % 2 ? P.lime : P.red);
    m.box(0.2, 0.2 + i * 0.24, 0.33, 0.05, 0.05, 0.02, P.blue);
  }
}

/** Board on legs (whiteboard, wireframes, checklist) with colored notes. */
export function board(
  m: Mesher,
  x: number,
  z: number,
  ry = 0,
  w = 1.6,
  h = 1.0,
  bg: number = P.white,
  notes: number[] = [P.blue, P.red, P.green],
): void {
  m.push(x, z, ry);
  m.box(-w / 2 + 0.1, 0, 0, 0.05, 0.5, 0.05, P.metal);
  m.box(w / 2 - 0.1, 0, 0, 0.05, 0.5, 0.05, P.metal);
  m.box(0, 0.5, 0, w, h, 0.06, bg);
  notes.forEach((c, i) => {
    m.box(-w / 2 + 0.25 + (i % 4) * (w / 4.4), 0.62 + Math.floor(i / 4) * 0.4 + (i % 2) * 0.1, 0.04, 0.28, 0.22, 0.02, c);
  });
}

/** Board fixed to a wall (no legs). */
export function wallBoard(
  m: Mesher,
  x: number,
  z: number,
  ry = 0,
  w = 1.6,
  h = 0.9,
  bg: number = P.white,
  notes: number[] = [],
): void {
  m.push(x, z, ry);
  m.box(0, 0.7, 0, w, h, 0.05, bg);
  notes.forEach((c, i) => {
    m.box(-w / 2 + 0.2 + (i % 5) * (w / 5.2), 0.85 + Math.floor(i / 5) * 0.35, 0.04, 0.24, 0.24, 0.02, c);
  });
}

export function table(m: Mesher, x: number, z: number, ry = 0, w = 1.6, d = 0.9, top: number = P.wood): void {
  m.push(x, z, ry);
  m.box(0, 0.66, 0, w, 0.07, d, top);
  for (const sx of [-1, 1]) for (const sz of [-1, 1]) m.box(sx * (w / 2 - 0.08), 0, sz * (d / 2 - 0.08), 0.07, 0.66, 0.07, P.darkWood);
}

export function bench(m: Mesher, x: number, z: number, ry = 0): void {
  table(m, x, z, ry, 1.9, 0.8, P.metal);
  m.push(x, z, ry);
  m.box(-0.5, 0.73, 0, 0.18, 0.1, 0.18, P.red);
  m.box(0, 0.73, 0, 0.18, 0.1, 0.18, P.lime);
  m.box(0.5, 0.73, 0, 0.18, 0.1, 0.18, P.yellow);
}

export function easel(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.box(-0.3, 0, 0, 0.05, 1.3, 0.05, P.darkWood, 0, 0.2);
  m.box(0.3, 0, 0, 0.05, 1.3, 0.05, P.darkWood, 0, 0.2);
  m.box(0, 0, -0.3, 0.05, 1.3, 0.05, P.darkWood, 0, -0.25);
  m.box(0, 0.55, 0.1, 0.8, 0.9, 0.04, P.paper);
  m.box(-0.15, 0.9, 0.13, 0.3, 0.25, 0.02, P.pink);
  m.box(0.15, 0.7, 0.13, 0.3, 0.3, 0.02, P.blue);
}

export function arcade(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.box(0, 0, 0, 0.8, 1.6, 0.7, 0x3b2a6b);
  m.box(0, 1.0, 0.36, 0.6, 0.45, 0.04, P.lime);
  m.box(0, 0.75, 0.4, 0.7, 0.1, 0.3, P.dark);
  m.cyl(-0.15, 0.85, 0.4, 0.03, 0.2, P.red);
  m.ico(0.15, 0.88, 0.4, 0.06, P.yellow);
}

export function globe(m: Mesher, x: number, z: number): void {
  m.push(x, z);
  m.cyl(0, 0, 0, 0.3, 0.06, P.dark);
  m.cyl(0, 0.06, 0, 0.04, 0.6, P.metal);
  m.ico(0, 0.95, 0, 0.35, P.blue);
  m.ico(0.12, 1.0, 0.2, 0.18, P.green);
}

export function kanban(m: Mesher, x: number, z: number, ry = 0): void {
  board(m, x, z, ry, 1.9, 1.1, P.white, [P.yellow, P.orange, P.yellow, P.lime, P.lime, P.lime, P.blue]);
}

export function safe(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.box(0, 0, 0, 0.9, 1.0, 0.7, 0x444b56);
  m.box(0, 0.08, 0.36, 0.75, 0.84, 0.03, 0x5b6573);
  m.cyl(0, 0.5, 0.4, 0.14, 0.04, P.yellow, Math.PI / 2);
  m.box(0.25, 0.4, 0.4, 0.04, 0.25, 0.04, P.metal);
}

export function pipes(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.cyl(-0.5, 0, 0, 0.09, 1.4, P.metal);
  m.cyl(0, 1.3, 0, 0.09, 1.2, P.metal, 0, Math.PI / 2);
  m.cyl(0.5, 0, 0, 0.09, 1.4, P.orange);
  m.box(0.1, 0, 0.7, 0.5, 0.45, 0.5, P.cardboard);
  m.box(0.7, 0, 0.6, 0.45, 0.4, 0.45, P.cardboard);
}

export function gauge(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.cyl(0, 0, 0, 0.05, 1.0, P.metal);
  m.cyl(0, 1.0, 0, 0.4, 0.1, P.white, Math.PI / 2);
  m.box(0.1, 1.2, 0.08, 0.04, 0.3, 0.02, P.red, 0, 0, -0.7);
}

export function magnifier(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.cyl(0, 0, 0, 0.25, 0.05, P.dark);
  m.cyl(0, 0.05, 0, 0.04, 0.9, P.metal);
  m.ico(0.15, 1.1, 0.1, 0.3, 0x9fd8ee, 0.25);
}

export function router(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  for (let i = 0; i < 4; i++) {
    m.box(0, i * 0.3, 0, 0.8, 0.22, 0.5, i % 2 ? P.dark : P.metal);
    m.box(-0.3, i * 0.3 + 0.08, 0.26, 0.06, 0.06, 0.02, P.lime);
  }
  m.cyl(-0.3, 1.2, 0, 0.02, 0.6, P.dark);
  m.cyl(0.3, 1.2, 0, 0.02, 0.6, P.dark);
}

export function typewriter(m: Mesher, x: number, z: number, ry = 0): void {
  table(m, x, z, ry, 1.2, 0.7);
  m.push(x, z, ry);
  m.box(0, 0.73, 0, 0.5, 0.14, 0.35, P.dark);
  m.box(0, 0.87, -0.15, 0.4, 0.05, 0.03, P.paper);
}

export function cabinet(m: Mesher, x: number, z: number, ry = 0, w = 0.8): void {
  m.push(x, z, ry);
  m.box(0, 0, 0, w, 1.2, 0.5, P.metal);
  for (let i = 0; i < 3; i++) {
    m.box(0, 0.08 + i * 0.38, 0.26, w - 0.1, 0.3, 0.02, 0x8a97a5);
    m.box(0, 0.2 + i * 0.38, 0.28, 0.2, 0.03, 0.03, P.dark);
  }
}

export function paperStack(m: Mesher, x: number, z: number): void {
  m.push(x, z);
  m.box(0, 0, 0, 0.4, 0.18, 0.3, P.paper);
  m.box(0.05, 0.18, 0, 0.35, 0.12, 0.28, P.white, 0.2);
}

export function tv(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.box(0, 0, 0, 1.2, 0.4, 0.4, P.darkWood);
  m.box(0, 0.4, 0, 1.1, 0.65, 0.06, P.darker);
  m.box(0, 0.45, 0.04, 1.0, 0.55, 0.02, 0x3a6ea5);
}

export function coffee(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.box(0, 0, 0, 0.8, 0.8, 0.5, P.darkWood);
  m.box(0, 0.8, 0, 0.5, 0.5, 0.4, P.dark);
  m.box(0, 1.0, 0.21, 0.08, 0.08, 0.02, P.red);
  m.cyl(0.25, 0.8, 0.1, 0.07, 0.12, P.white);
}

export function cooler(m: Mesher, x: number, z: number): void {
  m.push(x, z);
  m.box(0, 0, 0, 0.45, 0.95, 0.45, P.white);
  m.cyl(0, 0.95, 0, 0.19, 0.5, 0x8fd0f0);
}

export function rug(m: Mesher, x: number, z: number, w: number, d: number, color: number): void {
  m.push(x, z);
  m.box(0, 0, 0, w, 0.03, d, color);
}

export function pedestal(m: Mesher, x: number, z: number): void {
  m.push(x, z);
  m.box(0, 0, 0, 0.5, 0.8, 0.5, P.white);
  m.ico(0, 1.05, 0, 0.22, P.pink);
}

export function crates(m: Mesher, x: number, z: number): void {
  m.push(x, z);
  m.box(0, 0, 0, 0.55, 0.45, 0.5, P.cardboard);
  m.box(0.1, 0.45, 0, 0.45, 0.4, 0.45, P.cardboard, 0.3);
}

export function gameConsole(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.box(0, 0.73, 0, 0.35, 0.07, 0.25, P.dark);
  m.box(0.4, 0.73, 0.05, 0.2, 0.05, 0.12, P.red);
}

/** Second (static) monitor next to a desk monitor. */
export function dualMonitor(m: Mesher, x: number, z: number, ry = 0): void {
  m.push(x, z, ry);
  m.cyl(0.5, 0.73, -0.2, 0.05, 0.16, P.dark);
  m.box(0.5, 0.88, -0.2, 0.42, 0.3, 0.04, 0x28446b);
}

/** Stairs cell contents: steps up to the floor above and/or a well down to the floor below. */
export function stairs(m: Mesher, up: boolean, down: boolean): void {
  m.push(0, 0);
  if (up) {
    for (let i = 0; i < 7; i++) m.box(1.7, 0, 4.6 - i * 0.5, 2.0, 0.14 * (i + 1), 0.5, i % 2 ? 0xc9ced6 : 0xdfe3e9);
    m.box(0.62, 0, 1.6, 0.06, 1.0, 3.2, P.metal);
  }
  if (down) {
    m.box(4.4, 0.01, 3.2, 1.9, 0.04, 2.6, P.darker);
    for (let i = 0; i < 4; i++) m.box(4.4, 0.05, 2.2 + i * 0.5, 1.7, 0.02, 0.2, 0x252a33);
    m.box(3.42, 0, 1.9, 0.06, 0.9, 2.6, P.metal);
    m.box(5.38, 0, 1.9, 0.06, 0.9, 2.6, P.metal);
  }
}
