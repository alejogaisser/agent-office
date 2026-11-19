import type { Activity } from "../types";

/** Monitor screen color per activity. */
export const MONITOR_COLOR: Record<Activity, number> = {
  thinking: 0x7fb3d5,
  writing: 0x6fcf97,
  reading: 0x56ccf2,
  running: 0xf2c94c,
  browsing: 0xbb86fc,
  delegating: 0xf2994a,
  planning: 0x9ad0c2,
  tool: 0xa0aec0,
  waiting: 0x4a5568,
  sleeping: 0x2d3748,
  done: 0x276749,
  error: 0xe53e3e,
};

/** DOM bubble text per activity (empty string = no bubble). */
export const BUBBLE: Record<Activity, string> = {
  thinking: "...",
  writing: "",
  reading: "",
  running: "",
  browsing: "",
  delegating: "",
  planning: "",
  tool: "",
  waiting: "?",
  sleeping: "zZ",
  done: "✓",
  error: "!",
};
