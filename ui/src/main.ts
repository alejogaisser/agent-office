import "./styles.css";
import { getAmbientFps, getCatalog, getSnapshot, onDelta, onMinimized, setVisible } from "./ipc";
import { Store } from "./store";
import { flashArrow, renderSidebar } from "./sidebar";
import { initTooltip } from "./tooltip";
import { initDebug, noteDelta } from "./debug";
import {
  initRenderer,
  isHidden,
  onCameraChange,
  requestRender,
  resetZoom,
  rotate,
  setAmbient,
  setMinimized,
} from "./scene/renderer";
import { OfficeScene } from "./scene/office";
import type { Catalog, Delta, Handoff } from "./types";

const $ = <T extends HTMLElement>(id: string): T => document.getElementById(id) as T;

const store = new Store();
const sidebar = $("sidebar");
const stage = $("stage");
const canvas = $<HTMLCanvasElement>("scene");
const overlay = $("overlay");
const empty = $("empty");

let ambientFps = 2;
let catalog: Catalog | null = null;
let catalogFailed = false;
const office = new OfficeScene(overlay, $("sign"));

initRenderer(canvas, stage, (now) => office.frame(now, stage.clientWidth, stage.clientHeight));
onCameraChange(() => office.repositionLabels(stage.clientWidth, stage.clientHeight));
initTooltip({
  canvas,
  el: $("tooltip"),
  pickables: () => office.pickables(),
  agentFor: (id) => (store.selectedId ? store.agent(store.selectedId, id) : undefined),
});
initDebug($("debug"), store, office);
$("rot-left").addEventListener("click", () => rotate(-1));
$("rot-right").addEventListener("click", () => rotate(1));
$("fit").addEventListener("click", () => resetZoom());

function updateAmbient(): void {
  setAmbient(ambientFps, !isHidden() && office.hasWorking());
}

function updateEmpty(): void {
  const msg = catalogFailed
    ? "catalog unavailable"
    : store.error
      ? store.error
      : store.sessions.size === 0
        ? "No open Claude Code sessions found."
        : "";
  empty.textContent = msg;
  empty.hidden = msg === "";
}

function drawSidebar(): void {
  renderSidebar(sidebar, store, catalog, { onBuilding: selectBuilding, onFloor: selectFloor });
}

function buildScene(): void {
  office.buildFloor(catalogFailed ? null : store.selected(), store.selectedFloor);
}

function selectBuilding(id: string): void {
  if (store.selectedId === id && office.sessionId === id) return;
  store.selectBuilding(id);
  drawSidebar();
  buildScene();
  updateAmbient();
}

function selectFloor(team: string): void {
  if (store.selectedFloor === team && office.floorKey === team) return;
  store.selectFloor(team);
  drawSidebar();
  buildScene();
  updateAmbient();
}

function rebuildAll(): void {
  drawSidebar();
  buildScene();
  updateEmpty();
  updateAmbient();
  requestRender();
}

/** Floor rows show a down or up arrow for 3 s when a handoff crosses floors. */
function flashHandoffs(sid: string, handoffs: Handoff[]): void {
  const floorOf = (team: string): number => catalog?.teams.find((t) => t.key === team)?.floor ?? 0;
  for (const h of handoffs) {
    if (h.sessionId !== sid) continue;
    const from = h.relayFromAgentId ? store.agent(sid, h.relayFromAgentId)?.role.team ?? null : h.fromTeam;
    const to = h.toTeam;
    if (!from || !to || from === to) continue;
    const dir = floorOf(to) < floorOf(from) ? "down" : "up";
    flashArrow(sid, from, dir, drawSidebar);
    flashArrow(sid, to, dir === "down" ? "up" : "down", drawSidebar);
  }
}

function onDeltaMsg(d: Delta): void {
  if (isHidden()) {
    store.pendingHandoffs = [];
    return;
  }
  noteDelta(d.agentsUpserted.length + d.sessionsUpserted.length + d.spawned.length);
  const ch = store.apply(d);
  const handoffs = store.pendingHandoffs;
  store.pendingHandoffs = [];
  const sel = store.selected();
  if (sel) flashHandoffs(sel.id, handoffs);
  drawSidebar();
  updateEmpty();
  if (!sel) {
    office.buildFloor(null, null);
    return;
  }
  if (catalogFailed) return;
  if (office.sessionId !== sel.id || office.floorKey !== store.selectedFloor) {
    buildScene();
  } else if (ch.sessions.has(sel.id)) {
    office.applyDelta(sel, handoffs);
  }
  updateAmbient();
}

async function reloadSnapshot(): Promise<void> {
  store.load(await getSnapshot());
  rebuildAll();
}

document.addEventListener("visibilitychange", () => {
  void setVisible(!document.hidden);
  if (document.hidden) {
    store.pendingHandoffs = [];
    updateAmbient();
  } else {
    void reloadSnapshot();
  }
});

async function start(): Promise<void> {
  try {
    ambientFps = await getAmbientFps();
  } catch {
    // keep the default
  }
  try {
    catalog = await getCatalog();
    office.setCatalog(catalog);
  } catch {
    catalogFailed = true;
    updateEmpty();
  }
  await onDelta(onDeltaMsg);
  await onMinimized((m) => {
    setMinimized(m);
    if (m) store.pendingHandoffs = [];
    updateAmbient();
    if (!m) void reloadSnapshot();
  });
  await reloadSnapshot();
}

void start();
