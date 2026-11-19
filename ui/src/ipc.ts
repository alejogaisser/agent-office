import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Catalog, Delta, Snapshot } from "./types";

export function getSnapshot(): Promise<Snapshot> {
  return invoke<Snapshot>("get_snapshot");
}

export function getCatalog(): Promise<Catalog> {
  return invoke<Catalog>("get_catalog");
}

export function getAmbientFps(): Promise<number> {
  return invoke<number>("get_ambient_fps");
}

export function setVisible(visible: boolean): Promise<void> {
  return invoke<void>("set_visible", { visible });
}

export function onDelta(cb: (d: Delta) => void): Promise<UnlistenFn> {
  return listen<Delta>("office://delta", (e) => cb(e.payload));
}

/** Rust reports window minimize/restore (WebView2 does not flip document.hidden). */
export function onMinimized(cb: (minimized: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>("office://minimized", (e) => cb(e.payload));
}
