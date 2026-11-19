import * as THREE from "three";

export interface Tween {
  start: number;
  ms: number;
  update: (t: number) => void;
  done?: () => void;
  dead: boolean;
}

const active: Tween[] = [];

export function addTween(ms: number, update: (t: number) => void, done?: () => void): Tween {
  const tw: Tween = { start: performance.now(), ms: Math.max(1, ms), update, done, dead: false };
  active.push(tw);
  return tw;
}

/** Stops a tween without calling `update` or `done` again. */
export function killTween(tw: Tween): void {
  tw.dead = true;
}

export function hasTweens(): boolean {
  return active.length > 0;
}

/** Advances all tweens; returns true while any remain. */
export function stepTweens(now: number): boolean {
  for (let i = active.length - 1; i >= 0; i--) {
    const tw = active[i];
    if (tw.dead) {
      active.splice(i, 1);
      continue;
    }
    const t = Math.min(1, Math.max(0, (now - tw.start) / tw.ms));
    tw.update(t);
    if (t >= 1) {
      const k = active.indexOf(tw);
      if (k >= 0) active.splice(k, 1);
      tw.done?.();
    }
  }
  return active.length > 0;
}

const easeOutBack = (t: number): number => {
  const c1 = 1.70158;
  const c3 = c1 + 1;
  return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
};

/** Pop-in: scale 0 -> 1 over `ms` with a small overshoot. */
export function popIn(obj: THREE.Object3D, ms = 300): Tween {
  obj.scale.setScalar(0.001);
  return addTween(
    ms,
    (t) => obj.scale.setScalar(Math.max(0.001, easeOutBack(t))),
    () => obj.scale.setScalar(1),
  );
}
