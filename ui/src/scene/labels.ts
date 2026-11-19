import * as THREE from "three";
import { camera } from "./renderer";
import type { Cell } from "../types";

/** One DOM label per cell. Positions change only when the camera changes, not every frame. */
export class Labels {
  private items = new Map<string, { el: HTMLDivElement; world: THREE.Vector3; base: string }>();
  private tmp = new THREE.Vector3();

  constructor(private parent: HTMLElement) {}

  build(cells: Cell[]): void {
    this.clear();
    for (const c of cells) {
      const el = document.createElement("div");
      el.className = "cell-label cell-" + c.kind;
      el.textContent = c.label;
      this.parent.appendChild(el);
      this.items.set(c.key, {
        el,
        world: new THREE.Vector3(c.rect.x + c.rect.w / 2, 1.0, c.rect.z + 0.6),
        base: c.label,
      });
    }
  }

  /** Done tally next to the room name. */
  setTally(key: string, done: number): void {
    const it = this.items.get(key);
    if (!it) return;
    const text = done > 0 ? `${it.base} ✓${done}` : it.base;
    if (it.el.textContent !== text) it.el.textContent = text;
  }

  count(): number {
    return this.items.size;
  }

  reposition(width: number, height: number): void {
    for (const it of this.items.values()) {
      this.tmp.copy(it.world).project(camera);
      const visible = Math.abs(this.tmp.x) <= 1.05 && Math.abs(this.tmp.y) <= 1.05;
      it.el.style.display = visible ? "block" : "none";
      if (visible) {
        it.el.style.transform = `translate(${((this.tmp.x + 1) / 2) * width}px, ${((1 - this.tmp.y) / 2) * height}px) translate(-50%, -50%)`;
      }
    }
  }

  clear(): void {
    for (const it of this.items.values()) it.el.remove();
    this.items.clear();
  }
}
