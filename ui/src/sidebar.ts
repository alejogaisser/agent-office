import type { Store } from "./store";
import type { Catalog } from "./types";
import { WORKING } from "./types";

export interface SidebarHandlers {
  onBuilding: (id: string) => void;
  onFloor: (team: string) => void;
}

const ARROW_MS = 3000;
type Dir = "up" | "down";
const arrows = new Map<string, { dir: Dir; at: number }>();

/** Shows an arrow on a floor row for 3 s (a finite CSS animation, then it is removed). */
export function flashArrow(sessionId: string, team: string, dir: Dir, rerender: () => void): void {
  const key = `${sessionId}:${team}`;
  const at = performance.now();
  arrows.set(key, { dir, at });
  window.setTimeout(() => {
    if (arrows.get(key)?.at === at) {
      arrows.delete(key);
      rerender();
    }
  }, ARROW_MS + 50);
}

/** Buildings newest first; the selected building expands into floor rows 5..1. */
export function renderSidebar(el: HTMLElement, store: Store, catalog: Catalog | null, h: SidebarHandlers): void {
  el.replaceChildren();
  const head = document.createElement("div");
  head.className = "elev-head";
  head.textContent = "Buildings";
  el.appendChild(head);
  for (const s of store.sorted()) {
    const selected = s.id === store.selectedId;
    const row = document.createElement("button");
    row.type = "button";
    row.className = "building" + (selected ? " selected" : "");
    row.dataset.id = s.id;
    const dot = document.createElement("span");
    dot.className = "dot " + s.status;
    const text = document.createElement("span");
    text.className = "floor-text";
    const title = document.createElement("span");
    title.className = "floor-title";
    title.textContent = s.title;
    const proj = document.createElement("span");
    proj.className = "floor-project";
    proj.textContent = s.project;
    text.append(title, proj);
    row.append(dot, text);
    row.addEventListener("click", () => h.onBuilding(s.id));
    el.appendChild(row);
    if (!selected || !catalog) continue;

    const floors = document.createElement("div");
    floors.className = "floors";
    for (const t of [...catalog.teams].sort((a, b) => b.floor - a.floor)) {
      const active = s.activeTeams.includes(t.key);
      const working = s.agents.filter((a) => a.id !== "main" && a.role.team === t.key && WORKING.has(a.activity)).length;
      const fr = document.createElement("button");
      fr.type = "button";
      fr.className = "floor" + (t.key === store.selectedFloor ? " selected" : "") + (active ? "" : " dim");
      fr.dataset.team = t.key;
      const num = document.createElement("span");
      num.className = "floor-num";
      num.textContent = String(t.floor);
      const label = document.createElement("span");
      label.className = "floor-title";
      label.textContent = t.label;
      const count = document.createElement("span");
      count.className = "floor-count";
      count.textContent = working > 0 ? String(working) : "";
      count.title = "working agents";
      const arrow = document.createElement("span");
      arrow.className = "arrow";
      const a = arrows.get(`${s.id}:${t.key}`);
      if (a) {
        const elapsed = performance.now() - a.at;
        if (elapsed < ARROW_MS) {
          arrow.classList.add(a.dir);
          arrow.textContent = a.dir === "down" ? "↓" : "↑";
          arrow.style.animationDelay = `-${Math.round(elapsed)}ms`;
        }
      }
      fr.append(num, label, count, arrow);
      fr.addEventListener("click", () => h.onFloor(t.key));
      floors.appendChild(fr);
    }
    el.appendChild(floors);
  }
}
