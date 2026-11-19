import * as P from "./props";
import { P as C, type Mesher } from "./props";

/**
 * Per-room decoration in cell-local coordinates (0..6 x 0..6; the far wall is z=0 and the door
 * wall is z=6; the boss office is 12 wide). Desks and chairs come from the floor plan.
 */
export interface Theme {
  /** Floor tint; defaults to the room color mixed with white. */
  tint?: number;
  props: (m: Mesher) => void;
}

const L = 0.45; // against the left wall
const R = 5.55; // against the right wall
const FAR = 0.35; // against the far wall

const dual = (m: Mesher): void => {
  for (const x of [1.2, 3.0, 4.8]) P.dualMonitor(m, x, 2.3);
};

const T = (props: (m: Mesher) => void, tint?: number): Theme => ({ props, tint });

export const THEMES: Record<string, Theme> = {
  boss: T((m) => {
    P.rug(m, 6, 3.6, 5.2, 3.2, 0x8c3f4a);
    P.desk(m, 4.7, 1.55, Math.PI / 2, C.darkWood); // L-shaped wing
    P.chair(m, 4.2, 3.9, Math.PI, C.darkWood);
    P.chair(m, 7.8, 3.9, Math.PI, C.darkWood);
    P.sofa(m, 10.2, 4.6, -Math.PI / 2, 2.4, 0x7a4e3a);
    P.plant(m, 0.6, 0.7);
    P.plant(m, 11.4, 0.7);
  }, 0xd8c7a8),
  "software-architect": T((m) => {
    P.table(m, 3, 4.6, 0, 1.5, 0.9, C.blue);
    P.wallBoard(m, 3, FAR, 0, 2, 0.8, C.white, [C.blue, C.teal, C.blue]);
  }),
  "software-engineer": T((m) => {
    dual(m);
    P.plant(m, R, 4.9);
  }),
  "software-test-engineer": T((m) => {
    P.bench(m, 3, 4.7);
  }),
  "software-debugger": T((m) => {
    P.magnifier(m, 3, 4.6);
    P.crates(m, R, 4.9);
  }),
  "software-code-reviewer": T((m) => {
    P.armchair(m, 3, 4.7, Math.PI);
    P.shelf(m, 3, FAR);
  }),
  "software-security-reviewer": T((m) => {
    P.safe(m, 3, 4.8, Math.PI);
    P.safe(m, R, FAR + 0.3);
  }),
  "software-performance-reviewer": T((m) => {
    P.gauge(m, 3, 4.6);
    P.gauge(m, R, 4.9);
  }),
  "software-db-engineer": T((m) => {
    P.rack(m, 1.2, FAR + 0.3);
    P.rack(m, 2.2, FAR + 0.3);
    P.rack(m, 3.8, FAR + 0.3);
    P.rack(m, 4.8, FAR + 0.3);
  }),
  "software-devops": T((m) => {
    P.pipes(m, 3, 4.5);
    P.crates(m, L, 4.9);
  }),
  "software-diagrammer": T((m) => {
    P.board(m, 1.6, 4.9, Math.PI, 1.4, 0.9);
    P.board(m, 4.4, 4.9, Math.PI, 1.4, 0.9, C.white, [C.purple, C.pink]);
  }),
  "software-docs-writer": T((m) => {
    P.shelf(m, 1.5, FAR);
    P.shelf(m, 4.5, FAR);
    P.paperStack(m, 3, 4.6);
  }),
  "software-other": T((m) => {
    P.plant(m, R, 4.9);
  }),
  "content-brand": T((m) => {
    P.wallBoard(m, 3, FAR, 0, 2.4, 0.9, C.white, [C.pink, C.yellow, C.teal, C.blue, C.orange, C.purple, C.red, C.green]);
  }),
  "content-designer": T((m) => {
    P.easel(m, 3, 4.6);
  }),
  "content-qa": T((m) => {
    P.board(m, 3, 4.8, Math.PI, 1.6, 1.0, C.white, [C.lime, C.lime, C.red, C.lime]);
  }),
  "content-scout": T((m) => {
    P.globe(m, 3, 4.6);
    P.table(m, R - 0.2, 4.7, 0, 1.0, 0.7, C.paper);
  }),
  "content-writer": T((m) => {
    P.typewriter(m, 3, 4.6);
    P.paperStack(m, R, 4.9);
  }),
  "content-other": T((m) => {
    P.plant(m, R, 4.9);
  }),
  "product-discovery": T((m) => {
    P.wallBoard(m, 3, FAR, 0, 2.6, 1.0, 0xfff3b0, [C.yellow, C.pink, C.lime, C.orange, C.yellow, C.blue, C.pink, C.lime, C.yellow, C.orange]);
  }),
  "product-reviewer": T((m) => {
    P.armchair(m, 3, 4.7, Math.PI);
    P.paperStack(m, 4.1, 4.7);
  }),
  "product-spec": T((m) => {
    P.cabinet(m, 1.5, FAR);
    P.cabinet(m, 2.5, FAR);
    P.cabinet(m, 3.5, FAR);
    P.cabinet(m, 4.5, FAR);
  }),
  "product-ux": T((m) => {
    P.board(m, 1.6, 4.9, Math.PI, 1.2, 0.9, C.white, [C.teal, C.blue]);
    P.board(m, 3.2, 4.9, Math.PI, 1.2, 0.9, C.white, [C.pink, C.teal]);
    P.board(m, 4.8, 4.9, Math.PI, 1.2, 0.9, C.white, [C.orange, C.blue]);
  }),
  "product-other": T((m) => {
    P.plant(m, R, 4.9);
  }),
  "game-designer": T((m) => {
    P.table(m, 3, 4.6, 0, 1.4, 1.0, 0x2f6b3f);
    P.paperStack(m, 2.6, 4.6);
  }),
  "game-network-engineer": T((m) => {
    P.router(m, 4.9, 4.7);
    P.router(m, 1.1, 4.7);
  }),
  "game-programmer": T((m) => {
    dual(m);
    P.gameConsole(m, 3.0, 2.3);
  }),
  "game-qa": T((m) => {
    P.arcade(m, 1.0, FAR + 0.4);
    P.arcade(m, 5.0, FAR + 0.4);
    P.tv(m, 3, 5.2, Math.PI);
    P.sofa(m, 3, 4.1, 0, 1.6, 0x7b5ea7);
  }),
  "game-technical-artist": T((m) => {
    P.easel(m, 1.4, 4.7);
    P.pedestal(m, 4.5, 4.7);
  }),
  "game-other": T((m) => {
    P.plant(m, R, 4.9);
  }),
  explore: T((m) => {
    P.globe(m, 1.4, 4.7);
    P.table(m, 4.2, 4.7, 0, 1.6, 0.9, C.paper);
  }),
  plan: T((m) => {
    P.kanban(m, 3, 4.8, Math.PI);
  }),
  "general-purpose": T((m) => {
    P.desk(m, 4.2, 4.7, Math.PI);
    P.cabinet(m, 1.6, 4.9, Math.PI);
  }),
  "general-other": T((m) => {
    P.plant(m, R, 4.9);
  }),
  lounge: T((m) => {
    P.rug(m, 3, 3.4, 4, 3, 0x5c7a6a);
    P.sofa(m, 3, 4.0, 0, 2.4, 0x7a6d8f);
    P.coffee(m, 0.8, 0.7);
    P.cooler(m, 5.2, 0.7);
    P.plant(m, L, 5.2);
    P.plant(m, R, 5.2);
  }, 0xcfd8cf),
  stairs: T(() => undefined, 0xdadde3),
};
