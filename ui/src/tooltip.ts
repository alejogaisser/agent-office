import * as THREE from "three";
import { camera, isHidden } from "./scene/renderer";
import type { AgentView } from "./types";

const FILE_TOOLS = new Set(["Read", "Write", "Edit", "MultiEdit", "NotebookEdit"]);

export function basename(p: string): string {
  const parts = p.split(/[\\/]/).filter(Boolean);
  return parts.length ? parts[parts.length - 1] : p;
}

export function targetLabel(a: AgentView): string {
  if (!a.target) return "";
  if (a.toolName && FILE_TOOLS.has(a.toolName)) return basename(a.target);
  return a.target.slice(0, 60);
}

export function mmss(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000));
  return `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
}

function fmt(n: number): string {
  return n >= 10_000 ? `${(n / 1000).toFixed(1)}k` : String(n);
}

export function tokensLabel(a: AgentView): string {
  const p = a.tokens.partial ? ">=" : "";
  return `${p}${fmt(a.tokens.output)} / ${p}${fmt(a.tokens.context)}`;
}

interface Deps {
  canvas: HTMLCanvasElement;
  el: HTMLElement;
  pickables: () => THREE.Object3D[];
  agentFor: (id: string) => AgentView | undefined;
}

/** pointermove throttled to 30 Hz, raycasting only against person meshes. */
export function initTooltip(d: Deps): void {
  const ray = new THREE.Raycaster();
  const ndc = new THREE.Vector2();
  let last = 0;
  let timer: number | null = null;
  let current: string | null = null;
  let px = 0;
  let py = 0;

  const fill = (): void => {
    const a = current ? d.agentFor(current) : undefined;
    if (!a) {
      hide();
      return;
    }
    const target = targetLabel(a);
    d.el.replaceChildren();
    const lines = [
      a.role.label,
      `${a.activity}${target ? ": " + target : ""}`,
      `active ${mmss(Date.now() - a.activitySince)}`,
      `tokens ${tokensLabel(a)}`,
    ];
    lines.forEach((t, i) => {
      const row = document.createElement("div");
      if (i === 0) row.className = "tt-title";
      row.textContent = t;
      d.el.appendChild(row);
    });
  };

  const hide = (): void => {
    current = null;
    d.el.hidden = true;
    if (timer !== null) {
      clearInterval(timer);
      timer = null;
    }
  };

  d.canvas.addEventListener("pointermove", (e) => {
    const now = performance.now();
    if (now - last < 1000 / 30) return;
    last = now;
    const r = d.canvas.getBoundingClientRect();
    ndc.set(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
    ray.setFromCamera(ndc, camera);
    const hit = ray.intersectObjects(d.pickables(), false)[0];
    const id = (hit?.object.userData.agentId as string | undefined) ?? null;
    if (!id) {
      hide();
      return;
    }
    px = e.clientX - r.left;
    py = e.clientY - r.top;
    current = id;
    fill();
    d.el.hidden = false;
    const tw = d.el.offsetWidth;
    const th = d.el.offsetHeight;
    const x = Math.max(0, Math.min(px + 14, r.width - tw - 4));
    const y = Math.max(0, Math.min(py + 14, r.height - th - 4));
    d.el.style.transform = `translate(${x}px, ${y}px)`;
    // 1 s update timer runs only while the tooltip is visible
    if (timer === null) timer = window.setInterval(fill, 1000);
  });
  d.canvas.addEventListener("pointerleave", hide);
  document.addEventListener("visibilitychange", () => {
    if (isHidden()) hide();
  });
}
