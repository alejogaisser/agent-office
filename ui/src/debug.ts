import { fit, isHidden, renderInfo, stats } from "./scene/renderer";
import type { OfficeScene } from "./scene/office";
import type { Store } from "./store";

let lastDeltaAt = 0;
let lastDeltaSize = 0;

export function noteDelta(size: number): void {
  lastDeltaAt = Date.now();
  lastDeltaSize = size;
}

/** F3 overlay: frames, deltas, sessions, agents, floor, room lights, walkers, draw calls, geometries, fit. */
export function initDebug(el: HTMLElement, store: Store, office: OfficeScene): void {
  let timer: number | null = null;
  const draw = (): void => {
    let agents = 0;
    for (const s of store.sessions.values()) agents += s.agents.length;
    const last = lastDeltaAt ? new Date(lastDeltaAt).toISOString().slice(11, 23) : "-";
    const d = office.debug();
    const r = renderInfo();
    el.textContent = [
      `frames ${stats.frames}`,
      `last delta ${last} (${lastDeltaSize})`,
      `sessions ${store.sessions.size}`,
      `agents ${agents}`,
      `floor ${d.floor}`,
      `cells ${d.cells}`,
      `lit ${d.lit} dim ${d.dim} dark ${d.dark}`,
      `walkers ${d.walkers}`,
      `calls ${r.calls}`,
      `geometries ${r.geometries}`,
      `fit ${fit.ok ? "ok" : "BAD"} (${Math.round(fit.fill * 100)}%)`,
    ].join("\n");
  };
  window.addEventListener("keydown", (e) => {
    if (e.key !== "F3") return;
    e.preventDefault();
    el.hidden = !el.hidden;
    if (!el.hidden) {
      draw();
      // refresh the text only (no 3D render) while the overlay is open and visible
      timer = window.setInterval(() => {
        if (!isHidden()) draw();
      }, 500);
    } else if (timer !== null) {
      clearInterval(timer);
      timer = null;
    }
  });
}
