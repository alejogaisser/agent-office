import * as THREE from "three";
import { addTween, killTween, stepTweens, type Tween } from "./tweens";

/** `frames` is shown in the F3 overlay and used to verify render-on-demand. */
export const stats = { frames: 0 };

export const scene = new THREE.Scene();
export const camera = new THREE.OrthographicCamera(-10, 10, 7, -7, 0.1, 400);

let renderer: THREE.WebGLRenderer | null = null;
let host: HTMLElement;
let scheduled = false;
let onFrame: ((now: number) => void) | null = null;
let ambientTimer: number | null = null;
let minimized = false;

const ZOOM_MIN = 0.6;
const ZOOM_MAX = 2.5;
const ELEVATION = (36 * Math.PI) / 180;
const MARGIN = 1.05;
const INITIAL_AZIMUTH = Math.PI / 4;

let zoom = 1;
let azimuth = INITIAL_AZIMUTH;
let azTarget = INITIAL_AZIMUTH;
let azTween: Tween | null = null;
const floorBox = new THREE.Box3(new THREE.Vector3(-5, 0, -5), new THREE.Vector3(5, 2, 5));
const cameraListeners: (() => void)[] = [];

/** `fit ok` for the F3 overlay: every bbox corner projects inside the canvas and fills the view. */
export const fit = { ok: true, fill: 1 };

/** Hidden = page not visible OR window minimized (WebView2 never flips document.hidden on minimize). */
export function isHidden(): boolean {
  return document.hidden || minimized;
}

export function setMinimized(v: boolean): void {
  minimized = v;
  if (v) setAmbient(0, false);
  else requestRender();
}

/** Called after every camera change (fit, rotate, zoom, resize). */
export function onCameraChange(cb: () => void): void {
  cameraListeners.push(cb);
}

const corners: THREE.Vector3[] = Array.from({ length: 8 }, () => new THREE.Vector3());
const tmp = new THREE.Vector3();

function boxCorners(): THREE.Vector3[] {
  const { min, max } = floorBox;
  let i = 0;
  for (const x of [min.x, max.x]) for (const y of [min.y, max.y]) for (const z of [min.z, max.z]) corners[i++].set(x, y, z);
  return corners;
}

function applyCamera(): void {
  const w = Math.max(1, host.clientWidth);
  const h = Math.max(1, host.clientHeight);
  const aspect = w / h;
  const c = floorBox.getCenter(new THREE.Vector3());
  const dir = new THREE.Vector3(
    Math.cos(ELEVATION) * Math.sin(azimuth),
    Math.sin(ELEVATION),
    Math.cos(ELEVATION) * Math.cos(azimuth),
  );
  camera.position.copy(c).addScaledVector(dir, 120);
  camera.lookAt(c);
  camera.updateMatrixWorld();
  // extents of the box in view space
  const inv = camera.matrixWorldInverse.copy(camera.matrixWorld).invert();
  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;
  for (const p of boxCorners()) {
    tmp.copy(p).applyMatrix4(inv);
    minX = Math.min(minX, tmp.x);
    maxX = Math.max(maxX, tmp.x);
    minY = Math.min(minY, tmp.y);
    maxY = Math.max(maxY, tmp.y);
  }
  const cx = (minX + maxX) / 2;
  const cy = (minY + maxY) / 2;
  const halfH = Math.max(((maxX - minX) / 2) * MARGIN / aspect, ((maxY - minY) / 2) * MARGIN) / zoom;
  const halfW = halfH * aspect;
  camera.left = cx - halfW;
  camera.right = cx + halfW;
  camera.top = cy + halfH;
  camera.bottom = cy - halfH;
  camera.updateProjectionMatrix();
  camera.updateMatrixWorld();
  checkFit();
  for (const cb of cameraListeners) cb();
}

function checkFit(): void {
  if (zoom !== 1) return; // `fit` describes the fitted view only
  let ok = true;
  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;
  for (const p of boxCorners()) {
    tmp.copy(p).project(camera);
    if (Math.abs(tmp.x) > 1.0001 || Math.abs(tmp.y) > 1.0001) ok = false;
    minX = Math.min(minX, tmp.x);
    maxX = Math.max(maxX, tmp.x);
    minY = Math.min(minY, tmp.y);
    maxY = Math.max(maxY, tmp.y);
  }
  const fill = Math.max((maxX - minX) / 2, (maxY - minY) / 2);
  fit.fill = fill;
  fit.ok = ok && fill >= 0.7;
}

/** Sets the floor bounding box the camera frames, and refits (zoom 1). */
export function fitToBox(box: THREE.Box3): void {
  floorBox.copy(box);
  zoom = 1;
  if (host) applyCamera();
  requestRender();
}

/** Restores the fit zoom (the azimuth stays). */
export function resetZoom(): void {
  zoom = 1;
  applyCamera();
  requestRender();
}

/** Turns the view by 90 degrees in 300 ms. Rapid clicks accumulate on the target angle. */
export function rotate(dir: 1 | -1): void {
  if (azTween) killTween(azTween);
  azTarget += dir * (Math.PI / 2);
  const from = azimuth;
  const to = azTarget;
  azTween = runTween(
    300,
    (t) => {
      azimuth = from + (to - from) * t;
      applyCamera();
    },
    () => {
      azimuth = to;
      azTween = null;
      applyCamera();
    },
  );
}

export function renderInfo(): { calls: number; geometries: number } {
  return renderer
    ? { calls: renderer.info.render.calls, geometries: renderer.info.memory.geometries }
    : { calls: 0, geometries: 0 };
}

export function initRenderer(canvas: HTMLCanvasElement, stage: HTMLElement, frameHook: (now: number) => void): void {
  host = stage;
  onFrame = frameHook;
  renderer = new THREE.WebGLRenderer({ canvas, antialias: false, alpha: false, powerPreference: "low-power" });
  // GPU memory scales with canvas pixels; cap the ratio at 1 (flat low-poly art stays readable)
  renderer.setPixelRatio(1);
  renderer.shadowMap.enabled = false;
  renderer.setClearColor(0x1b1f27, 1);

  scene.add(new THREE.HemisphereLight(0xffffff, 0x667085, 1.1));
  const sun = new THREE.DirectionalLight(0xffffff, 1.2);
  sun.position.set(6, 12, 4);
  scene.add(sun);

  const resize = (): void => {
    renderer?.setSize(Math.max(1, host.clientWidth), Math.max(1, host.clientHeight), false);
    applyCamera();
    requestRender();
  };
  new ResizeObserver(resize).observe(host);
  canvas.addEventListener(
    "wheel",
    (e) => {
      e.preventDefault();
      zoom = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, zoom * (e.deltaY < 0 ? 1.1 : 1 / 1.1)));
      applyCamera();
      requestRender();
    },
    { passive: false },
  );
  resize();
}

function frame(now: number): void {
  scheduled = false;
  if (isHidden() || !renderer) return;
  const more = stepTweens(now);
  onFrame?.(now);
  renderer.render(scene, camera);
  stats.frames++;
  if (more) requestRender();
}

/** Coalesces any number of requests into one animation frame. */
export function requestRender(): void {
  if (scheduled || isHidden()) return;
  scheduled = true;
  requestAnimationFrame(frame);
}

/** Runs a tween; rAF stays active only while tweens are running. */
export function runTween(ms: number, update: (t: number) => void, done?: () => void): Tween {
  const tw = addTween(ms, update, done);
  requestRender();
  return tw;
}

/** Ambient animation: an interval (not a rAF loop), only while `active()` holds. */
export function setAmbient(fps: number, active: boolean): void {
  const want = active && fps > 0 && !isHidden();
  if (!want) {
    if (ambientTimer !== null) {
      clearInterval(ambientTimer);
      ambientTimer = null;
    }
    return;
  }
  if (ambientTimer !== null) return;
  ambientTimer = window.setInterval(requestRender, 1000 / fps);
}
